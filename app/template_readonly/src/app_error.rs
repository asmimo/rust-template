use axum::{http, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Env(#[from] utils::env::EnvError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),

    #[error(transparent)]
    Lettre(#[from] utils::lettre::LettreError),

    #[error(transparent)]
    MaxMindDB(#[from] utils::maxminddb::error::MaxmindDbError),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message): (http::StatusCode, Option<String>) = match self {
            AppError::Env(err) => {
                tracing::error!("Environment error: {err}");
                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::Io(err) => {
                tracing::error!("IO error: {err}");
                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::SerdeJson(err) => {
                tracing::error!("De/Serialization error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::Sqlx(err) => {
                tracing::error!("SQLx error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::HttpClient(err) => {
                tracing::error!("Reqwest error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::Lettre(err) => {
                tracing::error!("Lettre error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::MaxMindDB(err) => {
                tracing::error!("MaxMindDB error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::Join(err) => {
                tracing::error!("Tokio join error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::Anyhow(err) => {
                tracing::error!("Unhandled error: {err}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {msg}");

                (http::StatusCode::INTERNAL_SERVER_ERROR, Some(msg))
            }
        };

        let message = message.unwrap_or("An error has occurred".to_string());

        (status, message).into_response()
    }
}
