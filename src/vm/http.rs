use hyper::{Body, Client, Method, Request};
use hyperlocal::{UnixClientExt, Uri};
use std::path::Path;

use crate::error::RuntimeError;

//TODO: clean up
pub async fn send_request(vm_name: &str, url: &str, body: &str) -> Result<(), RuntimeError> {
    let vm_path = format!("./tmp/{}.socket", vm_name);
    let path = Path::new(&vm_path);
    let url: Uri = Uri::new(path, url).into();

    let client = Client::unix();

    let req = match Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("Accept", "Accept: application/json")
        .header("Content-Type", "Accept: application/json")
        .body(Body::from(body.to_owned()))
    {
        Ok(req) => req,
        Err(_) => return Err(RuntimeError::new("error making request")),
    };

    let res = match client.request(req).await {
        Ok(res) => res,
        Err(_) => return Err(RuntimeError::new("error getting response")),
    };

    println!("Response: {}", res.status());

    Ok(())
}
