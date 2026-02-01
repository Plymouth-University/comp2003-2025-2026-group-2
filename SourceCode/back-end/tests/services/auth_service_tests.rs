use crate::services::auth_service::AuthService;
use back_end::auth::AuthServiceExt;
use back_end::tests::common::{factories::*, setup_test_db};
use uuid::Uuid;

#[tokio::test]
async fn test_authenticate_user_success() {
    let pool = setup_test_db().await;
    
    // Create test user with known password
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test successful authentication
    let result = AuthService::authenticate(&pool, "test@example.com", "password123").await;
    
    assert!(result.is_ok());
    let authenticated_user = result.unwrap();
    assert_eq!(authenticated_user.id, user.id);
    assert_eq!(authenticated_user.email, user.email);
}

#[tokio::test]
async fn test_authenticate_user_invalid_email() {
    let pool = setup_test_db().await;
    
    // Test with non-existent email
    let result = AuthService::authenticate(&pool, "nonexistent@example.com", "password123").await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Invalid credentials"));
}

#[tokio::test]
async fn test_authenticate_user_invalid_password() {
    let pool = setup_test_db().await;
    
    // Create test user
    create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test with wrong password
    let result = AuthService::authenticate(&pool, "test@example.com", "wrongpassword").await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Invalid credentials"));
}

#[tokio::test]
async fn test_authenticate_user_missing_password() {
    let pool = setup_test_db().await;
    
    // Create test user without password hash
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Update user to have null password hash
    sqlx::query("UPDATE users SET password_hash = NULL WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Test authentication should fail
    let result = AuthService::authenticate(&pool, "test@example.com", "password123").await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Invalid credentials"));
}

#[tokio::test]
async fn test_register_user_success() {
    let pool = setup_test_db().await;
    
    // Test successful registration
    let result = AuthService::register(
        &pool,
        "newuser@example.com",
        "Password123!",
        Some("New User"),
        Some("company123")
    ).await;
    
    assert!(result.is_ok());
    let registered_user = result.unwrap();
    assert_eq!(registered_user.email, "newuser@example.com");
    assert_eq!(registered_user.full_name, Some("New User".to_string()));
    assert_eq!(registered_user.company_id, Some("company123".to_string()));
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let pool = setup_test_db().await;
    
    // Create existing user
    create_test_user(&pool, "existing@example.com", Some("company123")).await;
    
    // Test registration with duplicate email
    let result = AuthService::register(
        &pool,
        "existing@example.com",
        "Password123!",
        Some("New User"),
        Some("company123")
    ).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("already exists"));
}

#[tokio::test]
async fn test_register_user_weak_password() {
    let pool = setup_test_db().await;
    
    // Test registration with weak password
    let result = AuthService::register(
        &pool,
        "weak@example.com",
        "123",  // Too weak
        Some("Weak User"),
        Some("company123")
    ).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Password does not meet requirements"));
}

#[tokio::test]
async fn test_verify_password_success() {
    // Test password verification with known hash
    let password = "TestPassword123!";
    let hash = AuthService::hash_password(password).unwrap();
    
    let result = AuthService::verify_password(&hash, password);
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_verify_password_failure() {
    // Test password verification with wrong password
    let password = "TestPassword123!";
    let wrong_password = "WrongPassword123!";
    let hash = AuthService::hash_password(password).unwrap();
    
    let result = AuthService::verify_password(&hash, wrong_password);
    
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_hash_password_strength() {
    let password = "TestPassword123!";
    
    // Hash password multiple times to ensure consistency
    let hash1 = AuthService::hash_password(password).unwrap();
    let hash2 = AuthService::hash_password(password).unwrap();
    
    // Hashes should be different (due to salt) but both valid
    assert_ne!(hash1, hash2);
    
    // Both should verify correctly
    assert!(AuthService::verify_password(&hash1, password).unwrap());
    assert!(AuthService::verify_password(&hash2, password).unwrap());
}

#[tokio::test]
async fn test_token_generation_and_validation() {
    let pool = setup_test_db().await;
    
    // Create test user
    let user = create_test_user(&pool, "token@example.com", Some("company123")).await;
    
    // Generate token
    let token = AuthService::generate_token(&user).unwrap();
    
    // Validate token
    let claims = AuthService::validate_token(&token).unwrap();
    
    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.email, user.email);
    assert_eq!(claims.role, user.role);
}

#[tokio::test]
async fn test_token_validation_invalid() {
    // Test with invalid token
    let invalid_token = "invalid.jwt.token";
    
    let result = AuthService::validate_token(invalid_token);
    
    assert!(result.is_err());
}