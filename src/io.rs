use std::io::{Error, ErrorKind};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{error, info, warn};

pub fn create_folder(path: PathBuf) -> Result<(), std::io::Error> {
    validate_path(&path)?;

    if check_exists(&path).is_ok() {
        info!("Path create successfully [{}]", path.display());
        fs::create_dir_all(path)?;
        return Ok(());
    }

    if !path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Path is not a directory",
        ));
    }

    info!("Using folder [{}]", path.display());

    match fs::read_dir(&path) {
        Ok(entries) => {
            if entries.peekable().peek().is_some() {
                warn!("Warning: dir not empty [{}]", path.display());
            }
        }
        Err(e) => error!("Cannot read dir [{}]", e),
    };

    Ok(())
}

pub fn copy_file(src: PathBuf, dest: PathBuf) -> Result<(), std::io::Error> {
    validate_path(&src)?;
    check_exists(&src).err();

    validate_path(&dest)?;
    check_exists(&dest)?;

    match fs::copy(src, dest) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn delete_file(path: PathBuf) -> Result<(), std::io::Error> {
    validate_path(&path)?;

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path)?;

    Ok(())
}

fn validate_path(path: &Path) -> Result<(), std::io::Error> {
    match fs::canonicalize(&path) {
        Ok(p) => &p,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => path,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Error validating path [{}]", path.display()),
                ))
            }
        },
    };

    Ok(())
}

fn check_exists(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("File already exists [{}]", path.display()),
        ));
    }

    Ok(())
}

//TODO: figure out testing
// #[cfg(test)]
// mod test {
//     #[test]
//     fn create_file_works() {
//         const DRIVE_NAME: &str = "./vm1";
//         const DRIVE_FOLDER: &str = "./tmp_file1";

//         let _ = fs::create_dir(DRIVE_FOLDER);
//         let _ = create_file(DRIVE_NAME, DRIVE_FOLDER);

//         let drive_path = format!("{}/{}.ext4", DRIVE_FOLDER, DRIVE_NAME);
//         let path = PathBuf::from(drive_path);

//         assert_eq!(path.exists(), true);

//         let _ = fs::remove_file(&path);
//         let _ = fs::remove_dir(DRIVE_FOLDER);
//     }

//     #[test]
//     fn delete_file_works() {
//         const DRIVE_NAME: &str = "./vm2";
//         const DRIVE_FOLDER: &str = "./tmp_file2";

//         let _ = fs::create_dir(DRIVE_FOLDER);

//         let drive_path = format!("{}/{}.ext4", DRIVE_FOLDER, DRIVE_NAME);
//         fs::File::create(&drive_path).expect("Erro creating drive test file");

//         let _ = delete_file(DRIVE_NAME, DRIVE_FOLDER);
//         let path = PathBuf::from(drive_path);

//         assert_eq!(path.exists(), false);

//         let _ = fs::remove_dir(DRIVE_FOLDER);
//     }
// }
