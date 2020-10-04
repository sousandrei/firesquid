use hyper::{Body, Request, Response, StatusCode};
use tracing::error;

use super::VmInput;
use crate::{vm, StatePtr};

pub async fn handler(
    request: Request<Body>,
    state_ptr: StatePtr,
) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = &hyper::body::to_bytes(request.into_body()).await?;

    let body: VmInput = match serde_json::from_slice(body_bytes) {
        Ok(j) => j,
        Err(e) => {
            error!("{}", e);

            let response = super::build_response(StatusCode::BAD_REQUEST, e.to_string());
            return Ok(response);
        }
    };

    if let Err(e) = vm::spawn(&body.vm_name, state_ptr).await {
        error!("{}", e);

        let response = super::build_response(StatusCode::BAD_REQUEST, e.to_string());
        return Ok(response);
    };

    let response = super::build_response(
        StatusCode::OK,
        serde_json::json!({
            "success": true,
        })
        .to_string(),
    );
    Ok(response)
}
