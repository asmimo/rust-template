use crate::env;

#[derive(Debug, thiserror::Error)]
pub enum MaxmindDbError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Optional Error: {0}")]
    Optional(String),

    #[error(transparent)]
    Env(#[from] env::EnvError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    MaxMindDB(#[from] maxminddb::MaxMindDbError),
}

pub type MaxmindDbResult<T> = std::result::Result<T, MaxmindDbError>;
