use std::env;

// mod cli;

mod api;
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

    // if env::var_os("DAEMON").is_none() {
    //     println!("cli goes here");
    //     return Ok(());
    // }

    daemon::start().await?;

    Ok(())
}
