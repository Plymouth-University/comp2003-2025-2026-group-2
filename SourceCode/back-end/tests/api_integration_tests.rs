use back_end::{
    handlers::{RegisterRequest, LoginRequest, InviteUserRequest, AcceptInvitationRequest},
    auth::{hash_password, JwtConfig},
    db::{self, UserRole},
};
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

async fn setup_test_db() -> (SqlitePool, NamedTempFile) {
    let _temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_path = _temp_file.path().to_str().expect("Failed to get temp path");
    
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string)
        .await
        .expect("Failed to create test db");

    db::init_db(&pool)
        .await
        .expect("Failed to initialize test db");

    (pool, _temp_file)
}

#[tokio::test]
async fn test_register_creates_user_and_company() {
    let (pool, _temp) = setup_test_db().await;
    
    let password_hash = hash_password("SecurePassword123").unwrap();
    let user = db::create_user(
        &pool,
        "admin@example.com".to_string(),
        "Admin".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Admin,
    )
    .await
    .expect("Failed to create user");

    assert_eq!(user.email, "admin@example.com");
    assert_eq!(user.role, "admin");

    let company = db::create_company(&pool, "Tech Corp".to_string(), "456 Oak Ave".to_string())
        .await
        .expect("Failed to create company");

    assert_eq!(company.name, "Tech Corp");
}

#[tokio::test]
async fn test_user_creation_and_retrieval() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password_hash = hash_password("TestPassword123").unwrap();
    let user = db::create_user(
        &pool,
        "test@example.com".to_string(),
        "John".to_string(),
        "Doe".to_string(),
        password_hash.clone(),
        None,
        UserRole::Member,
    )
    .await
    .expect("Failed to create user");

    let retrieved = db::get_user_by_email(&pool, "test@example.com")
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(retrieved.email, user.email);
    assert_eq!(retrieved.first_name, "John");
    assert_eq!(retrieved.last_name, "Doe");
}

#[tokio::test]
async fn test_password_verification() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password = "MySecurePassword123";
    let password_hash = hash_password(password).unwrap();
    
    db::create_user(
        &pool,
        "user@example.com".to_string(),
        "Test".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Member,
    )
    .await
    .expect("Failed to create user");

    let user = db::get_user_by_email(&pool, "user@example.com")
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    let is_valid = back_end::auth::verify_password(password, &user.password_hash)
        .expect("Failed to verify password");
    
    assert!(is_valid);
}

#[tokio::test]
async fn test_invalid_password_verification() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password = "CorrectPassword123";
    let password_hash = hash_password(password).unwrap();
    
    db::create_user(
        &pool,
        "user@example.com".to_string(),
        "Test".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Member,
    )
    .await
    .expect("Failed to create user");

    let user = db::get_user_by_email(&pool, "user@example.com")
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    let is_valid = back_end::auth::verify_password("WrongPassword", &user.password_hash)
        .expect("Failed to verify password");
    
    assert!(!is_valid);
}

#[tokio::test]
async fn test_jwt_token_generation_and_validation() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123";
    
    let token = config.generate_token(user_id.to_string(), 24)
        .expect("Failed to generate token");

    let claims = config.validate_token(&token)
        .expect("Failed to validate token");

    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
}

#[tokio::test]
async fn test_company_creation_and_retrieval() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let company = db::create_company(&pool, "Test Company".to_string(), "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let retrieved = db::get_company_by_id(&pool, &company.id)
        .await
        .expect("Failed to retrieve company")
        .expect("Company not found");

    assert_eq!(retrieved.id, company.id);
    assert_eq!(retrieved.name, "Test Company");
    assert_eq!(retrieved.address, "123 Main St");
}

#[tokio::test]
async fn test_invitation_creation_and_retrieval() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let company = db::create_company(&pool, "Test Co".to_string(), "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now().to_rfc3339();

    let _invitation = db::create_invitation(
        &pool,
        company.id,
        "newuser@example.com".to_string(),
        token.clone(),
        expires_at,
    )
    .await
    .expect("Failed to create invitation");

    let retrieved = db::get_invitation_by_token(&pool, &token)
        .await
        .expect("Failed to retrieve invitation")
        .expect("Invitation not found");

    assert_eq!(retrieved.email, "newuser@example.com");
    assert_eq!(retrieved.token, token);
}

#[tokio::test]
async fn test_admin_user_creation() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password_hash = hash_password("AdminPass123").unwrap();
    
    let user = db::create_user(
        &pool,
        "admin@example.com".to_string(),
        "Admin".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Admin,
    )
    .await
    .expect("Failed to create admin user");

    assert!(user.is_admin());
    assert_eq!(user.get_role(), UserRole::Admin);
}

#[tokio::test]
async fn test_member_user_creation() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password_hash = hash_password("MemberPass123").unwrap();
    
    let user = db::create_user(
        &pool,
        "member@example.com".to_string(),
        "Member".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Member,
    )
    .await
    .expect("Failed to create member user");

    assert!(!user.is_admin());
    assert_eq!(user.get_role(), UserRole::Member);
}

#[tokio::test]
async fn test_user_with_company_association() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let company = db::create_company(&pool, "My Company".to_string(), "789 Elm St".to_string())
        .await
        .expect("Failed to create company");

    let password_hash = hash_password("Password123").unwrap();
    
    let user = db::create_user(
        &pool,
        "employee@example.com".to_string(),
        "Employee".to_string(),
        "Name".to_string(),
        password_hash,
        Some(company.id.clone()),
        UserRole::Member,
    )
    .await
    .expect("Failed to create user");

    assert_eq!(user.company_id, Some(company.id));
}

#[tokio::test]
async fn test_multiple_users_creation() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password_hash = hash_password("Password123").unwrap();
    
    for i in 0..5 {
        let _user = db::create_user(
            &pool,
            format!("user{}@example.com", i),
            format!("User{}", i),
            "Lastname".to_string(),
            password_hash.clone(),
            None,
            if i % 2 == 0 { UserRole::Admin } else { UserRole::Member },
        )
        .await
        .expect("Failed to create user");
    }

    let user = db::get_user_by_email(&pool, "user0@example.com")
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(user.email, "user0@example.com");
}

#[tokio::test]
async fn test_invitation_acceptance() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let company = db::create_company(&pool, "Test Co".to_string(), "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339();

    let _invitation = db::create_invitation(
        &pool,
        company.id,
        "newmember@example.com".to_string(),
        token,
        expires_at,
    )
    .await
    .expect("Failed to create invitation");

    assert_eq!(_invitation.accepted_at, None);

    db::accept_invitation(&pool, &_invitation.id)
        .await
        .expect("Failed to accept invitation");

    let updated = db::get_invitation_by_token(&pool, &_invitation.token)
        .await
        .expect("Failed to retrieve invitation");

    assert!(updated.is_none());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let (pool, _temp_file) = setup_test_db().await;
    
    let password_hash = hash_password("Password123").unwrap();
    
    let user = db::create_user(
        &pool,
        "testuser@example.com".to_string(),
        "Test".to_string(),
        "User".to_string(),
        password_hash,
        None,
        UserRole::Member,
    )
    .await
    .expect("Failed to create user");

    let retrieved = db::get_user_by_id(&pool, &user.id)
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(retrieved.id, user.id);
    assert_eq!(retrieved.email, user.email);
}

#[tokio::test]
async fn test_token_expiration_validation() {
    let config = JwtConfig::new("test_secret".to_string());
    
    let token = config.generate_token("user123".to_string(), -1)
        .expect("Failed to generate token");

    let result = config.validate_token(&token);
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_request_validation_structures() {
    let register_req = RegisterRequest {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "password123".to_string(),
        company_name: "Test Co".to_string(),
        company_address: "123 Main St".to_string(),
    };

    assert!(!register_req.email.is_empty());
    assert!(register_req.password.len() >= 1);

    let login_req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    assert!(!login_req.email.is_empty());

    let invite_req = InviteUserRequest {
        email: "newuser@example.com".to_string(),
    };

    assert!(!invite_req.email.is_empty());

    let accept_req = AcceptInvitationRequest {
        token: "token123".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        password: "password123".to_string(),
    };

    assert!(!accept_req.token.is_empty());
}
