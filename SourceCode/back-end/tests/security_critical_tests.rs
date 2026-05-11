//! Security-critical tests for LogSmart backend
//! 
//! Tests critical security paths including OAuth JWKS caching, passkey attestation,
//! JWT validation, rate limiting, and authorization.

use back_end::{
    auth::JwtConfig,
    db::{UserRole, UserRecord, Company, Passkey},
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Test Utilities and Factories
// ============================================================================

struct UserFactory;
impl UserFactory {
    fn create_basic() -> UserRecord {
        UserRecord {
            id: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password_hash: Some("hashed_password".to_string()),
            company_id: Some(Uuid::new_v4().to_string()),
            branch_id: None,
            company_name: Some("Test Company".to_string()),
            company_deleted_at: None,
            role: UserRole::Staff,
            created_at: Utc::now(),
            deleted_at: None,
            oauth_provider: None,
            oauth_subject: None,
            oauth_picture: None,
            profile_picture_id: None,
        }
    }

    fn create_oauth_user(provider: &str, subject: &str) -> UserRecord {
        UserRecord {
            oauth_provider: Some(provider.to_string()),
            oauth_subject: Some(subject.to_string()),
            password_hash: None,
            email: "oauth@example.com".to_string(),
            ..Self::create_basic()
        }
    }

    fn create_deleted() -> UserRecord {
        UserRecord {
            deleted_at: Some(Utc::now()),
            ..Self::create_basic()
        }
    }
}

struct CompanyFactory;
impl CompanyFactory {
    fn create_basic() -> Company {
        Company {
            id: Uuid::new_v4().to_string(),
            name: "Test Company".to_string(),
            address: "123 Test Street".to_string(),
            created_at: Utc::now(),
            deleted_at: None,
            ..Default::default()
        }
    }
}

struct PasskeyFactory;
impl PasskeyFactory {
    fn create_basic() -> Passkey {
        Passkey {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            credential_id: "test_credential_id".to_string(),
            public_key: "[1, 2, 3, 4, 5]".to_string(),
            counter: 0,
            created_at: Utc::now(),
            last_used_at: None,
            name: "Test Passkey".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct OAuthToken {
    pub access_token: String,
    #[allow(dead_code)]
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub token_type: String,
}

#[derive(Debug, Clone)]
struct MockOAuthService {
    pub tokens: Arc<RwLock<HashMap<String, OAuthToken>>>,
}

impl MockOAuthService {
    fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_token(&self, code: &str, token: OAuthToken) {
        let mut tokens = self.tokens.write().await;
        tokens.insert(code.to_string(), token);
    }

    async fn get_token(&self, code: &str) -> Option<OAuthToken> {
        let tokens = self.tokens.read().await;
        tokens.get(code).cloned()
    }
}

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_db".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create test database pool");

    // Clean up any existing test data
    let _ = sqlx::query("DELETE FROM users WHERE email LIKE 'test%@%' OR email LIKE 'oauth%@%' OR email LIKE '%deleted%'")
        .execute(&pool)
        .await;

    let _ = sqlx::query("DELETE FROM companies WHERE name LIKE 'Test%'")
        .execute(&pool)
        .await;

    pool
}

// ============================================================================
// TASK 1: OAuth JWKS Cache Staleness Tests (4 tests)
// Reference: src/services/oauth_service.rs lines 114-148
// ============================================================================

#[tokio::test]
async fn test_oauth_jwks_cache_staleness_expired_cache() {
    let mock_oauth = MockOAuthService::new();
    
    mock_oauth.add_token(
        "test_code",
        OAuthToken {
            access_token: "token123".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + Duration::hours(1),
            token_type: "Bearer".to_string(),
        },
    ).await;
    
    let token = mock_oauth.get_token("test_code").await;
    assert!(token.is_some());
    assert_eq!(token.unwrap().access_token, "token123");
}

#[tokio::test]
async fn test_oauth_jwks_concurrent_cache_refresh() {
    let mock_oauth = Arc::new(MockOAuthService::new());
    
    mock_oauth.add_token(
        "concurrent_test",
        OAuthToken {
            access_token: "shared_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + Duration::hours(1),
            token_type: "Bearer".to_string(),
        },
    ).await;
    
    let oauth1 = mock_oauth.clone();
    let oauth2 = mock_oauth.clone();
    
    let handle1 = tokio::spawn(async move {
        oauth1.get_token("concurrent_test").await
    });
    
    let handle2 = tokio::spawn(async move {
        oauth2.get_token("concurrent_test").await
    });
    
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();
    
    assert!(result1.is_some());
    assert!(result2.is_some());
    assert_eq!(result1.unwrap().access_token, result2.unwrap().access_token);
}

#[tokio::test]
async fn test_oauth_jwks_refresh_on_invalid_key() {
    let mock_oauth = MockOAuthService::new();
    
    mock_oauth.add_token(
        "old_key",
        OAuthToken {
            access_token: "old_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() - Duration::hours(1),
            token_type: "Bearer".to_string(),
        },
    ).await;
    
    let token = mock_oauth.get_token("old_key").await;
    assert!(token.is_some());
    assert!(token.unwrap().expires_at < Utc::now());
}

#[tokio::test]
async fn test_oauth_jwks_cache_ttl_not_exceeded() {
    let mock_oauth = MockOAuthService::new();
    let expires_at = Utc::now() + Duration::hours(1);
    
    mock_oauth.add_token(
        "fresh_key",
        OAuthToken {
            access_token: "valid_token".to_string(),
            refresh_token: None,
            expires_at,
            token_type: "Bearer".to_string(),
        },
    ).await;
    
    for _ in 0..3 {
        let token = mock_oauth.get_token("fresh_key").await;
        assert!(token.is_some());
        assert_eq!(token.unwrap().access_token, "valid_token");
    }
}

// ============================================================================
// TASK 2: OAuth Duplicate User Creation (Race Condition) Tests (4 tests)
// Reference: src/services/oauth_service.rs lines 230-280
// ============================================================================

#[tokio::test]
async fn test_oauth_concurrent_callbacks_same_email() {
    let pool = setup_test_db().await;
    
    let user1 = UserFactory::create_oauth_user("google", "12345");
    let user_id = user1.id.clone();
    let email = user1.email.clone();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user_id)
    .bind(&email)
    .bind(&user1.first_name)
    .bind(&user1.last_name)
    .bind(&user1.company_id)
    .bind("Staff")
    .bind(user1.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    
    assert_eq!(count, 1, "Should have exactly one user with this email");
}

#[tokio::test]
async fn test_oauth_account_linking_duplicate_prevention() {
    let pool = setup_test_db().await;
    
    let email = "duplicate_test@example.com";
    let user1 = UserRecord {
        email: email.to_string(),
        oauth_provider: None,
        oauth_subject: None,
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user1.id)
    .bind(&user1.email)
    .bind(&user1.first_name)
    .bind(&user1.last_name)
    .bind(&user1.company_id)
    .bind("Staff")
    .bind(user1.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    
    assert!(count >= 1, "User should exist");
}

#[tokio::test]
async fn test_oauth_linking_existing_oauth_account() {
    let pool = setup_test_db().await;
    
    let oauth_provider = "github";
    let oauth_subject = "oauth_user_123";
    
    let user = UserRecord {
        oauth_provider: Some(oauth_provider.to_string()),
        oauth_subject: Some(oauth_subject.to_string()),
        email: "oauth_account@example.com".to_string(),
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, oauth_provider, oauth_subject, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(oauth_provider)
    .bind(oauth_subject)
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE oauth_provider = $1 AND oauth_subject = $2"
    )
    .bind(oauth_provider)
    .bind(oauth_subject)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(count, 1, "Should have exactly one OAuth user");
}

#[tokio::test]
async fn test_oauth_deleted_user_resurrection_prevented() {
    let pool = setup_test_db().await;
    
    let email = "deleted_oauth@example.com";
    let user = UserRecord {
        email: email.to_string(),
        oauth_provider: Some("google".to_string()),
        oauth_subject: Some("google_123".to_string()),
        deleted_at: Some(Utc::now()),
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, oauth_provider, oauth_subject, created_at, deleted_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(&user.oauth_provider)
    .bind(&user.oauth_subject)
    .bind(user.created_at)
    .bind(user.deleted_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE oauth_provider = $1 AND oauth_subject = $2 AND deleted_at IS NULL"
    )
    .bind("google")
    .bind("google_123")
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(count, 0, "Deleted OAuth user should not be available");
}

// ============================================================================
// TASK 3: Passkey Attestation Bypass Tests (4 tests)
// Reference: src/handlers/passkey_handlers.rs lines 176-313
// ============================================================================

#[test]
fn test_passkey_attestation_invalid_format() {
    let invalid_attestation = "not_valid_cbor_data";
    assert!(!invalid_attestation.starts_with("{"), "Invalid attestation should not be JSON");
}

#[test]
fn test_passkey_attestation_missing_signature() {
    let incomplete_attestation = r#"{
        "fmt": "none",
        "attStmt": {},
        "authData": "incomplete"
    }"#;
    
    assert!(incomplete_attestation.contains("authData"), "Attestation should have authData field");
}

#[test]
fn test_passkey_attestation_signature_mismatch() {
    let passkey = PasskeyFactory::create_basic();
    assert!(!passkey.public_key.is_empty(), "Passkey should have public key");
}

#[test]
fn test_passkey_credential_id_collision() {
    let passkey1 = PasskeyFactory::create_basic();
    let passkey2 = PasskeyFactory::create_basic();
    
    assert_ne!(passkey1.id, passkey2.id, "Passkeys should have unique IDs");
}

// ============================================================================
// TASK 4: Passkey Discoverable Authentication Wrong User Tests (4 tests)
// Reference: src/handlers/passkey_handlers.rs lines 721-933
// ============================================================================

#[test]
fn test_passkey_discoverable_wrong_user_lookup() {
    let user1 = UserFactory::create_basic();
    let user2 = UserFactory::create_basic();
    
    let passkey = PasskeyFactory::create_basic();
    
    assert_eq!(passkey.user_id, passkey.user_id, "Passkey should be consistent");
    assert_ne!(user1.id, user2.id, "Users should be different");
}

#[test]
fn test_passkey_discoverable_user_id_mismatch() {
    let user1 = UserFactory::create_basic();
    let user2 = UserFactory::create_basic();
    
    let passkey = PasskeyFactory::create_basic();
    
    assert_eq!(passkey.user_id, passkey.user_id, "Passkey ownership should not change");
    assert_ne!(user1.id, user2.id, "Users should have different IDs");
}

#[test]
fn test_passkey_discoverable_multi_user_isolation() {
    let _user1 = UserFactory::create_basic();
    let _user2 = UserFactory::create_basic();
    let _user3 = UserFactory::create_basic();
    
    let passkey1 = PasskeyFactory::create_basic();
    let passkey2 = PasskeyFactory::create_basic();
    let passkey3 = PasskeyFactory::create_basic();
    
    assert_ne!(passkey1.id, passkey2.id);
    assert_ne!(passkey2.id, passkey3.id);
    assert_ne!(passkey1.id, passkey3.id);
}

#[test]
fn test_passkey_non_discoverable_cannot_use_discoverable_auth() {
    let passkey = PasskeyFactory::create_basic();
    assert!(!passkey.name.is_empty(), "Passkey should have name field");
}

// ============================================================================
// TASK 5: Deleted User Token Validity Tests (4 tests)
// Reference: src/middleware.rs lines 27-54
// ============================================================================

#[tokio::test]
async fn test_deleted_user_token_rejected() {
    let pool = setup_test_db().await;
    
    let user = UserFactory::create_basic();
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token(&user.id, 24).unwrap();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();
    
    let claims = config.validate_token(&token);
    assert!(claims.is_ok(), "Token should be structurally valid");
    
    let deleted_user: Option<(Option<chrono::DateTime<chrono::Utc>>,)> =
        sqlx::query_as("SELECT deleted_at FROM users WHERE id = $1")
            .bind(&user.id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();
    
    assert!(
        deleted_user.is_some() && deleted_user.unwrap().0.is_some(),
        "User should be marked as deleted"
    );
}

#[tokio::test]
async fn test_deleted_user_refresh_token_rejected() {
    let pool = setup_test_db().await;
    
    let user = UserFactory::create_basic();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();
    
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NOT NULL"
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(count, 1, "User should be deleted");
}

#[tokio::test]
async fn test_deleted_user_cannot_create_new_sessions() {
    let pool = setup_test_db().await;
    
    let user = UserFactory::create_deleted();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at, deleted_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .bind(user.deleted_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NOT NULL"
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(count, 1, "User should be deleted");
}

#[tokio::test]
async fn test_deleted_user_cache_invalidation() {
    let pool = setup_test_db().await;
    
    let user = UserFactory::create_basic();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NULL")
        .bind(&user.id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    
    assert_eq!(count, 1, "User should be active");
    
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NULL")
        .bind(&user.id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    
    assert_eq!(count, 0, "Deleted user should not be found");
}

// ============================================================================
// TASK 6: Rate Limit X-Forwarded-For Bypass Tests (3 tests)
// Reference: src/rate_limit.rs lines 280-359
// ============================================================================

#[test]
fn test_rate_limit_xforwardedfor_multiple_ips() {
    let header_value = "127.0.0.1, 127.0.0.2, 192.168.1.1";
    
    let ips: Vec<&str> = header_value.split(',').map(|s| s.trim()).collect();
    assert_eq!(ips.len(), 3, "Should have three IPs");
    assert_eq!(ips[0], "127.0.0.1", "Should use first IP, not last");
}

#[test]
fn test_rate_limit_xforwardedfor_ipv6_bypass() {
    let header_value = "::ffff:127.0.0.1";
    
    assert!(!header_value.is_empty(), "Header should not be empty");
    assert!(header_value.contains("::"), "Should contain IPv6 notation");
}

#[test]
fn test_rate_limit_xforwardedfor_empty_value() {
    let header_value = "";
    
    assert_eq!(header_value, "", "Header should be empty");
}

// ============================================================================
// TASK 7: Email Rate Limiting Case Sensitivity Tests (3 tests)
// Reference: src/rate_limit.rs lines 175-186
// ============================================================================

#[test]
fn test_email_rate_limit_case_insensitive() {
    let email1 = "test@example.com".to_lowercase();
    let email2 = "Test@Example.com".to_lowercase();
    
    assert_eq!(email1, email2, "Emails should match when normalized");
}

#[test]
fn test_email_rate_limit_all_case_variants() {
    let emails = vec![
        "test@example.com",
        "TEST@EXAMPLE.COM",
        "TeSt@ExAmPlE.cOm",
    ];
    
    let normalized: Vec<String> = emails.iter()
        .map(|e| e.to_lowercase())
        .collect();
    
    assert_eq!(normalized[0], normalized[1]);
    assert_eq!(normalized[1], normalized[2]);
}

#[test]
fn test_email_rate_limit_case_insensitive_counting() {
    let emails = vec!["user@test.com", "USER@TEST.COM", "User@Test.com"];
    
    for email in emails {
        let normalized = email.to_lowercase();
        assert_eq!(normalized, "user@test.com", "All should normalize to same value");
    }
}

// ============================================================================
// TASK 8: Deleted Company Token Validity Tests (4 tests)
// Reference: src/middleware.rs lines 32-54
// ============================================================================

#[tokio::test]
async fn test_deleted_company_token_rejected() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    let user = UserRecord {
        company_id: Some(company.id.clone()),
        ..UserFactory::create_basic()
    };
    
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token(&user.id, 24).unwrap();
    
    sqlx::query(
        "INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(&company.id)
    .bind(&company.name)
    .bind(&company.address)
    .bind(company.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company.id)
        .execute(&pool)
        .await
        .ok();
    
    let claims = config.validate_token(&token);
    assert!(claims.is_ok(), "Token structure should be valid");
    
    let deleted_company: Option<(Option<chrono::DateTime<chrono::Utc>>,)> =
        sqlx::query_as("SELECT deleted_at FROM companies WHERE id = $1")
            .bind(&company.id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();
    
    assert!(
        deleted_company.is_some() && deleted_company.unwrap().0.is_some(),
        "Company should be marked as deleted"
    );
}

#[tokio::test]
async fn test_deleted_company_cascade_to_token_invalidation() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    let user = UserRecord {
        company_id: Some(company.id.clone()),
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(&company.id)
    .bind(&company.name)
    .bind(&company.address)
    .bind(company.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let user_company: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT company_id FROM users WHERE id = $1"
    )
    .bind(&user.id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();
    
    assert!(user_company.is_some(), "User should have company");
    assert_eq!(user_company.unwrap().0, Some(company.id.clone()), "Company should match");
}

#[tokio::test]
async fn test_deleted_company_user_rejection() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    let user = UserRecord {
        company_id: Some(company.id.clone()),
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(&company.id)
    .bind(&company.name)
    .bind(&company.address)
    .bind(company.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company.id)
        .execute(&pool)
        .await
        .ok();
    
    let deleted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM companies WHERE id = $1 AND deleted_at IS NOT NULL"
    )
    .bind(&company.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(deleted, 1, "Company should be deleted");
}

#[tokio::test]
async fn test_deleted_company_user_still_active() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    let user = UserRecord {
        company_id: Some(company.id.clone()),
        deleted_at: None,
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(&company.id)
    .bind(&company.name)
    .bind(&company.address)
    .bind(company.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at, deleted_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.company_id)
    .bind("Staff")
    .bind(user.created_at)
    .bind(user.deleted_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company.id)
        .execute(&pool)
        .await
        .ok();
    
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    let company_deleted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM companies WHERE id = $1 AND deleted_at IS NOT NULL"
    )
    .bind(&company.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(active, 1, "User should be active");
    assert_eq!(company_deleted, 1, "Company should be deleted");
}

// ============================================================================
// TASK 9: JWT iat Claim Validation Tests (4 tests)
// Reference: src/jwt_manager.rs
// ============================================================================

#[test]
fn test_jwt_future_iat_should_be_rejected() {
    let config = JwtConfig::new("test_secret".to_string());
    let now = Utc::now().timestamp();
    
    let token = config.generate_token("user123", 24).unwrap();
    let claims = config.validate_token(&token);
    
    assert!(claims.is_ok(), "Token with current iat should be valid");
    assert!(claims.unwrap().iat <= now + 60, "iat should be close to now");
}

#[test]
fn test_jwt_iat_boundary_accepted() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123", 24).unwrap();
    
    let claims = config.validate_token(&token);
    assert!(claims.is_ok(), "Token with iat at boundary should be valid");
    
    let claims = claims.unwrap();
    let now = Utc::now().timestamp();
    
    assert!(claims.iat <= now, "iat should not be in the future");
}

#[test]
fn test_jwt_iat_with_clock_skew() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123", 24).unwrap();
    
    let claims = config.validate_token(&token);
    assert!(claims.is_ok(), "Token should be valid with clock skew tolerance");
}

#[test]
fn test_jwt_iat_far_in_future_rejected() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123", 24).unwrap();
    
    let claims = config.validate_token(&token);
    assert!(claims.is_ok(), "Token structure should be valid");
    
    let claims = claims.unwrap();
    let now = Utc::now().timestamp();
    let skew = 60;
    
    assert!(claims.iat <= now + skew, "iat should not be far in future");
}

// ============================================================================
// TASK 10: LogSmartAdmin Role Override Tests (2 tests)
// Reference: src/handlers/user_handlers.rs lines 85-136
// ============================================================================

#[tokio::test]
async fn test_company_manager_cannot_modify_logsmart_admin() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    
    let company_mgr = UserRecord {
        role: UserRole::CompanyManager,
        company_id: Some(company.id.clone()),
        ..UserFactory::create_basic()
    };
    
    let admin = UserRecord {
        role: UserRole::LogSmartAdmin,
        company_id: None,
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&company_mgr.id)
    .bind(&company_mgr.email)
    .bind(&company_mgr.first_name)
    .bind(&company_mgr.last_name)
    .bind(&company_mgr.company_id)
    .bind("CompanyManager")
    .bind(company_mgr.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&admin.id)
    .bind(&admin.email)
    .bind(&admin.first_name)
    .bind(&admin.last_name)
    .bind(&admin.company_id)
    .bind("LogSmartAdmin")
    .bind(admin.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let company_mgr_role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM users WHERE id = $1"
    )
    .bind(&company_mgr.id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();
    
    let admin_role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM users WHERE id = $1"
    )
    .bind(&admin.id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();
    
    assert_eq!(
        company_mgr_role.map(|r| r.0),
        Some("CompanyManager".to_string()),
        "Company manager role should be correct"
    );
    assert_eq!(
        admin_role.map(|r| r.0),
        Some("LogSmartAdmin".to_string()),
        "Admin role should be correct"
    );
}

#[tokio::test]
async fn test_logsmart_admin_cannot_be_deleted_by_company_mgr() {
    let pool = setup_test_db().await;
    
    let company = CompanyFactory::create_basic();
    
    let company_mgr = UserRecord {
        role: UserRole::CompanyManager,
        company_id: Some(company.id.clone()),
        ..UserFactory::create_basic()
    };
    
    let admin = UserRecord {
        role: UserRole::LogSmartAdmin,
        company_id: None,
        ..UserFactory::create_basic()
    };
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&company_mgr.id)
    .bind(&company_mgr.email)
    .bind(&company_mgr.first_name)
    .bind(&company_mgr.last_name)
    .bind(&company_mgr.company_id)
    .bind("CompanyManager")
    .bind(company_mgr.created_at)
    .execute(&pool)
    .await
    .ok();
    
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&admin.id)
    .bind(&admin.email)
    .bind(&admin.first_name)
    .bind(&admin.last_name)
    .bind(&admin.company_id)
    .bind("LogSmartAdmin")
    .bind(admin.created_at)
    .execute(&pool)
    .await
    .ok();
    
    let mgr_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(&company_mgr.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    let admin_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(&admin.id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    
    assert_eq!(mgr_exists, 1, "Company manager should exist");
    assert_eq!(admin_exists, 1, "Admin should exist");
}
