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
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password: {e}"))?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash format: {e}"))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

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

#[cfg(test)]
mod auth_tests {
    use super::*;

    // JWT Configuration Tests
    #[test]
    fn test_jwt_config_new() {
        let config = JwtConfig::new("test_secret".to_string());
        assert_eq!(config.secret, "test_secret");
    }

    #[test]
    fn test_jwt_config_empty_secret() {
        let config = JwtConfig::new("".to_string());
        assert_eq!(config.secret, "");
    }

    // JWT Token Generation Tests
    #[test]
    fn test_generate_token_success() {
        let config = JwtConfig::new("test_secret".to_string());
        let result = config.generate_token("user123".to_string(), 1);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        assert!(token.contains('.')); // JWT tokens have 3 parts separated by dots
    }

    #[test]
    fn test_generate_token_different_user_ids() {
        let config = JwtConfig::new("test_secret".to_string());
        let token1 = config.generate_token("user1".to_string(), 1).unwrap();
        let token2 = config.generate_token("user2".to_string(), 1).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_token_different_expiry() {
        let config = JwtConfig::new("test_secret".to_string());
        let user_id = "user123".to_string();
        let token1 = config.generate_token(user_id.clone(), 1).unwrap();
        let token2 = config.generate_token(user_id, 24).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_token_with_special_characters() {
        let config = JwtConfig::new("test_secret".to_string());
        let user_id = "user@domain.com+tag#123".to_string();
        let result = config.generate_token(user_id, 1);
        assert!(result.is_ok());
    }

    // JWT Token Validation Tests
    #[test]
    fn test_validate_token_success() {
        let config = JwtConfig::new("test_secret".to_string());
        let user_id = "user123".to_string();
        let token = config.generate_token(user_id.clone(), 24).unwrap();
        let result = config.validate_token(&token);
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.sub, user_id);
        assert!(claims.exp > chrono::Utc::now().timestamp());
        assert!(claims.iat <= chrono::Utc::now().timestamp());
    }

    #[test]
    fn test_validate_token_invalid_token() {
        let config = JwtConfig::new("test_secret".to_string());
        let result = config.validate_token("invalid_token");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_wrong_secret() {
        let config1 = JwtConfig::new("secret1".to_string());
        let config2 = JwtConfig::new("secret2".to_string());
        let token = config1.generate_token("user123".to_string(), 24).unwrap();
        let result = config2.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_expired() {
        let config = JwtConfig::new("test_secret".to_string());
        let token = config.generate_token("user123".to_string(), -1).unwrap(); // Expired 1 hour ago
        let result = config.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_empty_token() {
        let config = JwtConfig::new("test_secret".to_string());
        let result = config.validate_token("");
        assert!(result.is_err());
    }

    // Password Hashing Tests
    #[test]
    fn test_hash_password_success() {
        let password = "test_password123!";
        let result = hash_password(password);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password); // Hash should be different from original password
    }

    #[test]
    fn test_hash_password_different_passwords_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password2").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_same_password_different_hashes() {
        let password = "test_password123!";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_empty_password() {
        let result = hash_password("");
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_hash_password_long_password() {
        let long_password = "a".repeat(1000);
        let result = hash_password(&long_password);
        assert!(result.is_ok());
    }

    // Password Verification Tests
    #[test]
    fn test_verify_password_success() {
        let password = "test_password123!";
        let hash = hash_password(password).unwrap();
        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let hash = hash_password("correct_password").unwrap();
        let result = verify_password("wrong_password", &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hash = hash_password("").unwrap();
        let result = verify_password("", &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "invalid_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_hash() {
        let result = verify_password("password", "");
        assert!(result.is_err());
    }

    // Email Validation Tests
    #[test]
    fn test_validate_email_valid_emails() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
            "user_name123@test-domain.com",
            "a@b.co", // Minimal valid email with 2-char TLD
        ];

        for email in valid_emails {
            let result = validate_email(email);
            assert!(result.is_ok(), "Email {} should be valid", email);
        }
    }

    #[test]
    fn test_validate_email_invalid_emails() {
        let invalid_emails = vec![
            "",
            "plainaddress",
            "@missingdomain.com",
            "missing@.com",
            "missing@domain",
            "spaces @domain.com",
            "user@domain .com",
            "user@domain@domain.com",
            "user@.domain.com",
            ".user@domain.com",
            "user.@domain.com",
            "user@domain..com",
        ];

        for email in invalid_emails {
            let result = validate_email(email);
            assert!(result.is_err(), "Email {} should be invalid", email);
        }
    }

    #[test]
    fn test_validate_email_case_sensitivity() {
        let result = validate_email("Test@EXAMPLE.COM");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(250));
        let result = validate_email(&long_email);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_exactly_max_length() {
        let user = "a".repeat(64);
        let domain = "a".repeat(250 - 64 - 1 - 4); // 250 - user - @ - .com
        let email = format!("{}@{}.com", user, domain);
        let result = validate_email(&email);
        assert!(result.is_ok());
    }

    // Password Policy Validation Tests
    #[test]
    fn test_validate_password_policy_valid_passwords() {
        let valid_passwords = vec![
            "Password123!",
            "MySecureP@ssw0rd",
            "Complex_P@ssw0rd",
            "aB1@cdEF",
            "VerySecurePassword123!",
        ];

        for password in valid_passwords {
            let result = validate_password_policy(password);
            assert!(result.is_ok(), "Password {} should be valid", password);
        }
    }

    #[test]
    fn test_validate_password_policy_too_short() {
        let short_passwords = vec!["", "a", "ab", "abc", "Abc1!", "Pass1"];
        for password in short_passwords {
            let result = validate_password_policy(password);
            assert!(result.is_err(), "Password {} should be too short", password);
        }
    }

    #[test]
    fn test_validate_password_policy_exactly_min_length() {
        let min_password = "Abcdef1!";
        let result = validate_password_policy(min_password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_policy_too_long() {
        let long_password = "a".repeat(129);
        let result = validate_password_policy(&long_password);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_policy_exactly_max_length() {
        let max_password = format!("A{}1!", "b".repeat(125));
        let result = validate_password_policy(&max_password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_policy_no_uppercase() {
        let passwords = vec!["password123!", "lowercase1!", "alllowercase"];
        for password in passwords {
            let result = validate_password_policy(password);
            assert!(
                result.is_err(),
                "Password {} should fail (no uppercase)",
                password
            );
        }
    }

    #[test]
    fn test_validate_password_policy_no_lowercase() {
        let passwords = vec!["PASSWORD123!", "UPPERCASE1!", "ALLUPPERCASE"];
        for password in passwords {
            let result = validate_password_policy(password);
            assert!(
                result.is_err(),
                "Password {} should fail (no lowercase)",
                password
            );
        }
    }

    #[test]
    fn test_validate_password_policy_no_digit() {
        let passwords = vec!["Password!", "NoDigitsHere!", "allletters"];
        for password in passwords {
            let result = validate_password_policy(password);
            assert!(
                result.is_err(),
                "Password {} should fail (no digit)",
                password
            );
        }
    }

    #[test]
    fn test_validate_password_policy_no_special() {
        let passwords = vec!["Password123", "NoSpecialChars123", "allalphanumeric"];
        for password in passwords {
            let result = validate_password_policy(password);
            assert!(
                result.is_err(),
                "Password {} should fail (no special)",
                password
            );
        }
    }

    #[test]
    fn test_validate_password_policy_unicode_characters() {
        let unicode_password = "Pásswörd123!";
        let result = validate_password_policy(unicode_password);
        assert!(result.is_ok());
    }

    // UUID Token Generation Tests
    #[test]
    fn test_generate_uuid6_token_format() {
        let token = generate_uuid6_token();
        assert!(!token.is_empty());

        // UUID format: 8-4-4-4-12 hexadecimal characters with optional version info
        let parts: Vec<&str> = token.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert!(parts[4].len() >= 12);
    }

    #[test]
    fn test_generate_uuid6_token_uniqueness() {
        let token1 = generate_uuid6_token();
        let token2 = generate_uuid6_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_uuid6_token_consistent_length() {
        let token1 = generate_uuid6_token();
        let token2 = generate_uuid6_token();
        assert_eq!(token1.len(), token2.len());
    }

    // Integration Tests - Combined Workflow Tests
    #[test]
    fn test_authentication_workflow() {
        let config = JwtConfig::new("test_secret".to_string());
        let password = "SecurePassword123!";

        // Test password policy
        assert!(validate_password_policy(password).is_ok());

        // Test password hashing
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());

        // Test token generation and validation
        let user_id = "test_user".to_string();
        let token = config.generate_token(user_id.clone(), 1).unwrap();
        let claims = config.validate_token(&token).unwrap();
        assert_eq!(claims.user_id, user_id);
    }

    #[test]
    fn test_email_and_password_validation_combination() {
        let valid_combinations = vec![
            ("user@example.com", "Password123!"),
            ("test.user+tag@domain.co.uk", "SecureP@ssw0rd"),
        ];

        for (email, password) in valid_combinations {
            assert!(
                validate_email(email).is_ok(),
                "Email {} should be valid",
                email
            );
            assert!(
                validate_password_policy(password).is_ok(),
                "Password {} should be valid",
                password
            );
        }
    }

    // Edge Case Tests
    #[test]
    fn test_jwt_config_with_unicode_secret() {
        let unicode_secret = "sëcrët_wïth_ünïcödé_çhàracters".to_string();
        let config = JwtConfig::new(unicode_secret);
        let result = config.generate_token("user123".to_string(), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_with_emoji() {
        let emoji_password = "Password🔒123!";
        let result = validate_password_policy(emoji_password);
        assert!(result.is_ok());

        let hash = hash_password(emoji_password).unwrap();
        let verify_result = verify_password(emoji_password, &hash);
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
    }

    #[test]
    fn test_email_with_international_domain() {
        let international_emails = vec![
            "test@xn--example.com", // Punycode
            "user@münchen.de",      // Should fail with current regex
        ];

        for email in international_emails {
            let result = validate_email(email);
            // Current regex might not handle all international cases
            // This test documents current behavior
            println!("Email validation result for {}: {:?}", email, result);
        }
    }
}
