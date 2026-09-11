use std::env;

mod api;
mod cli;
mod consts;
mod daemon;
mod error;
mod folders;
mod image;
mod io;
mod kernel;
mod network;
mod runtime;
mod state;
mod unix_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    if env::var_os("DAEMON").is_none() {
        return cli::new().await;
    }

    daemon::start().await?;

    Ok(())
}
