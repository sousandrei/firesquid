use hyper::{Body, Request, Response, StatusCode};
use tracing::error;

use super::VmInput;
use crate::state;
use crate::state::StatePtr;
use crate::vm;

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

    if let None = state::get_vm_pid(state_ptr.clone(), body.vm_name.clone()).await {
        let response = super::build_response(
            StatusCode::BAD_REQUEST,
            format!("Machine not found: {}", body.vm_name),
        );
        return Ok(response);
    };

    if let Err(e) = vm::terminate(&body.vm_name).await {
        let response = super::build_response(
            StatusCode::BAD_REQUEST,
            format!("Error powering off vm: {}", e),
        );
        return Ok(response);
    };

    let response = super::build_response(
        StatusCode::OK,
        serde_json::json!({
            "sucess": true,
        })
        .to_string(),
    );
    Ok(response)
}
