use std::io::Error;
use std::path::PathBuf;

use crate::consts::TMP_DIR;
use crate::io;

pub fn delete_socket(socket_name: &str) -> Result<(), Error> {
    let socket_path = PathBuf::from(format!("{}/{}.socket", TMP_DIR, socket_name));
    io::delete_file(socket_path)
}
