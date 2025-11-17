use back_end::auth::{hash_password, verify_password, validate_email, validate_password_policy, generate_invitation_token, JwtConfig, Claims};

#[test]
fn test_jwt_config_new() {
    let config = JwtConfig::new("test_secret".to_string());
    assert_eq!(config.secret, "test_secret");
}

#[test]
fn test_generate_token_success() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123".to_string(), 1);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(!token.is_empty());
}

#[test]
fn test_generate_token_with_different_expiry() {
    let config = JwtConfig::new("test_secret".to_string());
    let token1 = config.generate_token("user123".to_string(), 1).unwrap();
    let token2 = config.generate_token("user123".to_string(), 24).unwrap();
    assert_ne!(token1, token2);
}

#[test]
fn test_validate_token_success() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123".to_string();
    let token = config.generate_token(user_id.clone(), 24).unwrap();
    let claims = config.validate_token(&token);
    assert!(claims.is_ok());
    let claims = claims.unwrap();
    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
}

#[test]
fn test_validate_token_invalid_token() {
    let config = JwtConfig::new("test_secret".to_string());
    let result = config.validate_token("invalid_token");
    assert!(result.is_err());
}

#[test]
fn test_validate_token_with_wrong_secret() {
    let config1 = JwtConfig::new("secret1".to_string());
    let config2 = JwtConfig::new("secret2".to_string());
    let token = config1.generate_token("user123".to_string(), 24).unwrap();
    let result = config2.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_hash_password_success() {
    let password = "mypassword123";
    let hash = hash_password(password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_password_consistent() {
    let password = "mypassword123";
    let hash1 = hash_password(password).unwrap();
    let hash2 = hash_password(password).unwrap();
    assert_ne!(hash1, hash2);
    assert!(verify_password(password, &hash1).unwrap());
    assert!(verify_password(password, &hash2).unwrap());
}

#[test]
fn test_hash_password_different_inputs() {
    let password1 = "password123";
    let password2 = "password456";
    let hash1 = hash_password(password1).unwrap();
    let hash2 = hash_password(password2).unwrap();
    assert_ne!(hash1, hash2);
}

#[test]
fn test_verify_password_correct() {
    let password = "mypassword123";
    let hash = hash_password(password).unwrap();
    let result = verify_password(password, &hash);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_password_incorrect() {
    let password = "mypassword123";
    let hash = hash_password(password).unwrap();
    let result = verify_password("wrongpassword", &hash);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_verify_password_empty_hash() {
    let password = "mypassword123";
    let result = verify_password(password, "invalid_hash");
    assert!(result.is_err());
}

#[test]
fn test_claims_struct() {
    let claims = Claims {
        sub: "user123".to_string(),
        exp: 1234567890,
        iat: 1234567800,
        user_id: "user123".to_string(),
    };
    assert_eq!(claims.sub, "user123");
    assert_eq!(claims.user_id, "user123");
    assert_eq!(claims.exp, 1234567890);
}

#[test]
fn test_hash_password_empty_string() {
    let password = "";
    let hash = hash_password(password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_password_long_password() {
    let password = "a".repeat(1000);
    let hash = hash_password(&password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_validate_email_valid() {
    assert!(validate_email("user@example.com").is_ok());
    assert!(validate_email("test.user+tag@domain.co.uk").is_ok());
    assert!(validate_email("user123@test-domain.com").is_ok());
}

#[test]
fn test_validate_email_invalid() {
    assert!(validate_email("").is_err());
    assert!(validate_email("invalid").is_err());
    assert!(validate_email("@example.com").is_err());
    assert!(validate_email("user@").is_err());
    assert!(validate_email("user name@example.com").is_err());
}

#[test]
fn test_validate_email_too_long() {
    let long_email = format!("{}@example.com", "a".repeat(300));
    assert!(validate_email(&long_email).is_err());
}

#[test]
fn test_validate_password_policy_valid() {
    assert!(validate_password_policy("Password123!").is_ok());
    assert!(validate_password_policy("Secure@Pass1").is_ok());
    assert!(validate_password_policy("MyP@ssw0rd").is_ok());
}

#[test]
fn test_validate_password_policy_too_short() {
    assert!(validate_password_policy("Pass1!").is_err());
}

#[test]
fn test_validate_password_policy_too_long() {
    let long_password = format!("Password1!{}", "a".repeat(200));
    assert!(validate_password_policy(&long_password).is_err());
}

#[test]
fn test_validate_password_policy_no_uppercase() {
    assert!(validate_password_policy("password123!").is_err());
}

#[test]
fn test_validate_password_policy_no_lowercase() {
    assert!(validate_password_policy("PASSWORD123!").is_err());
}

#[test]
fn test_validate_password_policy_no_digit() {
    assert!(validate_password_policy("Password!").is_err());
}

#[test]
fn test_validate_password_policy_no_special() {
    assert!(validate_password_policy("Password123").is_err());
}

#[test]
fn test_generate_invitation_token_format() {
    let token = generate_invitation_token();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36);
    assert_eq!(token.chars().filter(|&c| c == '-').count(), 4);
}

#[test]
fn test_generate_invitation_token_unique() {
    let token1 = generate_invitation_token();
    let token2 = generate_invitation_token();
    assert_ne!(token1, token2);
}
