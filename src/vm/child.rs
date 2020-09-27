use std::process::Stdio;
use tokio::process::Command;

use crate::error::RuntimeError;
use crate::vm::http;
use crate::State;

pub async fn spawn_process(
    vm_name: &str,
    state: State,
) -> Result<tokio::process::Child, RuntimeError> {
    let child = Command::new(format!("{}/firecracker", state.assets_dir))
        .args(&["--api-sock", &format!("./tmp/{}.socket", vm_name)])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    set_kernel(vm_name, &state.assets_dir).await?;
    println!("kernel");

    set_drive(vm_name, &state.tmp_dir).await?;
    println!("drive");

    start_machine(vm_name).await?;
    println!("start");

    Ok(child)
}

pub async fn set_kernel(vm_name: &str, assets_dir: &str) -> Result<(), RuntimeError> {
    let url = "/boot-source";

    let body = serde_json::json!({
        "kernel_image_path": format!("{}/vmlinux", assets_dir),
        "boot_args": "console=ttyS0 reboot=k panic=1 pci=off"
    });

    println!("{}", url);
    println!("{}", body);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn set_drive(vm_name: &str, tmp_dir: &str) -> Result<(), RuntimeError> {
    let url = "/drives/rootfs";

    let body = serde_json::json!({
        "drive_id": "rootfs",
        "path_on_host": format!("{}/{}.ext4", tmp_dir, vm_name),
        "is_root_device": true,
        "is_read_only": false
    });

    println!("{}", url);
    println!("{}", body);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn start_machine(vm_name: &str) -> Result<(), RuntimeError> {
    let url = "/actions";
    let body = serde_json::json!({
        "action_type": "InstanceStart",
    });

    println!("{}", url);
    println!("{}", body);

    http::send_request(vm_name, url, &body.to_string()).await
}

pub async fn stop_machine(vm_name: &str) -> Result<(), RuntimeError> {
    let url = "/actions";
    let body = serde_json::json!({
        "action_type": "SendCtrlAltDel",
    });

    println!("{}", url);
    println!("{}", body);

    http::send_request(vm_name, url, &body.to_string()).await
}
