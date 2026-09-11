use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub socket_path: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            data_dir: path_from_env("FIRESQUID_DATA_DIR", "/var/lib/firesquid"),
            runtime_dir: path_from_env("FIRESQUID_RUNTIME_DIR", "/run/firesquid"),
            cache_dir: path_from_env("FIRESQUID_CACHE_DIR", "/var/cache/firesquid"),
            socket_path: path_from_env("FIRESQUID_SOCKET", "/run/firesquid/firesquid.sock"),
        }
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("firesquid.db")
    }

    pub async fn prepare_directories(&self) -> std::io::Result<()> {
        for path in [&self.data_dir, &self.runtime_dir, &self.cache_dir] {
            tokio::fs::create_dir_all(path).await?;
        }

        if let Some(parent) = self.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Ok(())
    }
}

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}
