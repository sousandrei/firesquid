use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn create_tmp_folder(user_path: &str) -> super::IoError {
    let mut path = PathBuf::from(user_path);

    path = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {}
                _ => panic!("Error resolving tmp path: {}", e),
            }

            fs::create_dir(&path).expect(&format!("Cannot create dir [{}]", path.display()));
            fs::canonicalize(path).unwrap()
        }
    };

    if !path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Path is not a directory",
        ));
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            if entries.peekable().peek().is_some() {
                println!("tmp dir not empty");
            }
        }
        Err(e) => println!("Cannot read dir: {}", e),
    };

    Ok(())
}

#[test]
fn create_tmp_folder_works_custom_path() {
    const PATH: &str = "./tmp1";

    let _ = create_tmp_folder(PATH);
    let path = PathBuf::from(PATH);

    assert_eq!(path.is_dir(), true);

    let _ = fs::remove_dir(PATH);
}

#[test]
fn create_tmp_folder_works_empty() {
    let _ = create_tmp_folder(super::DEFAULT_TMP_PATH);
    let path = PathBuf::from(super::DEFAULT_TMP_PATH);

    assert_eq!(path.is_dir(), true);

    let _ = fs::remove_dir(super::DEFAULT_TMP_PATH);
}

#[test]
#[should_panic]
fn create_tmp_folder_invalid_path() {
    let _ = create_tmp_folder("");
}
