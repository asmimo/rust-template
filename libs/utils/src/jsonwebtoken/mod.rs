use jiff::ToSpan;
use jsonwebtoken::errors::{Error, ErrorKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

/// Encodes an email claim into a JSON Web Token that expires after 24 hours.
pub fn encode_jwt(secret: &str, email: String) -> Result<String, Error> {
    let now = jiff::Timestamp::now();
    let exp = now
        .checked_add(24.hours())
        .map_err(|err| ErrorKind::Provider(err.to_string()))?;

    let claims = Claims {
        iat: now.as_second(),
        exp: exp.as_second(),
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
