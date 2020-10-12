use hyper::{Body, Response, StatusCode};

use super::build_response;
use crate::state::StatePtr;

pub async fn handler(state_ptr: StatePtr) -> Result<Response<Body>, hyper::Error> {
    let state = state_ptr.lock().await;

    let response_json = serde_json::json!(state.vms);

    let body = match serde_json::to_string_pretty(&response_json) {
        Ok(b) => b,
        Err(e) => {
            let response = build_response(
                StatusCode::OK,
                format!("Error parsing state [{}]", e.to_string()),
            );
            return Ok(response);
        }
    };

    let response = build_response(StatusCode::OK, body);
    Ok(response)
}
