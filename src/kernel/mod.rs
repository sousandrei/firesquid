//! Local Linux source configuration and kernel compilation.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{config::Config, error::Error};

const DEFAULT_GUEST_CONFIG_URL: &str = "https://raw.githubusercontent.com/firecracker-microvm/firecracker/main/resources/guest_configs/microvm-kernel-ci-x86_64-6.18.config";
const DEFAULT_GUEST_CONFIG_NAME: &str = "microvm-kernel-ci-x86_64-6.18.config";

pub struct BuildOptions {
    pub source: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub repository: String,
    pub tag: String,
    pub jobs: usize,
}

pub fn build(config: &Config, options: BuildOptions) -> Result<PathBuf, Error> {
    let source = match options.source {
        Some(path) => path,
        None => fetch_kernel_source(config, &options.repository, &options.tag)?,
    };
    if options.jobs == 0 {
        return Err(Error::InvalidRequest(
            "kernel build jobs must be greater than zero".to_owned(),
        ));
    }
    require_directory(&source, "Linux source tree")?;
    if !source.join("Makefile").is_file() {
        return Err(Error::InvalidRequest(format!(
            "Linux source tree has no Makefile: {}",
            source.display()
        )));
    }

    let kernel_config = match options.config {
        Some(path) => path,
        None => fetch_default_guest_config(config)?,
    };
    if !kernel_config.is_file() {
        return Err(Error::InvalidRequest(format!(
            "kernel config does not exist: {}",
            kernel_config.display()
        )));
    }
    validate_guest_config(&kernel_config)?;

    let output_dir = config.cache_dir.join("kernel-build");
    std::fs::create_dir_all(&output_dir)?;
    let output_dir = std::fs::canonicalize(output_dir)?;
    for tool in ["make", "flex", "bison"] {
        require_tool(tool)?;
    }
    std::fs::copy(kernel_config, output_dir.join(".config"))?;

    run_make(&source, &output_dir, "olddefconfig", options.jobs)?;
    run_make(&source, &output_dir, "vmlinux", options.jobs)?;

    let output = output_dir.join("vmlinux");
    if !output.is_file() {
        return Err(Error::Runtime(format!(
            "kernel build completed without bzImage: {}",
            output.display()
        )));
    }
    Ok(output)
}

fn run_make(source: &Path, output: &Path, target: &str, jobs: usize) -> Result<(), Error> {
    let status = Command::new("make")
        .arg("-C")
        .arg(source)
        .arg(format!("O={}", output.display()))
        .arg(format!("-j{jobs}"))
        .arg(target)
        .status()
        .map_err(|error| Error::Runtime(format!("failed to run make: {error}")))?;
    if !status.success() {
        return Err(Error::Runtime(format!(
            "make {target} failed with {status}"
        )));
    }
    Ok(())
}

fn validate_guest_config(path: &Path) -> Result<(), Error> {
    let contents = std::fs::read_to_string(path)?;
    for required in [
        "CONFIG_X86_64=y",
        "CONFIG_VIRTIO_BLK=y",
        "CONFIG_VIRTIO_NET=y",
        "CONFIG_EXT4_FS=y",
        "CONFIG_SERIAL_8250_CONSOLE=y",
    ] {
        if !contents.lines().any(|line| line == required) {
            return Err(Error::InvalidRequest(format!(
                "Firecracker guest config is missing {required}"
            )));
        }
    }
    Ok(())
}

fn fetch_default_guest_config(config: &Config) -> Result<PathBuf, Error> {
    let directory = config.cache_dir.join("guest-configs");
    let destination = directory.join(DEFAULT_GUEST_CONFIG_NAME);
    if destination.is_file() {
        return Ok(destination);
    }
    std::fs::create_dir_all(&directory)?;
    let status = Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error"])
        .arg(DEFAULT_GUEST_CONFIG_URL)
        .arg("--output")
        .arg(&destination)
        .status()
        .map_err(|error| {
            Error::Runtime(format!(
                "failed to fetch Firecracker guest config with curl: {error}"
            ))
        })?;
    if !status.success() {
        let _ = std::fs::remove_file(&destination);
        return Err(Error::Runtime(format!(
            "failed to fetch Firecracker guest config: curl exited with {status}"
        )));
    }
    Ok(destination)
}

fn fetch_kernel_source(config: &Config, repository: &str, tag: &str) -> Result<PathBuf, Error> {
    if repository.trim().is_empty() || tag.trim().is_empty() {
        return Err(Error::InvalidRequest(
            "kernel repository and tag cannot be empty".to_owned(),
        ));
    }
    let cache_name = format!("linux-{}", tag.replace(['/', '\\'], "_"));
    let destination = config.cache_dir.join(&cache_name);
    if destination.join("Makefile").is_file() {
        return Ok(destination);
    }
    if destination.exists() {
        return Err(Error::InvalidRequest(format!(
            "cached Linux source is incomplete: {}",
            destination.display()
        )));
    }

    let temporary = config.cache_dir.join(format!("{cache_name}.partial"));
    if temporary.exists() {
        std::fs::remove_dir_all(&temporary)?;
    }
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--single-branch",
            "--branch",
            tag,
            repository,
        ])
        .arg(&temporary)
        .status()
        .map_err(|error| {
            Error::Runtime(format!("failed to clone Linux source with git: {error}"))
        })?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&temporary);
        return Err(Error::Runtime(format!(
            "failed to clone Linux {tag}: git exited with {status}"
        )));
    }
    std::fs::rename(&temporary, &destination)?;
    Ok(destination)
}

fn require_directory(path: &Path, label: &str) -> Result<(), Error> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(Error::InvalidRequest(format!(
            "{label} does not exist: {}",
            path.display()
        )))
    }
}

fn require_tool(tool: &str) -> Result<(), Error> {
    let Some(paths) = std::env::var_os("PATH") else {
        return Err(Error::Runtime(format!(
            "required tool is not available: {tool}"
        )));
    };
    if std::env::split_paths(&paths).any(|path| path.join(tool).is_file()) {
        Ok(())
    } else {
        Err(Error::Runtime(format!(
            "required kernel build tool is missing from PATH: {tool}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{require_directory, validate_guest_config};

    #[test]
    fn rejects_missing_kernel_source() {
        assert!(require_directory(std::path::Path::new("/not/a/kernel"), "source").is_err());
    }

    #[test]
    fn rejects_config_without_firecracker_devices() {
        let path =
            std::env::temp_dir().join(format!("firesquid-kernel-config-{}", std::process::id()));
        std::fs::write(&path, "CONFIG_X86_64=y\n").unwrap();
        assert!(validate_guest_config(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
