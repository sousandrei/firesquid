use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use clap::{Parser, Subcommand};
use tokio::net::UnixStream;

use crate::{
    config::Config,
    daemon,
    error::Error,
    image, kernel,
    protocol::{self, Operation, Request, Response, ResponseBody},
};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Parser)]
#[command(name = "firesquid", version, about = "Firesquid microVM manager")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Daemon,
    Status,
    Kernel {
        #[command(subcommand)]
        command: KernelCommand,
    },
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    Vm {
        #[command(subcommand)]
        command: VmCommand,
    },
}

#[derive(Debug, Subcommand)]
enum KernelCommand {
    Build {
        #[arg(long)]
        source: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "https://github.com/torvalds/linux.git")]
        repo: String,
        #[arg(long, default_value = "v6.18")]
        tag: String,
        #[arg(long, default_value_t = 2)]
        jobs: usize,
    },
}

#[derive(Debug, Subcommand)]
enum ImageCommand {
    Build {
        #[arg(default_value = "ubuntu")]
        profile: String,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long, default_value_t = 2048)]
        size_mib: u32,
    },
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    Clean,
}

#[derive(Debug, Subcommand)]
enum VmCommand {
    List,
    Create {
        name: String,
        #[arg(long)]
        kernel_path: String,
        #[arg(long)]
        rootfs_path: String,
        #[arg(long, default_value_t = 1)]
        vcpus: u32,
        #[arg(long, default_value_t = 512)]
        memory_mib: u32,
    },
    Start {
        id: String,
    },
    Stop {
        id: String,
    },
    Kill {
        id: String,
    },
    Inspect {
        id: String,
    },
    Delete {
        id: String,
    },
    Logs {
        id: String,
    },
}

pub async fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let json = cli.json;
    match cli.command {
        Command::Daemon => daemon::run(Config::from_env()).await,
        Command::Kernel { command } => match command {
            KernelCommand::Build {
                source,
                config: kernel_config,
                repo,
                tag,
                jobs,
            } => {
                let firesquid_config = Config::from_env();
                let output = run_blocking(move || {
                    kernel::build(
                        &firesquid_config,
                        kernel::BuildOptions {
                            source,
                            config: kernel_config,
                            repository: repo,
                            tag,
                            jobs,
                        },
                    )
                })
                .await?;
                println!("{}", output.display());
                Ok(())
            }
        },
        Command::Image { command } => match command {
            ImageCommand::Build {
                output,
                profile,
                size_mib,
            } => {
                let firesquid_config = Config::from_env();
                let output = output.unwrap_or_else(|| {
                    firesquid_config
                        .cache_dir
                        .join("rootfs")
                        .join(format!("{profile}.ext4"))
                });
                let init = firesquid_config.profiles_dir.join(&profile).join("init");
                let output_for_build = output.clone();
                run_blocking(move || {
                    let archive = image::prepare(&firesquid_config, &profile)?;
                    let result = image::build(
                        &firesquid_config,
                        &archive,
                        &output_for_build,
                        &init,
                        size_mib,
                    );
                    let _ =
                        std::fs::remove_dir_all(firesquid_config.cache_dir.join("rootfs-staging"));
                    if result.is_ok() {
                        let _ = std::fs::remove_file(archive);
                    }
                    result
                })
                .await?;
                println!("{}", output.display());
                Ok(())
            }
        },
        Command::Cache { command } => match command {
            CacheCommand::Clean => {
                let firesquid_config = Config::from_env();
                let removed = run_blocking(move || kernel::clean_cache(&firesquid_config)).await?;
                println!("removed {removed} kernel cache entries");
                Ok(())
            }
        },
        Command::Status => {
            let body = request(Operation::Status).await?;
            if json {
                print_json(&body)
            } else if let ResponseBody::Status { service, vms } = body {
                println!("{service} is running");
                println!();
                println!(
                    "{:<16} {:<16} {:<9} {:>8} {:>10} {:>10} {:>10}",
                    "ID", "NAME", "STATUS", "CPU", "MEMORY", "READ", "WRITE"
                );
                for info in vms {
                    print_vm_summary(&info);
                }
                Ok(())
            } else {
                Err(Error::Protocol("unexpected status response".to_owned()))
            }
        }
        Command::Vm { command } => {
            let body = match command {
                VmCommand::List => request(Operation::VmList).await?,
                VmCommand::Create {
                    name,
                    kernel_path,
                    rootfs_path,
                    vcpus,
                    memory_mib,
                } => {
                    request(Operation::VmCreate {
                        name,
                        kernel_path,
                        rootfs_path,
                        vcpus,
                        memory_mib,
                    })
                    .await?
                }
                VmCommand::Start { id } => request(Operation::VmStart { id }).await?,
                VmCommand::Stop { id } => request(Operation::VmStop { id }).await?,
                VmCommand::Kill { id } => request(Operation::VmKill { id }).await?,
                VmCommand::Inspect { id } => request(Operation::VmInspect { id }).await?,
                VmCommand::Delete { id } => request(Operation::VmDelete { id }).await?,
                VmCommand::Logs { id } => request(Operation::VmLogs { id }).await?,
            };
            print_response(body, json)
        }
    }
}

async fn run_blocking<F, T>(operation: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| Error::Runtime(format!("local build task failed: {error}")))?
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

fn print_response(body: ResponseBody, json: bool) -> Result<(), Error> {
    if json {
        return print_json(&body);
    }

    match body {
        ResponseBody::Status { service, .. } => println!("{service} is running"),
        ResponseBody::VmList(vms) => {
            for vm in vms {
                println!("{}\t{}\t{}", vm.id, vm.name, vm.status);
            }
        }
        ResponseBody::Vm(info) => print_vm_details(&info),
        ResponseBody::Logs(logs) => print!("{logs}"),
        ResponseBody::Empty => println!("ok"),
    }
    Ok(())
}

fn print_vm_summary(info: &protocol::VmInfo) {
    println!(
        "{:<16} {:<16} {:<9} {:>7.1}% {:>10} {:>10} {:>10}",
        info.vm.id,
        info.vm.name,
        info.vm.status,
        info.metrics.cpu_usage_percent,
        format_bytes(info.metrics.memory_bytes),
        format_bytes(info.metrics.read_bytes),
        format_bytes(info.metrics.write_bytes),
    );
}

fn print_vm_details(info: &protocol::VmInfo) {
    println!("VM:      {} ({})", info.vm.name, info.vm.id);
    println!("Status:  {}", info.vm.status);
    if let Some(error) = &info.vm.last_error {
        println!("Error:   {error}");
    }
    println!("Created: {}", info.vm.created_at);
    println!(
        "PID:     {}",
        info.vm
            .pid
            .map_or_else(|| "-".to_owned(), |pid| pid.to_string())
    );
    println!("Kernel:  {}", info.vm.kernel_path);
    println!("Rootfs:  {}", info.vm.rootfs_path);
    println!("vCPUs:   {}", info.vm.vcpus);
    println!("Memory:  {} MiB configured", info.vm.memory_mib);
    println!();
    println!("Runtime metrics:");
    println!("  CPU:       {:.1}%", info.metrics.cpu_usage_percent);
    println!("  Resident:  {}", format_bytes(info.metrics.memory_bytes));
    println!("  Read:      {}", format_bytes(info.metrics.read_bytes));
    println!("  Write:     {}", format_bytes(info.metrics.write_bytes));
    println!(
        "  Network RX: {}",
        format_bytes(info.metrics.network_received_bytes)
    );
    println!(
        "  Network TX: {}",
        format_bytes(info.metrics.network_transmitted_bytes)
    );
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn print_json(body: &ResponseBody) -> Result<(), Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(body).map_err(|error| Error::Protocol(error.to_string()))?
    );
    Ok(())
}
