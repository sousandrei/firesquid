use std::{os::unix::prelude::PermissionsExt, sync::Arc};

use tokio::{
    net::{UnixListener, UnixStream},
    signal::unix::{SignalKind, signal},
};
use tracing::{error, info};

use crate::{
    api,
    config::Config,
    error::Error,
    protocol,
    state::{SharedState, State},
};

pub async fn run(config: Config) -> Result<(), Error> {
    config.prepare_directories().await?;
    let state: SharedState = Arc::new(State::open(&config.database_path()).await?);
    state.reconcile().await?;

    let _ = tokio::fs::remove_file(&config.socket_path).await;
    let listener = UnixListener::bind(&config.socket_path)?;
    let mut permissions = tokio::fs::metadata(&config.socket_path)
        .await?
        .permissions();
    permissions.set_mode(0o660);
    tokio::fs::set_permissions(&config.socket_path, permissions).await?;
    info!(socket = %config.socket_path.display(), "daemon listening");

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, state).await {
                        error!(%error, "control connection failed");
                    }
                });
            }
            _ = interrupt.recv() => break,
            _ = terminate.recv() => break,
        }
    }

    let _ = tokio::fs::remove_file(&config.socket_path).await;
    Ok(())
}

async fn handle_connection(mut stream: UnixStream, state: SharedState) -> Result<(), Error> {
    if let Some(request) = protocol::read_frame(&mut stream).await? {
        let response = if request.version != protocol::VERSION {
            protocol::Response::error(
                request.request_id,
                &Error::Protocol(format!("unsupported protocol version: {}", request.version)),
            )
        } else {
            api::dispatch(&state, request.request_id, request.operation).await
        };
        protocol::write_response(&mut stream, &response).await?;
    }
    Ok(())
}
