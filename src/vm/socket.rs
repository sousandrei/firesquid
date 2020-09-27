use std::io::Error;
use std::path::PathBuf;

use crate::io;

pub fn delete_socket(socket_name: &str, tmp_path: &str) -> Result<(), Error> {
    let socket_path = PathBuf::from(format!("{}/{}.socket", tmp_path, socket_name));
    io::delete_file(socket_path)
}
