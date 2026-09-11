use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use tracing::error;

use super::VmInput;
use crate::runtime;
use crate::state;
use crate::state::StatePtr;

pub async fn handler(
    State(state_ptr): State<StatePtr>,
    Json(body): Json<VmInput>,
) -> impl IntoResponse {
    if state::get_vm_pid(state_ptr.clone(), &body.vm_name)
        .await
        .is_none()
    {
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Err(e) = runtime::terminate(&body.vm_name).await {
        error!("{}", e);

        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    };

    StatusCode::OK.into_response()
}
