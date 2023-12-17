use bytes::Bytes;
use clap::{App, AppSettings, Arg, SubCommand};
use http_body_util::{BodyExt, Empty};
use hyper::body::Buf;
use hyper::StatusCode;

use crate::api::VmInput;
use crate::consts::SOCKET;
use crate::state::Vm;

pub async fn new() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = App::new("firesquid")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("list")
                .alias("l")
                .about("List all running machines"),
        )
        .subcommand(
            SubCommand::with_name("spawn")
                .alias("s")
                .about("Spawns a machine with given name")
                .arg(
                    Arg::with_name("machine_name")
                        .help("Name of the machine to spawn")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("kill")
                .alias("k")
                .about("Gracefully shutdown given machine")
                .arg(
                    Arg::with_name("machine_name")
                        .help("Name of the machine")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .alias("d")
                .about("Instant stop and remove given machine")
                .arg(
                    Arg::with_name("machine_name")
                        .help("Name of the machine")
                        .required(true),
                ),
        )
        .get_matches();

    let (subcommand, arg_matches) = matches.subcommand().expect("command not present");

    match subcommand {
        "list" => list().await,
        "spawn" => {
            let name = arg_matches.value_of("machine_name").unwrap_or("");
            spawn(name).await
        }
        "kill" => {
            let name = arg_matches.value_of("machine_name").unwrap_or("");
            kill(name).await
        }
        "delete" => {
            let name = arg_matches.value_of("machine_name").unwrap_or("");
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
