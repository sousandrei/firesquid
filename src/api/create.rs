use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use tracing::error;

use super::VmInput;
use crate::runtime;
use crate::state::StatePtr;

pub async fn handler(
    State(state_ptr): State<StatePtr>,
    Json(body): Json<VmInput>,
) -> impl IntoResponse {
    if let Err(e) = runtime::spawn(&body.vm_name, state_ptr).await {
        error!("{}", e);

        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    };

    StatusCode::CREATED.into_response()
}
