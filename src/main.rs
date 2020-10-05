use clap::{load_yaml, App};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{error, info};

mod api;
mod error;
mod folders;
mod io;
mod vm;

#[derive(Serialize, Deserialize, Debug)]
pub struct Vm {
    name: String,
    pid: u32,
}

pub type StatePtr = Arc<Mutex<State>>;

pub struct State {
    vms: Vec<Vm>,
    tmp_dir: String,
    log_dir: String,
    assets_dir: String,
    drive_name: String,
    kernel_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let tmp_dir = matches
        .value_of("tmp_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [tmp_dir]"))?;

    let log_dir = matches
        .value_of("log_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [log_dir]"))?;

    let assets_dir = matches
        .value_of("assets_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [assets_dir]"))?;

    let drive_name = matches
        .value_of("drive_name")
        .ok_or(error::RuntimeError::new("Invalid parameter [drive_name]"))?;

    let kernel_name = matches
        .value_of("kernel_name")
        .ok_or(error::RuntimeError::new("Invalid parameter [kernel_name]"))?;

    let port = matches
        .value_of("port")
        .ok_or(error::RuntimeError::new("Invalid parameter [port]"))?;

    let port: u16 = port.parse()?;

    //TODO: figure it &str is better than String here
    let state = State {
        vms: Vec::new(),
        tmp_dir: String::from(tmp_dir),
        log_dir: String::from(log_dir),
        assets_dir: String::from(assets_dir),
        drive_name: String::from(drive_name),
        kernel_name: String::from(kernel_name),
    };

    folders::init(&state.tmp_dir)?;
    folders::init(&state.log_dir)?;

    let addr = ([127, 0, 0, 1], port).into();

    let state_ptr = Arc::new(Mutex::new(state));

    let service = make_service_fn(|_| {
        let state = state_ptr.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let state = state.clone();
                async move { api::router(req, state).await }
            }))
        }
    });

    let server = Server::try_bind(&addr)?.serve(service);

    info!("Listening on http://{}", addr);

    let (tx, mut rx) = mpsc::channel(1);

    let graceful = server.with_graceful_shutdown(async {
        rx.recv().await;
        info!("Shutting down hyper server");
    });

    listen_for_signal(tx.clone(), SignalKind::interrupt());
    listen_for_signal(tx.clone(), SignalKind::terminate());

    graceful.await?;

    let state = state_ptr.lock().await;
    for v in state.vms.iter() {
        info!("Terminating [{}]", v.name);

        if let Err(e) = vm::terminate(&v.name).await {
            error!("Error on termination [{}, {}]", v.name, e.to_string());
        }
    }

    // Holds the process open for last machines
    for v in state.vms.iter() {
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

    Ok(())
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
            let output: Vec<&str> = output.split("\n").collect();

            Ok(output.contains(&pid.to_string().as_str()))
        }
    }
}

fn listen_for_signal(mut tx: tokio::sync::mpsc::Sender<SignalKind>, kind: SignalKind) {
    tokio::task::spawn(async move {
        let mut stream = signal(kind).expect(&format!("Error opening signal stream [{:?}]", kind));

        loop {
            stream.recv().await;
            info!("Termination signal received");
            break;
        }

        tx.send(kind).await.unwrap();
    });
}
