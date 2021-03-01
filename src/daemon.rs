use crate::state::StatePtr;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyperlocal::UnixServerExt;
use std::{fs, path::Path};
use std::{os::unix::prelude::PermissionsExt, sync::Arc};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::api;
use crate::consts::{LOG_DIR, SOCKET, TMP_DIR};
use crate::folders;
use crate::vm;

pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(RwLock::new(Vec::new()));

    folders::init(TMP_DIR)?;
    folders::init(LOG_DIR)?;

    let path = Path::new(SOCKET);

    if path.exists() {
        fs::remove_file(path)?;
    }

    let state_ptr = Arc::new(state);

    let service = make_service_fn(|_| {
        let state = state_ptr.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let state = state.clone();
                async move { api::router(req, state).await }
            }))
        }
    });

    let server = Server::bind_unix(path)?.serve(service);

    // Allowing socket to be wrote on by non-root
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o666);
    fs::set_permissions(path, perms).unwrap();

    info!("Listening on {}", SOCKET);

    let (tx, mut rx) = mpsc::channel(1);

    let graceful = server.with_graceful_shutdown(async {
        rx.recv().await;
        info!("Shutting down hyper server");
    });

    listen_for_signal(tx.clone(), SignalKind::interrupt());
    listen_for_signal(tx.clone(), SignalKind::terminate());

    graceful.await?;

    terminate_all_vms(state_ptr).await;

    Ok(())
}

async fn terminate_all_vms(state_ptr: StatePtr) {
    let vms = state_ptr.read().await;

    for v in vms.iter() {
        info!("Terminating [{}]", v.name);

        if let Err(e) = vm::terminate(&v.name).await {
            error!("Error on termination [{}, {}]", v.name, e.to_string());
        }
    }

    // Waits for last machines
    for v in vms.iter() {
        loop {
            match get_process(v.pid).await {
                Ok(value) => match value {
                    false => break,
                    true => {}
                },
                Err(e) => {
                    error!("Error on checking process [{}, {}]", v.name, e.to_string());
                    break;
                }
            }
        }
    }
}

async fn get_process(pid: u32) -> Result<bool, std::io::Error> {
    match tokio::process::Command::new("ls")
        .arg("/proc")
        .output()
        .await
    {
        Err(e) => Err(e),
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);
            let output: Vec<&str> = output.split('\n').collect();

            Ok(output.contains(&pid.to_string().as_str()))
        }
    }
}

fn listen_for_signal(tx: tokio::sync::mpsc::Sender<SignalKind>, kind: SignalKind) {
    tokio::task::spawn(async move {
        let mut stream =
            signal(kind).unwrap_or_else(|_| panic!("Error opening signal stream [{:?}]", kind));

        stream.recv().await;
        info!("Termination signal received");

        tx.send(kind).await.unwrap();
    });
}
