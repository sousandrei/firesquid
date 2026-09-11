use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Parser, Subcommand};
use tokio::net::UnixStream;

use crate::{
    config::Config,
    daemon,
    error::Error,
    protocol::{self, Operation, Request, Response, ResponseBody},
};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Parser)]
#[command(name = "firesquid", version, about = "Firesquid microVM manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Daemon,
    Status,
    Vm {
        #[command(subcommand)]
        command: VmCommand,
    },
}

#[derive(Debug, Subcommand)]
enum VmCommand {
    List,
    Create { name: String },
    Inspect { id: String },
    Delete { id: String },
}

pub async fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon => daemon::run(Config::from_env()).await,
        Command::Status => match request(Operation::Status).await? {
            ResponseBody::Status { service } => {
                println!("{service} is running");
                Ok(())
            }
            _ => Err(Error::Protocol("unexpected status response".to_owned())),
        },
        Command::Vm { command } => {
            let body = match command {
                VmCommand::List => request(Operation::VmList).await?,
                VmCommand::Create { name } => request(Operation::VmCreate { name }).await?,
                VmCommand::Inspect { id } => request(Operation::VmInspect { id }).await?,
                VmCommand::Delete { id } => request(Operation::VmDelete { id }).await?,
            };
            print_response(body)
        }
    }
}

async fn request(operation: Operation) -> Result<ResponseBody, Error> {
    let config = Config::from_env();
    let mut stream = UnixStream::connect(&config.socket_path).await?;
    let request = Request {
        version: protocol::VERSION,
        request_id: NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed),
        operation,
    };
    let payload = postcard::to_allocvec(&request)?;
    let length = u32::try_from(payload.len())
        .map_err(|_| Error::Protocol("request is too large".to_owned()))?;
    tokio::io::AsyncWriteExt::write_all(&mut stream, &length.to_be_bytes()).await?;
    tokio::io::AsyncWriteExt::write_all(&mut stream, &payload).await?;
    tokio::io::AsyncWriteExt::flush(&mut stream).await?;

    let mut length_bytes = [0; 4];
    tokio::io::AsyncReadExt::read_exact(&mut stream, &mut length_bytes).await?;
    let length = u32::from_be_bytes(length_bytes);
    if length == 0 || length > 1024 * 1024 {
        return Err(Error::Protocol("invalid response frame size".to_owned()));
    }
    let mut response_payload = vec![0; length as usize];
    tokio::io::AsyncReadExt::read_exact(&mut stream, &mut response_payload).await?;
    let response: Response = postcard::from_bytes(&response_payload)?;
    if response.version != protocol::VERSION {
        return Err(Error::Protocol("unsupported response version".to_owned()));
    }
    response
        .result
        .map_err(|error| Error::InvalidRequest(error.message))
}

fn print_response(body: ResponseBody) -> Result<(), Error> {
    match body {
        ResponseBody::Status { service } => println!("{service} is running"),
        ResponseBody::VmList(vms) => {
            for vm in vms {
                println!("{}\t{}\t{}", vm.id, vm.name, vm.status);
            }
        }
        ResponseBody::Vm(vm) => println!("{}\t{}\t{}", vm.id, vm.name, vm.status),
        ResponseBody::Empty => println!("ok"),
    }
    Ok(())
}
