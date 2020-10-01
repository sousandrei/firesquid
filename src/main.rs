use clap::{load_yaml, App};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use tracing::info;

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

pub struct State {
    vms: Arc<Mutex<Vec<Vm>>>,
    tmp_dir: String,
    log_dir: String,
    assets_dir: String,
    drive_name: String,
    kernel_name: String,
}

impl Clone for State {
    fn clone(&self) -> State {
        State {
            vms: self.vms.clone(),
            tmp_dir: self.tmp_dir.clone(),
            log_dir: self.log_dir.clone(),
            assets_dir: self.assets_dir.clone(),
            drive_name: self.drive_name.clone(),
            kernel_name: self.kernel_name.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let state = State {
        vms: Arc::new(Mutex::new(Vec::new())),
        tmp_dir: String::from(matches.value_of("tmp_dir").unwrap()),
        log_dir: String::from(matches.value_of("log_dir").unwrap()),
        assets_dir: String::from(matches.value_of("assets_dir").unwrap()),
        drive_name: String::from(matches.value_of("drive_name").unwrap()),
        kernel_name: String::from(matches.value_of("kernel_name").unwrap()),
    };

    folders::init(&state.tmp_dir)?;
    folders::init(&state.log_dir)?;

    let port = matches.value_of("port").unwrap();
    let port: u16 = port.parse().unwrap();

    let addr = ([127, 0, 0, 1], port).into();

    let service = make_service_fn(|_| {
        let state = state.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let state = state.clone();
                async move { api::router(req, state).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
