use std::{path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::{config::Config, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VmStatus {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl TryFrom<&str> for VmStatus {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created" => Ok(Self::Created),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "stopping" => Ok(Self::Stopping),
            "stopped" => Ok(Self::Stopped),
            "failed" => Ok(Self::Failed),
            _ => Err(Error::Protocol(format!("unknown VM status: {value}"))),
        }
    }
}

impl std::fmt::Display for VmStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Created => "created",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vm {
    pub id: String,
    pub name: String,
    pub status: VmStatus,
    pub last_error: Option<String>,
    pub kernel_path: String,
    pub rootfs_path: String,
    pub vcpus: u32,
    pub memory_mib: u32,
    pub pid: Option<u32>,
}

#[derive(Clone)]
pub struct State {
    pool: SqlitePool,
    pub config: Config,
}

pub type SharedState = Arc<State>;

impl State {
    pub async fn open(path: &Path, config: Config) -> Result<Self, Error> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let state = Self { pool, config };
        sqlx::migrate!("./migrations").run(&state.pool).await?;
        Ok(state)
    }

    pub async fn reconcile(&self) -> Result<(), Error> {
        sqlx::query_file!("src/queries/vm_reconcile.sql")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_vms(&self) -> Result<Vec<Vm>, Error> {
        let rows = sqlx::query_file_as!(VmRow, "src/queries/vm_list.sql")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Vm::try_from).collect()
    }

    pub async fn get_vm(&self, id: &str) -> Result<Option<Vm>, Error> {
        let row = sqlx::query_file_as!(VmRow, "src/queries/vm_get.sql", id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Vm::try_from).transpose()
    }

    pub async fn delete_vm(&self, id: &str) -> Result<bool, Error> {
        let result = sqlx::query_file!("src/queries/vm_delete.sql", id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    #[cfg(test)]
    pub async fn create_vm(&self, name: &str) -> Result<Vm, Error> {
        self.create_vm_with_spec(&VmSpec {
            name: name.to_owned(),
            kernel_path: String::new(),
            rootfs_path: String::new(),
            vcpus: 1,
            memory_mib: 512,
        })
        .await
    }

    pub async fn create_vm_with_spec(&self, spec: &VmSpec) -> Result<Vm, Error> {
        validate_name(&spec.name)?;
        sqlx::query_file!(
            "src/queries/vm_create.sql",
            &spec.name,
            &spec.name,
            &spec.kernel_path,
            &spec.rootfs_path,
            i64::from(spec.vcpus),
            i64::from(spec.memory_mib)
        )
        .execute(&self.pool)
        .await
        .map_err(|error| {
            if let sqlx::Error::Database(database_error) = &error
                && database_error.is_unique_violation()
            {
                return Error::InvalidRequest(format!("VM already exists: {}", spec.name));
            }
            Error::Database(error)
        })?;

        self.get_vm(&spec.name)
            .await?
            .ok_or_else(|| Error::Protocol("created VM was not found".to_owned()))
    }

    pub async fn begin_start(&self, id: &str) -> Result<Vm, Error> {
        let vm = self
            .get_vm(id)
            .await?
            .ok_or_else(|| Error::InvalidRequest(format!("VM not found: {id}")))?;
        if matches!(
            vm.status,
            VmStatus::Starting | VmStatus::Running | VmStatus::Stopping
        ) {
            return Err(Error::InvalidRequest(format!("VM is already active: {id}")));
        }
        sqlx::query_file!("src/queries/vm_start.sql", id)
            .execute(&self.pool)
            .await?;
        self.get_vm(id)
            .await?
            .ok_or_else(|| Error::Protocol("started VM was not found".to_owned()))
    }

    pub async fn mark_running(&self, id: &str, pid: u32) -> Result<(), Error> {
        sqlx::query_file!("src/queries/vm_mark_running.sql", i64::from(pid), id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_stopping(&self, id: &str) -> Result<(), Error> {
        sqlx::query_file!("src/queries/vm_mark_stopping.sql", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_stopped(&self, id: &str) -> Result<(), Error> {
        sqlx::query_file!("src/queries/vm_mark_stopped.sql", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: &str, message: &str) -> Result<(), Error> {
        sqlx::query_file!("src/queries/vm_mark_failed.sql", message, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VmSpec {
    pub name: String,
    pub kernel_path: String,
    pub rootfs_path: String,
    pub vcpus: u32,
    pub memory_mib: u32,
}

#[derive(Debug)]
struct VmRow {
    id: String,
    name: String,
    status: String,
    last_error: Option<String>,
    kernel_path: String,
    rootfs_path: String,
    vcpus: i64,
    memory_mib: i64,
    pid: Option<i64>,
}

impl TryFrom<VmRow> for Vm {
    type Error = Error;

    fn try_from(row: VmRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            status: VmStatus::try_from(row.status.as_str())?,
            last_error: row.last_error,
            kernel_path: row.kernel_path,
            rootfs_path: row.rootfs_path,
            vcpus: u32::try_from(row.vcpus)
                .map_err(|_| Error::Protocol("invalid vCPU count in state".to_owned()))?,
            memory_mib: u32::try_from(row.memory_mib)
                .map_err(|_| Error::Protocol("invalid memory size in state".to_owned()))?,
            pid: row
                .pid
                .map(u32::try_from)
                .transpose()
                .map_err(|_| Error::Protocol("invalid process ID in state".to_owned()))?,
        })
    }
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || name.len() > 63
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.chars().any(char::is_whitespace)
    {
        return Err(Error::InvalidRequest(
            "VM name must be 1-63 non-whitespace characters without path separators".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{State, VmStatus};
    use crate::config::Config;

    static NEXT_TEST_DB: AtomicU64 = AtomicU64::new(0);

    async fn test_state() -> State {
        let test_id = NEXT_TEST_DB.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "firesquid-state-{}-{}-{}.db",
            std::process::id(),
            test_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before Unix epoch")
                .as_nanos()
        ));
        State::open(&path, Config::from_env())
            .await
            .expect("state should open")
    }

    #[tokio::test]
    async fn creates_and_persists_vm_records() {
        let state = test_state().await;
        let vm = state.create_vm("demo").await.expect("VM should create");

        assert_eq!(vm.id, "demo");
        assert_eq!(vm.status, VmStatus::Created);
        assert_eq!(state.list_vms().await.unwrap(), vec![vm]);
    }

    #[tokio::test]
    async fn rejects_duplicate_and_invalid_names() {
        let state = test_state().await;
        state.create_vm("demo").await.unwrap();

        assert!(state.create_vm("demo").await.is_err());
        assert!(state.create_vm("bad/name").await.is_err());
    }

    #[tokio::test]
    async fn reconciliation_fails_runtime_states() {
        let state = test_state().await;
        sqlx::query_file!("src/queries/vm_insert_running.test.sql")
            .execute(&state.pool)
            .await
            .unwrap();

        state.reconcile().await.unwrap();
        let vm = state.get_vm("demo").await.unwrap().unwrap();
        assert_eq!(vm.status, VmStatus::Failed);
    }
}
