use std::env;

mod api;
mod cli;
mod consts;
mod daemon;
mod error;
mod folders;
mod io;
mod state;
mod vm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    if env::var_os("DAEMON").is_none() {
        return cli::new().await;
    }

    daemon::start().await?;

    Ok(())
}
