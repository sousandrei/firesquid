use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub socket_path: PathBuf,
    pub firecracker_path: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            data_dir: path_from_env("FIRESQUID_DATA_DIR", "/var/lib/firesquid"),
            log_dir: path_from_env("FIRESQUID_LOG_DIR", "/var/log/firesquid"),
            runtime_dir: path_from_env("FIRESQUID_RUNTIME_DIR", "/run/firesquid"),
            cache_dir: path_from_env("FIRESQUID_CACHE_DIR", "/var/cache/firesquid"),
            socket_path: path_from_env("FIRESQUID_SOCKET", "/run/firesquid/firesquid.sock"),
            firecracker_path: path_from_env("FIRESQUID_FIRECRACKER", "firecracker"),
        }
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("firesquid.db")
    }

    pub async fn prepare_directories(&self) -> std::io::Result<()> {
        for path in [
            &self.data_dir,
            &self.log_dir,
            &self.runtime_dir,
            &self.cache_dir,
        ] {
            tokio::fs::create_dir_all(path).await?;
        }

        if let Some(parent) = self.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Ok(())
    }

    pub fn vm_runtime_dir(&self, id: &str) -> PathBuf {
        self.runtime_dir.join(id)
    }

    pub fn vm_socket(&self, id: &str) -> PathBuf {
        self.vm_runtime_dir(id).join("firecracker.sock")
    }

    pub fn vm_log(&self, id: &str) -> PathBuf {
        self.log_dir.join(format!("{id}.log"))
    }
}

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}
