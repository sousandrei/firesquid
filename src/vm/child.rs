use chrono::prelude::*;
use core::marker::{Send, Sync, Unpin};
use std::process::Stdio;
use tokio::io::AsyncRead;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::info;

use crate::error::RuntimeError;
use crate::state::StatePtr;
use crate::vm::http;

pub async fn spawn_process(
    vm_name: &str,
    state: StatePtr,
) -> Result<tokio::process::Child, RuntimeError> {
    let mut child = Command::new(format!("{}/firecracker", &state.assets_dir))
        .args(&[
            "--api-sock",
            &format!("{}/{}.socket", &state.tmp_dir, vm_name),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn vm child process");

    let time = Local::now();

    let stdout = child.stdout.take().expect("Failed to bind stdout on vm");
    handle_io(stdout, vm_name, "stdout", &state.log_dir, time);

    let stderr = child.stderr.take().expect("Failed to bind stderr on vm");
    handle_io(stderr, vm_name, "stderr", &state.log_dir, time);

    //TODO: wait for file to appear?
    sleep(Duration::from_millis(10)).await;

    set_kernel(vm_name, &state.assets_dir, &state.kernel_name).await?;
    set_drive(vm_name, &state.tmp_dir).await?;
    start_machine(vm_name).await?;

    Ok(child)
}

fn handle_io<T: 'static + AsyncRead + Send + Sync + Unpin>(
    io: T,
    name: &str,
    extension: &str,
    log_dir: &str,
    time: chrono::DateTime<chrono::Local>,
) {
    let name = String::from(name);
    let extension = String::from(extension);
    let log_dir = String::from(log_dir);

    tokio::spawn(async move {
        let mut reader = BufReader::new(io).lines();

        let mut stdout = File::create(&format!("{}/{}-{}.{}", log_dir, name, time, extension))
            .await
            .expect("error opening stdout");

        while let Some(line) = reader.next_line().await.unwrap_or(Option::None) {
            stdout
                .write_all(format!("{}\n", line).as_bytes())
                .await
                .expect("failed to write to file");
        }
    });
}

pub async fn set_kernel(
    vm_name: &str,
    assets_dir: &str,
    kernel_name: &str,
) -> Result<(), RuntimeError> {
    let url = "/boot-source";
    let kernel_path = format!("{}/{}", assets_dir, kernel_name);

    let body = serde_json::json!({
        "kernel_image_path": kernel_path,
        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
    });

    info!("Set Kernel [{}, {}]", vm_name, kernel_path);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn set_drive(vm_name: &str, tmp_dir: &str) -> Result<(), RuntimeError> {
    let url = "/drives/rootfs";
    let drive_path = format!("{}/{}.ext4", tmp_dir, vm_name);

    let body = serde_json::json!({
        "drive_id": "rootfs",
        "path_on_host": drive_path,
        "is_root_device": true,
        "is_read_only": false
    });

    info!("Set Drive [{}, {}]", vm_name, drive_path);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn start_machine(vm_name: &str) -> Result<(), RuntimeError> {
    let url = "/actions";
    let body = serde_json::json!({
        "action_type": "InstanceStart",
    });

    info!("Starting [{}]", vm_name);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn stop_machine(vm_name: &str) -> Result<(), RuntimeError> {
    let url = "/actions";
    let body = serde_json::json!({
        "action_type": "SendCtrlAltDel",
    });

    info!("Stopping [{}]", vm_name);

    http::send_request(vm_name, url, &body.to_string()).await
}
