use hyper::{Body, Response, StatusCode};

use crate::StatePtr;

pub async fn handler(state_ptr: StatePtr) -> Result<Response<Body>, hyper::Error> {
    let state = state_ptr.lock().await;

    let response_json = serde_json::json!(state.vms);

    let body = serde_json::to_string_pretty(&response_json).unwrap();

    let response = super::build_response(StatusCode::OK, body);
    Ok(response)
}
