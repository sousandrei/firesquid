use std::io::Error;
use std::path::PathBuf;

use crate::io;

pub fn create_drive(drive_name: &str, tmp_path: &str, assets_dir: &str) -> Result<(), Error> {
    let drive_src = PathBuf::from(format!("{}/rootfs.ext4", assets_dir));
    let drive_dest = PathBuf::from(format!("{}/{}.ext4", tmp_path, drive_name));

    io::copy_file(drive_src, drive_dest)
}

pub fn delete_drive(drive_name: &str, tmp_path: &str) -> Result<(), Error> {
    let drive_dest = PathBuf::from(format!("{}/{}.ext4", tmp_path, drive_name));
    io::delete_file(drive_dest)
}