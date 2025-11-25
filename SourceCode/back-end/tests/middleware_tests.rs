use back_end::auth::{Claims, JwtConfig};

#[test]
fn test_auth_error_missing_token_format() {
    let config = JwtConfig::new("test_secret".to_string());
    let token_result = config.validate_token("");
    assert!(token_result.is_err());
}

#[test]
fn test_auth_error_invalid_token_format() {
    let config = JwtConfig::new("test_secret".to_string());
    let token_result = config.validate_token("invalid.token.format");
    assert!(token_result.is_err());
}

#[test]
fn test_jwt_validation_with_valid_token() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123";
    let token = config.generate_token(user_id.to_string(), 24).unwrap();
    let claims_result = config.validate_token(&token);
    assert!(claims_result.is_ok());
    let claims = claims_result.unwrap();
    assert_eq!(claims.user_id, user_id);
}

#[test]
fn test_jwt_validation_expired_token() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123".to_string(), -1).unwrap();
    let claims_result = config.validate_token(&token);
    assert!(claims_result.is_err());
}

#[test]
fn test_bearer_token_extraction() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user456";
    let token = config.generate_token(user_id.to_string(), 24).unwrap();
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);
}

#[test]
fn test_jwt_claims_contain_user_id() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "test_user_id";
    let token = config.generate_token(user_id.to_string(), 24).unwrap();
    let claims = config.validate_token(&token).unwrap();
    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
}

#[test]
fn test_jwt_claims_expiration() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123".to_string(), 1).unwrap();
    let claims = config.validate_token(&token).unwrap();
    assert!(claims.exp > claims.iat);
}

#[test]
fn test_jwt_different_secrets_cannot_validate() {
    let config1 = JwtConfig::new("secret1".to_string());
    let config2 = JwtConfig::new("secret2".to_string());
    let token = config1.generate_token("user123".to_string(), 24).unwrap();
    let result = config2.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_claims_structure() {
    let claims = Claims {
        sub: "user123".to_string(),
        exp: 1234567890,
        iat: 1234567800,
        user_id: "user123".to_string(),
    };
    assert_eq!(claims.sub, claims.user_id);
    assert!(claims.exp > claims.iat);
}

#[test]
fn test_jwt_token_not_empty() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123".to_string(), 24).unwrap();
    assert!(!token.is_empty());
    assert!(token.len() > 0);
}
