use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use anyhow::Result;
use sha2::{Sha256, Digest};
use hex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub user_id: String,
}

pub struct JwtConfig {
    pub secret: String,
}

impl JwtConfig {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: String, expires_in_hours: i64) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        let claims = Claims {
            sub: user_id.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            user_id,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(data.claims)
    }
}

pub fn hash_password(password: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let computed_hash = hash_password(password)?;
    Ok(computed_hash == hash)
}
