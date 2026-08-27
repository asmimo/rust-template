#[derive(Debug, thiserror::Error)]
pub enum LettreError {
    #[error(transparent)]
    MessageBuilder(#[from] lettre::error::Error),

    #[error(transparent)]
    Transport(#[from] lettre::transport::smtp::Error),

    #[error(transparent)]
    InvalidAddress(#[from] lettre::address::AddressError),

    #[error(transparent)]
    Environment(#[from] crate::env::EnvError),
}
