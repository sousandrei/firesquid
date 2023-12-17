use hyper::{Method, Request};
use tracing::info;

use crate::consts::TMP_DIR;
use crate::error::RuntimeError;

pub async fn send_request(vm_name: &str, url: &str, body: &str) -> Result<(), RuntimeError> {
    let vm_path = format!("{}/{}.socket", TMP_DIR, vm_name);
    let url = format!("http://localhost/{}", url);

    let mut client = unix_client::get_client(&vm_path).await.unwrap();

    let req = Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(body.to_owned())?;

    let res = client.send_request(req).await?;

    info!("{}, {}", vm_path, res.status());

    Ok(())
}
