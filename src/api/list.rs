use hyper::{Body, Response, StatusCode};

use super::build_response;
use crate::state;
use crate::state::StatePtr;

pub async fn handler(state_ptr: StatePtr) -> Result<Response<Body>, hyper::Error> {
    let vms = state::get_vms(state_ptr).await;

    let response_json = serde_json::json!(vms);

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
