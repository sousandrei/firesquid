use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn create_drive(drive_name: &str, tmp_path: &str) -> super::IoError {
    let drive_dist = format!("{}/rootfs.ext4", super::DEFAULT_ASSETS_PATH);
    let drive_path = format!("{}/{}.ext4", tmp_path, drive_name);
    let mut path = PathBuf::from(drive_path);

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
            format!("Drive file already exists [{}]", path.display()),
        ));
    }

    match fs::copy(drive_dist, &path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn delete_drive(drive_name: &str, tmp_path: &str) -> super::IoError {
    let drive_path = format!("{}/{}.ext4", tmp_path, drive_name);
    let mut path = PathBuf::from(drive_path);

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
            format!("Drive file does not exist [{}]", path.display()),
        ));
    }

    fs::remove_file(&path)
}

#[test]
fn create_drive_works() {
    const DRIVE_NAME: &str = "./vm1";
    const DRIVE_FOLDER: &str = "./tmp_drive1";

    let _ = fs::create_dir(DRIVE_FOLDER);
    let _ = create_drive(DRIVE_NAME, DRIVE_FOLDER);

    let drive_path = format!("{}/{}.ext4", DRIVE_FOLDER, DRIVE_NAME);
    let path = PathBuf::from(drive_path);

    assert_eq!(path.exists(), true);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(DRIVE_FOLDER);
}

#[test]
fn delete_drive_works() {
    const DRIVE_NAME: &str = "./vm2";
    const DRIVE_FOLDER: &str = "./tmp_drive2";

    let _ = fs::create_dir(DRIVE_FOLDER);

    let drive_path = format!("{}/{}.ext4", DRIVE_FOLDER, DRIVE_NAME);
    fs::File::create(&drive_path).expect("Erro creating drive test file");

    let _ = delete_drive(DRIVE_NAME, DRIVE_FOLDER);
    let path = PathBuf::from(drive_path);

    assert_eq!(path.exists(), false);

    let _ = fs::remove_dir(DRIVE_FOLDER);
}
