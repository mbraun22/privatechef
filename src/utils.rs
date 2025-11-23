use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::JwtConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: String, expiration_seconds: i64) -> Self {
        let now = Utc::now();
        Claims {
            sub: user_id,
            exp: (now + Duration::seconds(expiration_seconds)).timestamp(),
            iat: now.timestamp(),
        }
    }
}

pub fn create_token(user_id: String, config: &JwtConfig) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, config.expiration);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
}

pub fn create_refresh_token(user_id: String, config: &JwtConfig) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, config.refresh_expiration);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

