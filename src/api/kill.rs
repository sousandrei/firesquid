use hyper::{Body, Request, Response, StatusCode};

use super::VmInput;
use crate::State;

pub async fn handler(req: Request<Body>, state: State) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = &hyper::body::to_bytes(req.into_body()).await?;

    let body: VmInput = match serde_json::from_slice(body_bytes) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("{}", e);

            let mut error = Response::default();
            *error.status_mut() = StatusCode::BAD_REQUEST;
            *error.body_mut() = Body::from(e.to_string());
            return Ok(error);
        }
    };

    let mut pid = 0;

    {
        let vms = state.vms.lock().unwrap();
        if let Some(vm) = vms.iter().find(|vm| vm.name == body.vm_name) {
            pid = vm.pid;
        };
    }

    if pid == 0 {
        let mut error = Response::default();
        *error.status_mut() = StatusCode::BAD_REQUEST;
        *error.body_mut() = Body::from(format!("machine not found: {}", body.vm_name));
        return Ok(error);
    }

    let child = tokio::process::Command::new("kill")
        .arg(pid.to_string())
        .spawn()
        .unwrap();

    if let Err(e) = child.await {
        let mut error = Response::default();
        *error.status_mut() = StatusCode::BAD_REQUEST;
        *error.body_mut() = Body::from(format!("error killing vm: {}", e));
        return Ok(error);
    };

    let mut res = Response::default();
    *res.body_mut() = Body::from("success");
    Ok(res)
}
