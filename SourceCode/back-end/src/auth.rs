use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use regex::Regex;
use serde::{Deserialize, Serialize};

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
    #[must_use]
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Generates a JWT token for a user.
    ///
    /// # Errors
    /// Returns an error if token encoding fails.
    pub fn generate_token(&self, user_id: &str, expires_in_hours: i64) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            user_id: user_id.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Validates a JWT token and returns the claims.
    ///
    /// # Errors
    /// Returns an error if token is invalid, expired, or signed with wrong key.
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(data.claims)
    }
}

/// Hashes a password using Argon2.
///
/// # Errors
/// Returns an error if password hashing fails.
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password: {e}"))?;
    Ok(password_hash.to_string())
}

/// Verifies a password against an Argon2 hash.
///
/// # Errors
/// Returns an error if the hash format is invalid.
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash format: {e}"))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Validates an email address format.
///
/// # Errors
/// Returns an error if email format is invalid or exceeds 254 characters.
///
/// # Panics
/// Panics if the internal regex pattern is invalid (should not happen).
pub fn validate_email(email: &str) -> Result<()> {
    // Standard email validation regex preventing leading/trailing dots and consecutive dots
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9_+-]+(\.[a-zA-Z0-9_+-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)*\.[a-zA-Z]{2,}$",
    )
    .expect("Invalid regex pattern");

    if !email_regex.is_match(email) {
        return Err(anyhow!("Invalid email format"));
    }

    if email.len() > 254 {
        return Err(anyhow!("Email address too long"));
    }

    Ok(())
}

/// Validates password meets security policy requirements.
///
/// # Errors
/// Returns an error if password fails policy (length, complexity requirements).
pub fn validate_password_policy(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(anyhow!("Password must be at least 8 characters long"));
    }

    if password.len() > 128 {
        return Err(anyhow!("Password must not exceed 128 characters"));
    }

    let has_uppercase = password.chars().any(char::is_uppercase);
    let has_lowercase = password.chars().any(char::is_lowercase);
    let has_digit = password.chars().any(char::is_numeric);
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err(anyhow!(
            "Password must contain at least one uppercase letter"
        ));
    }

    if !has_lowercase {
        return Err(anyhow!(
            "Password must contain at least one lowercase letter"
        ));
    }

    if !has_digit {
        return Err(anyhow!("Password must contain at least one digit"));
    }

    if !has_special {
        return Err(anyhow!(
            "Password must contain at least one special character"
        ));
    }

    Ok(())
}

#[must_use]
pub fn generate_uuid6_token() -> String {
    uuid::Uuid::now_v6(&[0u8; 6]).to_string()
}
