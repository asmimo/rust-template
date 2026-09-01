use crate::env;

#[derive(Debug, thiserror::Error)]
pub enum MaxmindDbError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Env(#[from] env::EnvError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    MaxMindDB(#[from] maxminddb::MaxMindDbError),

    #[error(transparent)]
    ParseError(#[from] chrono_tz::ParseError),

    #[error("Custom Error: {0}")]
    Custom(String),
}

pub type MaxmindDbResult<T> = std::result::Result<T, MaxmindDbError>;
