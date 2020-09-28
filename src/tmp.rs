use std::io::ErrorKind;
use std::path::PathBuf;
use tracing::error;

use crate::io;

//TODO: better error handling
pub fn init(path: &str) -> Result<(), std::io::Error> {
    let path = PathBuf::from(path);

    match io::create_folder(path) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => Ok(()),
            _ => {
                error!("{:?}", e.kind());
                Err(e)
            }
        },
    }
}
