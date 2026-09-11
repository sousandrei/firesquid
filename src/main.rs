mod api;
mod cli;
mod config;
mod daemon;
mod error;
mod image;
mod kernel;
mod network;
mod protocol;
mod runtime;
mod state;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    cli::run().await
}
