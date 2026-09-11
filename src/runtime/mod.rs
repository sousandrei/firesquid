use std::process::Stdio;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use hyper_util::rt::TokioIo;
use nix::{
    sys::signal::{Signal, kill as send_signal},
    unistd::Pid,
};
use tokio::{
    io::AsyncRead,
    net::UnixStream,
    process::{Child, Command},
    time::{Duration, sleep},
};

use crate::{
    error::Error,
    state::{SharedState, Vm, VmStatus},
};

pub async fn start(state: &SharedState, id: &str) -> Result<(), Error> {
    let vm = state.begin_start(id).await?;
    validate_artifacts(&vm)?;
    let config = state.config.clone();
    tokio::fs::create_dir_all(config.vm_runtime_dir(id)).await?;
    let socket = config.vm_socket(id);
    let _ = tokio::fs::remove_file(&socket).await;

    let mut child = Command::new("firecracker")
        .arg("--api-sock")
        .arg(&socket)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::Runtime(format!("failed to spawn Firecracker: {error}")))?;
    let pid = child
        .id()
        .ok_or_else(|| Error::Runtime("Firecracker did not expose a process ID".to_owned()))?;
    capture_output(child.stdout.take(), config.vm_log(id), "stdout").await?;
    capture_output(child.stderr.take(), config.vm_log(id), "stderr").await?;

    if let Err(error) = wait_for_socket(&socket, &mut child).await {
        let _ = child.kill().await;
        let _ = state.mark_failed(id, &error.to_string()).await;
        cleanup(&socket).await;
        return Err(error);
    }

    if let Err(error) = FirecrackerClient::new(socket).configure(&vm).await {
        let _ = child.kill().await;
        let _ = state.mark_failed(id, &error.to_string()).await;
        cleanup(&config.vm_socket(id)).await;
        return Err(error);
    }

    state.mark_running(id, pid).await?;
    let state = state.clone();
    let id = id.to_owned();
    tokio::spawn(async move {
        let result = child.wait().await;
        match result {
            Ok(status) if status.success() => {
                let _ = state.mark_stopped(&id).await;
            }
            Ok(status) => {
                let _ = state
                    .mark_failed(&id, &format!("Firecracker exited with {status}"))
                    .await;
            }
            Err(error) => {
                let _ = state.mark_failed(&id, &error.to_string()).await;
            }
        }
        cleanup(&state.config.vm_socket(&id)).await;
    });
    Ok(())
}

pub async fn stop(state: &SharedState, id: &str) -> Result<(), Error> {
    let vm = active_vm(state, id).await?;
    state.mark_stopping(id).await?;
    FirecrackerClient::new(state.config.vm_socket(&vm.id))
        .action("SendCtrlAltDel")
        .await
}

pub async fn kill(state: &SharedState, id: &str) -> Result<(), Error> {
    let vm = active_vm(state, id).await?;
    let pid = vm
        .pid
        .ok_or_else(|| Error::Runtime(format!("VM has no process ID: {id}")))?;
    send_signal(
        Pid::from_raw(
            i32::try_from(pid).map_err(|_| Error::Runtime("invalid process ID".to_owned()))?,
        ),
        Signal::SIGKILL,
    )
    .map_err(|error| Error::Runtime(format!("failed to kill VM: {error}")))?;
    state.mark_stopping(id).await
}

async fn active_vm(state: &SharedState, id: &str) -> Result<Vm, Error> {
    let vm = state
        .get_vm(id)
        .await?
        .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
    if !matches!(
        vm.status,
        VmStatus::Starting | VmStatus::Running | VmStatus::Stopping
    ) {
        return Err(Error::InvalidRequest(format!("VM is not running: {id}")));
    }
    Ok(vm)
}

fn validate_artifacts(vm: &Vm) -> Result<(), Error> {
    if vm.kernel_path.is_empty() || vm.rootfs_path.is_empty() {
        return Err(Error::InvalidRequest(
            "VM requires kernel and rootfs paths".to_owned(),
        ));
    }
    Ok(())
}

async fn wait_for_socket(socket: &std::path::Path, child: &mut Child) -> Result<(), Error> {
    for _ in 0..100 {
        if socket.exists() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(Error::Runtime(format!(
                "Firecracker exited before API was ready: {status}"
            )));
        }
        sleep(Duration::from_millis(10)).await;
    }
    Err(Error::Runtime(
        "timed out waiting for Firecracker API socket".to_owned(),
    ))
}

async fn capture_output<R>(
    reader: Option<R>,
    path: std::path::PathBuf,
    stream: &'static str,
) -> Result<(), Error>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let reader =
        reader.ok_or_else(|| Error::Runtime(format!("Firecracker {stream} pipe unavailable")))?;
    tokio::spawn(async move {
        match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
        {
            Ok(mut file) => {
                if let Err(error) =
                    tokio::io::copy(&mut tokio::io::BufReader::new(reader), &mut file).await
                {
                    tracing::error!(%error, stream, "failed to capture Firecracker output");
                }
            }
            Err(error) => tracing::error!(%error, stream, "failed to open Firecracker log"),
        }
    });
    Ok(())
}

async fn cleanup(socket: &std::path::Path) {
    let _ = tokio::fs::remove_file(socket).await;
}

struct FirecrackerClient {
    socket: std::path::PathBuf,
}

impl FirecrackerClient {
    fn new(socket: std::path::PathBuf) -> Self {
        Self { socket }
    }

    async fn configure(&self, vm: &Vm) -> Result<(), Error> {
        self.request(Method::PUT, "/boot-source", format!(
            r#"{{"kernel_image_path":"{}","boot_args":"console=ttyS0 reboot=k panic=1 pci=off"}}"#,
            escape_json(&vm.kernel_path),
        )).await?;
        self.request(Method::PUT, "/drives/rootfs", format!(
            r#"{{"drive_id":"rootfs","path_on_host":"{}","is_root_device":true,"is_read_only":false}}"#,
            escape_json(&vm.rootfs_path),
        )).await?;
        self.request(
            Method::PUT,
            "/machine-config",
            format!(
                r#"{{"vcpu_count":{},"mem_size_mib":{}}}"#,
                vm.vcpus, vm.memory_mib,
            ),
        )
        .await?;
        self.action("InstanceStart").await
    }

    async fn action(&self, action: &str) -> Result<(), Error> {
        self.request(
            Method::POST,
            "/actions",
            format!(r#"{{"action_type":"{action}"}}"#),
        )
        .await
    }

    async fn request(&self, method: Method, path: &str, body: String) -> Result<(), Error> {
        let stream = UnixStream::connect(&self.socket).await?;
        let io = TokioIo::new(stream);
        let (mut sender, connection) =
            hyper::client::conn::http1::handshake(io)
                .await
                .map_err(|error| {
                    Error::Runtime(format!("Firecracker API handshake failed: {error}"))
                })?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::debug!(%error, "Firecracker API connection closed");
            }
        });
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|error| Error::Runtime(format!("invalid Firecracker request: {error}")))?;
        let response = sender
            .send_request(request)
            .await
            .map_err(|error| Error::Runtime(format!("Firecracker API request failed: {error}")))?;
        let status = response.status();
        let _ = response.into_body().collect().await;
        if status != StatusCode::NO_CONTENT && !status.is_success() {
            return Err(Error::Runtime(format!("Firecracker API returned {status}")));
        }
        Ok(())
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
