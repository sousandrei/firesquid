use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::state;
use crate::state::StatePtr;

pub async fn handler(State(state_ptr): State<StatePtr>) -> impl IntoResponse {
    let vms = state::get_vms(state_ptr).await;

    (StatusCode::OK, Json(vms))
}
