use jsonwebtoken::errors::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}
pub fn encode_jwt(secret: &str, email: String) -> Result<String, Error> {
    let now = chrono::Utc::now();
    let expires_in = chrono::Duration::hours(24);
    let exp = now + expires_in;

    let claims = Claims {
        iat: now.timestamp(),
        exp: exp.timestamp(),
        email,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_jwt(secret: &str, token: &str) -> Result<TokenData<Claims>, Error> {
    let token_data = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data)
}

// public export
pub use jsonwebtoken::*;
