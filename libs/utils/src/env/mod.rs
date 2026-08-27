use std::env;

mod error;

pub use error::EnvError;
pub fn get_env(key: &str) -> Result<String, EnvError> {
    env::var(key).map_err(|e| match e {
        env::VarError::NotPresent => EnvError::NotFound {
            key: key.to_string(),
        },
        env::VarError::NotUnicode(_) => EnvError::InvalidUtf8 {
            key: key.to_string(),
        },
    })
}

pub fn get_env_or_default(key: &str, default: String) -> String {
    get_env(key).unwrap_or(default)
}

pub fn get_env_parsed<T>(key: &str) -> Result<T, EnvError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = get_env(key)?;
    value.parse().map_err(|e: T::Err| EnvError::ParseFailed {
        key: key.to_string(),
        value,
        error: e.to_string(),
    })
}
