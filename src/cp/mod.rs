extern crate hyper;
extern crate hyperlocal;
extern crate tokio;

use hyper::{Body, Client, Method, Request};
use hyperlocal::{UnixClientExt, Uri};
use std::path::Path;
use std::process::Command;

pub fn spawn(vm_name: &str) {
    let mut child = Command::new("./assets/firecracker")
        .args(&["--api-sock", &format!("./tmp/{}.socket", vm_name)])
        .spawn()
        .expect("eita");

    set_kernel(vm_name).expect(&format!("failing setting kernel for {}", vm_name));
    set_drive(vm_name).expect(&format!("failing setting drive for {}", vm_name));
    start_machine(vm_name).expect(&format!("failing booting {}", vm_name));

    // child.kill().expect("cannot kill vm");
    child
        .wait()
        .expect(&format!("error waiting for {}", vm_name));
}

#[tokio::main]
async fn set_kernel(vm_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "/boot-source";
    let body = &format!("{{\"kernel_image_path\":\"./tmp/{}.vmlinux\",\"boot_args\":\"console=ttyS0 reboot=k panic=1 pci=off\"}}",vm_name);

    make_request(vm_name, url, body).await
}

#[tokio::main]
async fn set_drive(vm_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "/drives/rootfs";
    let body = &format!("{{\"drive_id\":\"rootfs\",\"path_on_host\":\"./tmp/{}.ext4\",\"is_root_device\":true,\"is_read_only\":false}}",vm_name);

    make_request(vm_name, url, body).await
}

#[tokio::main]
async fn start_machine(vm_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "/actions";
    let body = "{\"action_type\":\"InstanceStart\"}";

    make_request(vm_name, url, body).await
}

async fn make_request(
    vm_name: &str,
    url: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let vm_path = format!("./tmp/{}.socket", vm_name);
    let path = Path::new(&vm_path);
    let url: Uri = Uri::new(path, url).into();

    println!("{:?}", url);

    let client = Client::unix();

    let req = Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("Accept", "Accept: application/json")
        .header("Content-Type", "Accept: application/json")
        .body(Body::from(body.to_owned()))?;

    let resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    Ok(())
}
