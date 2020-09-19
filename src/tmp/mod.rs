use std::io::Error;

pub mod folder;

pub type IoError = Result<(), Error>;
pub const DEFAULT_PATH: &str = "./tmp";

pub fn initialize_tmp(path: &str) -> IoError {
    let path = match path.len() {
        0 => DEFAULT_PATH,
        _ => path,
    };
    folder::create_tmp_folder(path)
}
