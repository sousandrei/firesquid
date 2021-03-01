use clap::{App, AppSettings, Arg, SubCommand};
use hyper::{body::Buf, Body, Client, Request, StatusCode};
use hyperlocal::{UnixClientExt, UnixConnector};

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

    let (subcommand, arg_matches) = matches.subcommand();
    let name = arg_matches.unwrap().value_of("machine_name").unwrap_or("");

    match subcommand {
        "list" => list().await,
        "spawn" => spawn(name).await,
        "kill" => kill(name).await,
        "delete" => delete(name).await,
        _ => unreachable!(),
    }
}

fn get_client_url(path: Option<&str>) -> (Client<UnixConnector>, hyper::Uri) {
    let path = match path {
        Some(value) => format!("/{}", value.to_string()),
        None => "/".to_string(),
    };

    let url = hyperlocal::Uri::new(SOCKET, &path).into();
    let client = Client::unix();

    (client, url)
}

async fn list() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, url) = get_client_url(None);

    let res = client.get(url).await?;
    let body = hyper::body::aggregate(res).await?;

    let vms: Vec<Vm> = serde_json::from_reader(body.reader())?;

    println!("Vms");

    for vm in vms {
        println!("pid: {} | name: {}", vm.pid, vm.name);
    }

    Ok(())
}

async fn spawn(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, url) = get_client_url(None);

    let body = Body::from(serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?);

    let req = Request::builder()
        .method("POST")
        .uri(url)
        .body(body)
        .expect("request builder");

    let res = client.request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Spawned vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}

async fn kill(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, url) = get_client_url(Some("kill"));

    let body = Body::from(serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?);

    let req = Request::builder()
        .method("POST")
        .uri(url)
        .body(body)
        .expect("request builder");

    let res = client.request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Killed vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}

async fn delete(name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (client, url) = get_client_url(None);

    let body = Body::from(serde_json::to_string(&VmInput {
        vm_name: String::from(name),
    })?);

    let req = Request::builder()
        .method("DELETE")
        .uri(url)
        .body(body)
        .expect("request builder");

    let res = client.request(req).await?;
    let status = res.status();

    match status {
        StatusCode::OK => println!("Deleted vm: {}", name),
        _ => println!("Error: {}", status),
    };

    Ok(())
}
