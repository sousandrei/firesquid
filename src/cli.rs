use bytes::Bytes;
use clap::{Arg, Command};
use http_body_util::{BodyExt, Empty};
use hyper::body::Buf;
use hyper::StatusCode;

use crate::api::VmInput;
use crate::consts::SOCKET;
use crate::state::Vm;

pub async fn new() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = clap::Command::new("firesquid")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .alias("l")
                .about("List all running machines"),
        )
        .subcommand(
            Command::new("spawn")
                .alias("s")
                .about("Spawns a machine with given name")
                .arg(
                    Arg::new("machine_name")
                        .help("Name of the machine to spawn")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("kill")
                .alias("k")
                .about("Gracefully shutdown given machine")
                .arg(
                    Arg::new("machine_name")
                        .help("Name of the machine")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("delete")
                .alias("d")
                .about("Instant stop and remove given machine")
                .arg(
                    Arg::new("machine_name")
                        .help("Name of the machine")
                        .required(true),
                ),
        )
        .get_matches();

    let (subcommand, arg_matches) = matches.subcommand().expect("command not present");

    match subcommand {
        "list" => list().await,
        "spawn" => {
            let name = arg_matches
                .get_one::<String>("machine_name")
                .map(|s| s.as_str())
                .unwrap();

            spawn(name).await
        }
        "kill" => {
            let name = arg_matches
                .get_one::<String>("machine_name")
                .map(|s| s.as_str())
                .unwrap();

            kill(name).await
        }
        "delete" => {
            let name = arg_matches
                .get_one::<String>("machine_name")
                .map(|s| s.as_str())
                .unwrap();

            delete(name).await
        }
        _ => unreachable!(),
    }
}

async fn list() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = unix_client::get_client(SOCKET).await.unwrap();

    let req = hyper::Request::builder()
        .uri("/")
        .body(Empty::<Bytes>::new())?;

    let res = client.send_request(req).await?;

    let body = res.collect().await?.aggregate();

    let vms: Vec<Vm> = serde_json::from_reader(body.reader())?;

    for vm in vms {
        println!("pid: {} | name: {}", vm.pid, vm.name);
    }

    Ok(())
}

async fn spawn(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = unix_client::get_client(SOCKET).await.unwrap();

    let body = serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?;

    let req = hyper::Request::builder()
        .method("POST")
        .uri("/")
        .body(body)
        .expect("request builder");

    let res = client.send_request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Spawned vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}

async fn kill(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = unix_client::get_client(SOCKET).await.unwrap();

    let body = serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?;

    let req = hyper::Request::builder()
        .method("POST")
        .uri("/kill")
        .body(body)
        .expect("request builder");

    let res = client.send_request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Killed vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}

async fn delete(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = unix_client::get_client(SOCKET).await.unwrap();

    let body = serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?;

    let req = hyper::Request::builder()
        .method("DELETE")
        .uri("/")
        .body(body)
        .expect("request builder");

    let res = client.send_request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Deleted vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}
