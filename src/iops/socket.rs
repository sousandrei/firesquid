use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn create_socket(socket_name: &str, tmp_path: &str) -> super::IoError {
    let socket_path = format!("{}/{}.socket", tmp_path, socket_name);
    let mut path = PathBuf::from(socket_path);

    path = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => path,
            _ => panic!("Error resolving tmp path [{}]", e),
        },
    };

    if path.exists() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Socket file already exists [{}]", path.display()),
        ));
    }

    match fs::File::create(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn delete_socket(socket_name: &str, tmp_path: &str) -> super::IoError {
    let socket_path = format!("{}/{}.socket", tmp_path, socket_name);
    let mut path = PathBuf::from(socket_path);

    path = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => path,
            _ => panic!("Error resolving tmp path [{}]", e),
        },
    };

    if !path.exists() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Socket file does not exist [{}]", path.display()),
        ));
    }

    fs::remove_file(&path)
}

#[test]
fn create_socket_works() {
    const SOCKET_NAME: &str = "./vm2";
    const SOCKET_FOLDER: &str = "./tmp_socket";

    let _ = fs::create_dir(SOCKET_FOLDER);
    let _ = create_socket(SOCKET_NAME, SOCKET_FOLDER);

    let socket_path = format!("{}/{}.socket", SOCKET_FOLDER, SOCKET_NAME);
    let path = PathBuf::from(socket_path);

    assert_eq!(path.exists(), true);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(SOCKET_FOLDER);
}

#[test]
fn delete_socket_works() {
    const SOCKET_NAME: &str = "./vm2";
    const SOCKET_FOLDER: &str = "./tmp_socket";

    let _ = fs::create_dir(SOCKET_FOLDER);

    let socket_path = format!("{}/{}.socket", SOCKET_FOLDER, SOCKET_NAME);
    fs::File::create(&socket_path).expect("Erro creating socket test file");

    let _ = delete_socket(SOCKET_NAME, SOCKET_FOLDER);
    let path = PathBuf::from(socket_path);

    assert_eq!(path.exists(), false);

    let _ = fs::remove_dir(SOCKET_FOLDER);
}
