use hyper::{Body, Request, Response, StatusCode};
use tracing::error;

use super::VmInput;
use crate::{vm, State};

pub async fn handler(request: Request<Body>, state: State) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = &hyper::body::to_bytes(request.into_body()).await?;

    let body: VmInput = match serde_json::from_slice(body_bytes) {
        Ok(j) => j,
        Err(e) => {
            error!("{}", e);

            let response = super::build_response(StatusCode::BAD_REQUEST, e.to_string());
            return Ok(response);
        }
    };

    {
        let vms = state.vms.lock().unwrap();
        if let None = vms.iter().find(|vm| vm.name == body.vm_name) {
            let response = super::build_response(
                StatusCode::BAD_REQUEST,
                format!("Machine not found: {}", body.vm_name),
            );
            return Ok(response);
        };
    }

    if let Err(e) = vm::terminate(&body.vm_name).await {
        let response = super::build_response(
            StatusCode::BAD_REQUEST,
            format!("Error powering off vm: {}", e),
        );
        return Ok(response);
    };

    let response = super::build_response(StatusCode::OK, format!("Success"));
    Ok(response)
}
