use std::io::Error;

pub mod drive;
pub mod folder;
pub mod kernel;
pub mod socket;

pub type IoError = Result<(), Error>;
pub const DEFAULT_TMP_PATH: &str = "./tmp";
pub const DEFAULT_ASSETS_PATH: &str = "./assets";

pub fn initialize_tmp(path: &str) -> IoError {
    let path = match path.len() {
        0 => DEFAULT_TMP_PATH,
        _ => path,
    };
    folder::create_tmp_folder(path)
}

pub fn initialize_vm(vm_name: &str) -> IoError {
    kernel::create_kernel(vm_name, DEFAULT_TMP_PATH)?;
    drive::create_drive(vm_name, DEFAULT_TMP_PATH)
}

pub fn terminate_vm(vm_name: &str) -> IoError {
    socket::delete_socket(vm_name, DEFAULT_TMP_PATH)?;
    kernel::delete_kernel(vm_name, DEFAULT_TMP_PATH)?;
    drive::delete_drive(vm_name, DEFAULT_TMP_PATH)
}
