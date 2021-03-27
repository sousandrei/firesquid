use hyper::{Body, Client, Method, Request};
use hyperlocal::{UnixClientExt, Uri};
use std::path::Path;
use tracing::info;

use crate::consts::TMP_DIR;
use crate::error::RuntimeError;

pub async fn send_request(vm_name: &str, url: &str, body: &str) -> Result<(), RuntimeError> {
    let vm_path = format!("{}/{}.socket", TMP_DIR, vm_name);
    let path = Path::new(&vm_path);
    let url: Uri = Uri::new(path, url);

    let client = Client::unix();

    let req = Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_owned()))?;

    let res = client.request(req).await?;

    info!("{}, {}", path.display(), res.status());

    Ok(())
}
