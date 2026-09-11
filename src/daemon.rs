use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use std::{fs, path::Path};
use std::{os::unix::prelude::PermissionsExt, sync::Arc};
use tokio::net::UnixListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::api;
use crate::consts::{LOG_DIR, SOCKET, TMP_DIR};
use crate::folders;
use crate::runtime;
use crate::state::StatePtr;

pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(RwLock::new(Vec::new()));

    folders::init(TMP_DIR)?;
    folders::init(LOG_DIR)?;

    let path = Path::new(SOCKET);

    if path.exists() {
        fs::remove_file(path)?;
    }

    let state_ptr: StatePtr = Arc::new(state);

    let listener = UnixListener::bind(path)?;

    // Packaging should place the client in the service's socket group.
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o660);
    fs::set_permissions(path, perms)?;

    info!("Listening on {}", SOCKET);

    let (tx, mut rx) = mpsc::channel(1);

    let app = api::router(state_ptr.clone());
    let server = async {
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, _) = accepted?;
                    let app = app.clone();
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        let service = TowerToHyperService::new(app);
                        if let Err(error) = Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(io, service)
                            .await
                        {
                            error!(%error, "Unix socket connection failed");
                        }
                    });
                }
                _ = rx.recv() => break,
            }
        }
        Ok::<(), std::io::Error>(())
    };

    listen_for_signal(tx.clone(), SignalKind::interrupt());
    listen_for_signal(tx.clone(), SignalKind::terminate());

    server.await?;

    terminate_all_vms(state_ptr).await;

    Ok(())
}

async fn terminate_all_vms(state_ptr: StatePtr) {
    let vms = state_ptr.read().await;

    for v in vms.iter() {
        info!("Terminating [{}]", v.name);

        if let Err(e) = runtime::terminate(&v.name).await {
            error!("Error on termination [{}, {}]", v.name, e.to_string());
        }
    }

    // Waits for last machines
    for v in vms.iter() {
        loop {
            match get_process(v.pid).await {
                Ok(value) if !value => break,
                Ok(_) => {}
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
            let output = output.split('\n').any(|x| x == pid.to_string().as_str());
            Ok(output)
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
