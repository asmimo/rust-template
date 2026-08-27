use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor, message::Message,
    transport::smtp::authentication::Credentials,
};

mod builder;
mod config;
mod error;

pub use builder::*;
pub use config::*;
pub use error::*;

#[derive(Debug, Clone)]
pub struct Lettre(Arc<AsyncSmtpTransport<Tokio1Executor>>);

impl Lettre {
    pub fn init() -> Result<Self, LettreError> {
        let config = EmailConfig::from_env()?;
        let transport = Self::init_with_config(config)?;

        Ok(Self(Arc::new(transport)))
    }

    pub fn init_with_config(
        config: EmailConfig,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, LettreError> {
        let creds = Credentials::new(config.username, config.password);

        let transport = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        };

        let transport = transport.credentials(creds);

        let transport = if let Some(port) = config.smtp_port {
            transport.port(port)
        } else {
            transport
        };

        Ok(transport.build())
    }

    #[tracing::instrument(skip(self, message))]
    pub async fn send(&self, message: Message) -> Result<(), LettreError> {
        self.0.send(message).await?;
        Ok(())
    }
}
