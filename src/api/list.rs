use hyper::{Body, Response, StatusCode};

use crate::State;

pub async fn handler(state: State) -> Result<Response<Body>, hyper::Error> {
    let state = state.vms.lock().unwrap();
    let state = &*state;

    let response_json = serde_json::json!(state);

    let body = serde_json::to_string_pretty(&response_json).unwrap();

    let response = super::build_response(StatusCode::OK, body);
    Ok(response)
}
