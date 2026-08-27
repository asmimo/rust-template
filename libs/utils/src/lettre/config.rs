use crate::env;
use crate::lettre::LettreError;

#[must_use]
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: Option<u16>,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl EmailConfig {
    pub fn new(smtp_host: String, username: String, password: String) -> Self {
        Self {
            smtp_host,
            smtp_port: None,
            username,
            password,
            use_tls: true,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.smtp_port = Some(port);
        self
    }

    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    pub fn from_env() -> Result<Self, LettreError> {
        Ok(Self {
            smtp_host: env::get_env("SMTP_HOST")?,
            smtp_port: env::get_env("SMTP_PORT")
                .ok()
                .and_then(|port| port.parse().ok()),
            username: env::get_env("SMTP_USERNAME")?,
            password: env::get_env("SMTP_PASSWORD")?,
            use_tls: env::get_env_or_default("SMTP_USE_TLS", "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}
