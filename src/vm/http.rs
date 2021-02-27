use hyper::{Body, Client, Method, Request};
use hyperlocal::*;
use std::path::Path;
use tracing::info;

use crate::error::RuntimeError;

pub async fn send_request(vm_name: &str, url: &str, body: &str) -> Result<(), RuntimeError> {
    let vm_path = format!("./tmp/{}.socket", vm_name);
    let path = Path::new(&vm_path);
    let url: Uri = Uri::new(path, url).into();

    let client = Client::unix();

    let req = match Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_owned()))
    {
        Ok(req) => req,
        Err(e) => {
            let msg = format!("Error building request [{}, {}]", vm_path, e.to_string());
            return Err(RuntimeError::new(&msg));
        }
    };

    let res = match client.request(req).await {
        Ok(res) => res,
        Err(e) => {
            let msg = format!("Error getting response [{}, {}]", vm_path, e.to_string());
            return Err(RuntimeError::new(&msg));
        }
    };

    info!("{}, {}", path.display(), res.status());

    Ok(())
}
