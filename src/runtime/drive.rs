use std::io::Error;
use std::path::PathBuf;

use crate::io;

use crate::consts::{ASSETS_DIR, DRIVE_NAME, TMP_DIR};

pub fn create_drive(drive_name: &str) -> Result<(), Error> {
    let drive_src = PathBuf::from(format!("{}/{}.ext4", ASSETS_DIR, DRIVE_NAME));
    let drive_dest = PathBuf::from(format!("{}/{}.ext4", TMP_DIR, drive_name));
    io::copy_file(drive_src, drive_dest)
}

pub fn delete_drive(drive_name: &str) -> Result<(), Error> {
    let drive_dest = PathBuf::from(format!("{}/{}.ext4", TMP_DIR, drive_name));
    io::delete_file(drive_dest)
}
