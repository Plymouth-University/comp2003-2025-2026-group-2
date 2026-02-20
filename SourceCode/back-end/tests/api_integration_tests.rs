use back_end::{
    auth::{JwtConfig, hash_password},
    db::{self, UserRole},
    dto::{AcceptInvitationRequest, InviteUserRequest, LoginRequest, RegisterRequest},
};
use sqlx::PgPool;

/// Get a connection pool to the test database
async fn get_test_pool() -> PgPool {
    let connection_string = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:adminpassword@localhost:5432/logsmartdb".to_string());

    PgPool::connect(&connection_string)
        .await
        .expect("Failed to create test db connection")
}

/// Setup each test with a database pool
/// Each test uses unique identifiers so they don't interfere with each other
async fn setup_test_pool() -> PgPool {
    get_test_pool().await
}

#[tokio::test]
async fn test_register_creates_user_and_company() {
    let pool = setup_test_pool().await;

    let test_id = uuid::Uuid::new_v4().to_string().replace("-", "");
    let password_hash = hash_password("SecurePassword123").unwrap();
    let user = db::create_user(
        &pool,
        format!("testregisteradmin{}@example.com", test_id),
        "Admin".to_string(),
        "User".to_string(),
        Some(password_hash),
        None,
        UserRole::CompanyManager,
    )
    .await
    .expect("Failed to create user");

    assert!(user.email.starts_with("testregisteradmin"));
    assert_eq!(user.role, UserRole::CompanyManager);

    let company = db::create_company(
        &pool,
        format!("Tech Corp Test {}", test_id),
        "456 Oak Ave".to_string(),
    )
    .await
    .expect("Failed to create company");

    assert!(company.name.starts_with("Tech Corp Test"));
}

#[tokio::test]
async fn test_user_creation_and_retrieval() {
    let pool = setup_test_pool().await;

    let password_hash = hash_password("TestPassword123").unwrap();
    let test_email = format!("test_user_{}@example.com", uuid::Uuid::new_v4());

    let user = db::create_user(
        &pool,
        test_email.clone(),
        "John".to_string(),
        "Doe".to_string(),
        Some(password_hash.clone()),
        None,
        UserRole::Staff,
    )
    .await
    .expect("Failed to create user");

    let retrieved = db::get_user_by_email(&pool, &test_email)
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(retrieved.email, user.email);
    assert_eq!(retrieved.first_name, "John");
    assert_eq!(retrieved.last_name, "Doe");
}

#[tokio::test]
async fn test_member_user_creation() {
    let pool = setup_test_pool().await;

    let password = "SecurePassword123";
    let password_hash = hash_password(password).unwrap();
    let test_email = format!("test_member_{}@example.com", uuid::Uuid::new_v4());

    let user = db::create_user(
        &pool,
        test_email.clone(),
        "John".to_string(),
        "Doe".to_string(),
        Some(password_hash.clone()),
        None,
        UserRole::Staff,
    )
    .await
    .expect("Failed to create user");

    let retrieved = db::get_user_by_email(&pool, &test_email)
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(retrieved.email, user.email);
    assert_eq!(retrieved.first_name, user.first_name);
    assert_eq!(retrieved.last_name, user.last_name);
    assert_eq!(retrieved.password_hash, user.password_hash);
}

#[tokio::test]
async fn test_invalid_password_verification() {
    let pool = setup_test_pool().await;

    let password = "CorrectPassword123";
    let password_hash = hash_password(password).unwrap();
    let test_email = format!("test_password_{}@example.com", uuid::Uuid::new_v4());

    db::create_user(
        &pool,
        test_email.clone(),
        "Test".to_string(),
        "User".to_string(),
        Some(password_hash),
        None,
        UserRole::Staff,
    )
    .await
    .expect("Failed to create user");

    let user = db::get_user_by_email(&pool, &test_email)
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    let is_valid =
        back_end::auth::verify_password("WrongPassword", user.password_hash.as_ref().unwrap())
            .expect("Failed to verify password");

    assert!(!is_valid);
}

#[tokio::test]
async fn test_jwt_token_generation_and_validation() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123";

    let token = config
        .generate_token(user_id.as_str(), 24)
        .expect("Failed to generate token");

    let claims = config
        .validate_token(&token)
        .expect("Failed to validate token");

    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
}

#[tokio::test]
async fn test_company_creation_and_retrieval() {
    let pool = setup_test_pool().await;

    let company_name = format!("Test Company {}", uuid::Uuid::new_v4());
    let company = db::create_company(&pool, company_name.clone(), "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let retrieved = db::get_company_by_id(&pool, &company.id)
        .await
        .expect("Failed to retrieve company")
        .expect("Company not found");

    assert_eq!(retrieved.id, company.id);
    assert_eq!(retrieved.name, company_name);
    assert_eq!(retrieved.address, "123 Main St");
}

#[tokio::test]
async fn test_invitation_creation_and_retrieval() {
    let pool = setup_test_pool().await;

    let company_name = format!("Test Co {}", uuid::Uuid::new_v4());
    let company = db::create_company(&pool, company_name, "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let token = uuid::Uuid::new_v4().to_string();
    let test_email = format!("newuser_{}@example.com", uuid::Uuid::new_v4());
    let expires_at = chrono::Utc::now() + chrono::Duration::days(1);

    let _invitation = db::create_invitation(
        &pool,
        company.id,
        test_email.clone(),
        token.clone(),
        UserRole::Staff,
        None,
        expires_at,
    )
    .await
    .expect("Failed to create invitation");

    let retrieved = db::get_invitation_by_token(&pool, &token)
        .await
        .expect("Failed to retrieve invitation")
        .expect("Invitation not found");

    assert_eq!(retrieved.email, test_email);
    assert_eq!(retrieved.token, token);
}

#[tokio::test]
async fn test_admin_user_creation() {
    let pool = setup_test_pool().await;

    let password_hash = hash_password("AdminPass123").unwrap();
    let test_email = format!("admin_{}@example.com", uuid::Uuid::new_v4());

    let user = db::create_user(
        &pool,
        test_email,
        "Admin".to_string(),
        "User".to_string(),
        Some(password_hash),
        None,
        UserRole::CompanyManager,
    )
    .await
    .expect("Failed to create admin user");

    assert!(user.is_company_manager());
    assert_eq!(user.get_role(), UserRole::CompanyManager);
}

#[tokio::test]
async fn test_user_with_company_association() {
    let pool = setup_test_pool().await;

    let company_name = format!("My Company {}", uuid::Uuid::new_v4());
    let company = db::create_company(&pool, company_name, "789 Elm St".to_string())
        .await
        .expect("Failed to create company");

    let password_hash = hash_password("Password123").unwrap();
    let test_email = format!("employee_{}@example.com", uuid::Uuid::new_v4());

    let user = db::create_user(
        &pool,
        test_email,
        "Employee".to_string(),
        "Name".to_string(),
        Some(password_hash),
        Some(company.id.clone()),
        UserRole::Staff,
    )
    .await
    .expect("Failed to create user");

    assert_eq!(user.company_id, Some(company.id));
}

#[tokio::test]
async fn test_multiple_users_creation() {
    let pool = setup_test_pool().await;

    let password_hash = hash_password("Password123").unwrap();
    let test_id = uuid::Uuid::new_v4().to_string().replace("-", "");

    for i in 0..5 {
        let email = format!("multiuser{}_{}@example.com", test_id, i);
        let _user = db::create_user(
            &pool,
            email,
            format!("User{}", i),
            "Lastname".to_string(),
            Some(password_hash.clone()),
            None,
            if i % 2 == 0 {
                UserRole::CompanyManager
            } else {
                UserRole::Staff
            },
        )
        .await
        .expect("Failed to create user");
    }

    let first_email = format!("multiuser{}_0@example.com", test_id);
    let user = db::get_user_by_email(&pool, &first_email)
        .await
        .expect("Failed to retrieve user")
        .expect("User not found");

    assert_eq!(user.email, first_email);
}

#[tokio::test]
async fn test_invitation_acceptance() {
    let pool = setup_test_pool().await;

    let company_name = format!("Test Co {}", uuid::Uuid::new_v4());
    let company = db::create_company(&pool, company_name, "123 Main St".to_string())
        .await
        .expect("Failed to create company");

    let token = uuid::Uuid::new_v4().to_string();
    let test_email = format!("newmember_{}@example.com", uuid::Uuid::new_v4());
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let _invitation = db::create_invitation(
        &pool,
        company.id,
        test_email,
        token,
        UserRole::Staff,
        None,
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
async fn test_token_expiration_validation() {
    let config = JwtConfig::new("test_secret".to_string());

    let token = config
        .generate_token("user123", -1)
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
        role: Some(UserRole::Staff),
        branch_id: None,
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
