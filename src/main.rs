use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::sync::Arc;
use std::{env, net::SocketAddr};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{error, info};

// mod cli;

mod api;
mod consts;
mod error;
mod folders;
mod io;
mod state;
mod vm;

use crate::state::StatePtr;

use crate::consts::{LOG_DIR, TMP_DIR};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    // if env::var_os("DAEMON").is_none() {
    //     println!("cli goes here");
    //     return Ok(());
    // }

    start_daemon().await?;

    Ok(())
}

async fn start_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(RwLock::new(Vec::new()));

    folders::init(TMP_DIR)?;
    folders::init(LOG_DIR)?;

    // TODO: check if HTTP is the way, then change port if needed
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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

    let server = Server::bind(&addr).serve(service);

    info!("Listening on http://{}", addr);

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
