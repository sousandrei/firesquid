use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl From<postcard::Error> for Error {
    fn from(error: postcard::Error) -> Self {
        Self::Protocol(error.to_string())
    }
}
