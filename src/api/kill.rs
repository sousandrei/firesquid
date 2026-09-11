use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use tracing::error;

use super::VmInput;
use crate::state;
use crate::state::StatePtr;

//TODO: process kill into vm package
pub async fn handler(
    State(state_ptr): State<StatePtr>,
    Json(body): Json<VmInput>,
) -> impl IntoResponse {
    let pid = state::get_vm_pid(state_ptr.clone(), &body.vm_name)
        .await
        .unwrap_or(0);

    if pid == 0 {
        return StatusCode::NOT_FOUND.into_response();
    }

    let mut child = match tokio::process::Command::new("kill")
        .arg(pid.to_string())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);

            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    if let Err(e) = child.wait().await {
        error!("{}", e);

        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    };

    StatusCode::OK.into_response()
}
