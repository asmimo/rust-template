#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("Environment variable not found: '{key}'")]
    NotFound { key: String },

    #[error("Environment variable not valid UTF-8: '{key}'")]
    InvalidUtf8 { key: String },

    #[error("Failed to parse environment variable '{key}' with value '{value}': {error}")]
    ParseFailed {
        key: String,
        value: String,
        error: String,
    },
}
