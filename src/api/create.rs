use hyper::{Body, Request, Response, StatusCode};
use tracing::error;

use super::VmInput;
use crate::{vm, State};

pub async fn handler(req: Request<Body>, state: State) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = &hyper::body::to_bytes(req.into_body()).await?;

    let body: VmInput = match serde_json::from_slice(body_bytes) {
        Ok(j) => j,
        Err(e) => {
            error!("{}", e);

            let mut error = Response::default();
            *error.status_mut() = StatusCode::BAD_REQUEST;
            *error.body_mut() = Body::from(e.to_string());
            return Ok(error);
        }
    };

    match vm::spawn(&body.vm_name, state).await {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);

            let mut error = Response::default();
            *error.status_mut() = StatusCode::BAD_REQUEST;
            *error.body_mut() = Body::from(e.to_string());
            return Ok(error);
        }
    };

    let response_json = serde_json::json!({
        "vm_name": body.vm_name,
        "status":"finished"
    });

    let response = serde_json::to_string_pretty(&response_json).unwrap();
    Ok(Response::new(Body::from(response)))
}
