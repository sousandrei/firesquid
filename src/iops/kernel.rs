use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn create_kernel(kernel_name: &str, tmp_path: &str) -> super::IoError {
    let kernel_dist = format!("{}/vmlinux", super::DEFAULT_ASSETS_PATH);
    let kernel_path = format!("{}/{}.vmlinux", tmp_path, kernel_name);
    let mut path = PathBuf::from(kernel_path);

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

    match fs::copy(kernel_dist, &path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn delete_kernel(kernel_name: &str, tmp_path: &str) -> super::IoError {
    let kernel_path = format!("{}/{}.vmlinux", tmp_path, kernel_name);
    let mut path = PathBuf::from(kernel_path);

    path = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => path,
            _ => panic!("Error resolving tmp path [{}]", e),
        },
    };

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path)
}

#[test]
fn create_kernel_works() {
    const KERNEL_NAME: &str = "./vm1";
    const KERNEL_FOLDER: &str = "./tmp_kernel1";

    let _ = fs::create_dir(KERNEL_FOLDER);
    let _ = create_kernel(KERNEL_NAME, KERNEL_FOLDER);

    let kernel_path = format!("{}/{}.vmlinux", KERNEL_FOLDER, KERNEL_NAME);
    let path = PathBuf::from(kernel_path);

    assert_eq!(path.exists(), true);

    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(KERNEL_FOLDER);
}

#[test]
fn delete_kernel_works() {
    const KERNEL_NAME: &str = "./vm2";
    const KERNEL_FOLDER: &str = "./tmp_kernel2";

    let _ = fs::create_dir(KERNEL_FOLDER);

    let kernel_path = format!("{}/{}.vmlinux", KERNEL_FOLDER, KERNEL_NAME);
    fs::File::create(&kernel_path).expect("Erro creating kernel test file");

    let _ = delete_kernel(KERNEL_NAME, KERNEL_FOLDER);
    let path = PathBuf::from(kernel_path);

    assert_eq!(path.exists(), false);

    let _ = fs::remove_dir(KERNEL_FOLDER);
}
