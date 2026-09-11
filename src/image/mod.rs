//! Local Docker/OCI image import and ext4 rootfs creation.

use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};

use flate2::read::GzDecoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tar::Archive;

use crate::{config::Config, error::Error};

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

pub fn import(config: &Config, archive: &Path) -> Result<PathBuf, Error> {
    require_file(archive, "image archive")?;
    let file = File::open(archive)?;
    let mut tar = Archive::new(file);
    let mut manifest = None;
    for entry in tar.entries()? {
        let mut entry = entry?;
        if entry.path()?.as_ref() == Path::new("manifest.json") {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut entry, &mut contents)?;
            manifest = Some(contents);
            break;
        }
    }
    let manifests: Vec<Manifest> =
        serde_json::from_str(&manifest.ok_or_else(|| {
            Error::InvalidRequest("Docker archive lacks manifest.json".to_owned())
        })?)
        .map_err(|error| Error::InvalidRequest(format!("invalid Docker manifest: {error}")))?;
    if manifests.len() != 1 || manifests[0].layers.is_empty() {
        return Err(Error::InvalidRequest(
            "Docker archive must contain one image with layers".to_owned(),
        ));
    }

    let digest = sha256_file(archive)?;
    let directory = config.cache_dir.join("images");
    std::fs::create_dir_all(&directory)?;
    let destination = directory.join(format!("{digest}.tar"));
    std::fs::copy(archive, &destination)?;
    Ok(destination)
}

pub fn prepare(reference: &str, output: &Path) -> Result<(), Error> {
    if reference.trim().is_empty() {
        return Err(Error::InvalidRequest(
            "image reference cannot be empty".to_owned(),
        ));
    }
    if output.exists() {
        return Err(Error::InvalidRequest(format!(
            "image archive already exists: {}",
            output.display()
        )));
    }
    let pull = Command::new("docker")
        .args(["pull", reference])
        .status()
        .map_err(|error| Error::Runtime(format!("failed to run docker pull: {error}")))?;
    if !pull.success() {
        return Err(Error::Runtime(format!("docker pull failed with {pull}")));
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let save = Command::new("docker")
        .args(["save", reference, "-o"])
        .arg(output)
        .status()
        .map_err(|error| Error::Runtime(format!("failed to run docker save: {error}")))?;
    if !save.success() {
        return Err(Error::Runtime(format!("docker save failed with {save}")));
    }
    Ok(())
}

pub fn build(config: &Config, archive: &Path, output: &Path, size_mib: u32) -> Result<(), Error> {
    require_file(archive, "image archive")?;
    if size_mib == 0 {
        return Err(Error::InvalidRequest(
            "rootfs size must be greater than zero".to_owned(),
        ));
    }
    if output.exists() {
        return Err(Error::InvalidRequest(format!(
            "rootfs output already exists: {}",
            output.display()
        )));
    }
    let staging = config.cache_dir.join("rootfs-staging");
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    flatten_archive(archive, &staging)?;
    install_init(&staging)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let status = Command::new("mke2fs")
        .args(["-q", "-t", "ext4", "-L", "firesquid-rootfs", "-d"])
        .arg(&staging)
        .arg(output)
        .arg(format!("{size_mib}M"))
        .status()
        .map_err(|error| Error::Runtime(format!("failed to run mke2fs: {error}")))?;
    if !status.success() {
        return Err(Error::Runtime(format!("mke2fs failed with {status}")));
    }
    std::fs::remove_dir_all(staging)?;
    Ok(())
}

fn flatten_archive(archive: &Path, staging: &Path) -> Result<(), Error> {
    let file = File::open(archive)?;
    let mut outer = Archive::new(file);
    let mut manifest = None;
    for entry in outer.entries()? {
        let mut entry = entry?;
        if entry.path()?.as_ref() == Path::new("manifest.json") {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            manifest = Some(contents);
            break;
        }
    }
    let manifests: Vec<Manifest> =
        serde_json::from_str(&manifest.ok_or_else(|| {
            Error::InvalidRequest("Docker archive lacks manifest.json".to_owned())
        })?)
        .map_err(|error| Error::InvalidRequest(format!("invalid Docker manifest: {error}")))?;
    let layer_paths = manifests
        .first()
        .map(|manifest| manifest.layers.clone())
        .unwrap_or_default();
    let mut layers = Vec::new();
    let file = File::open(archive)?;
    let mut outer = Archive::new(file);
    for entry in outer.entries()? {
        let mut entry = entry?;
        if layer_paths
            .iter()
            .any(|layer| entry.path().is_ok_and(|path| path == Path::new(layer)))
        {
            let mut layer = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut layer)?;
            layers.push(layer);
        }
    }
    if layers.is_empty() {
        return Err(Error::InvalidRequest(
            "Docker archive contains no layers".to_owned(),
        ));
    }
    for layer in layers {
        if layer.starts_with(&[0x1f, 0x8b]) {
            unpack_layer(GzDecoder::new(layer.as_slice()), staging)?;
        } else {
            unpack_layer(layer.as_slice(), staging)?;
        }
    }
    Ok(())
}

fn unpack_layer<R: Read>(reader: R, staging: &Path) -> Result<(), Error> {
    let mut archive = Archive::new(reader);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let relative = safe_relative(entry.path()?.as_ref())?;
        apply_whiteout(staging, &relative)?;
        if relative
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".wh."))
        {
            continue;
        }
        entry.unpack_in(staging)?;
    }
    Ok(())
}

fn apply_whiteout(staging: &Path, relative: &Path) -> Result<(), Error> {
    let Some(name) = relative.file_name().map(|name| name.to_string_lossy()) else {
        return Ok(());
    };
    if !name.starts_with(".wh.") {
        return Ok(());
    }
    let parent = relative.parent().unwrap_or(Path::new(""));
    if name == ".wh..wh..opq" {
        let directory = staging.join(parent);
        if directory.is_dir() {
            for entry in std::fs::read_dir(directory)? {
                std::fs::remove_dir_all(entry?.path())?;
            }
        }
    } else {
        let target = staging.join(parent).join(&name[4..]);
        if target.is_dir() {
            std::fs::remove_dir_all(target)?;
        } else if target.exists() {
            std::fs::remove_file(target)?;
        }
    }
    Ok(())
}

fn safe_relative(path: &Path) -> Result<PathBuf, Error> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Error::InvalidRequest(format!(
            "unsafe archive path: {}",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn install_init(staging: &Path) -> Result<(), Error> {
    let init = staging.join("sbin/init");
    std::fs::create_dir_all(init.parent().expect("static init parent"))?;
    std::fs::write(
        &init,
        b"#!/bin/sh\nmount -t devtmpfs dev /dev\nmount -t proc proc /proc\nmount -t sysfs sysfs /sys\nexec /bin/sh\n",
    )?;
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(&init)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(init, permissions)?;
    Ok(())
}

fn require_file(path: &Path, label: &str) -> Result<(), Error> {
    if path.is_file() {
        Ok(())
    } else {
        Err(Error::InvalidRequest(format!(
            "{label} does not exist: {}",
            path.display()
        )))
    }
}

fn sha256_file(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::safe_relative;

    #[test]
    fn rejects_archive_path_traversal() {
        assert!(safe_relative(std::path::Path::new("../../etc/passwd")).is_err());
    }
}
