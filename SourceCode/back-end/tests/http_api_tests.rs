use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use back_end::{
    handlers::{register_company_admin, login, get_current_user, invite_user, accept_invitation},
    db, AppState,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tempfile::NamedTempFile;
use tower::ServiceExt;

async fn setup_test_app() -> (Router, NamedTempFile) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_str().expect("Failed to get temp path");
    
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string)
        .await
        .expect("Failed to create test db");

    db::init_db(&pool)
        .await
        .expect("Failed to initialize test db");

    let state = AppState { sqlite: pool };

    let app = Router::new()
        .route("/auth/register", post(register_company_admin))
        .route("/auth/login", post(login))
        .route("/auth/me", get(get_current_user))
        .route("/auth/invitations/send", post(invite_user))
        .route("/auth/invitations/accept", post(accept_invitation))
        .with_state(state);

    (app, temp_file)
}

async fn make_request(
    app: &mut Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let body_bytes = if let Some(b) = body {
        serde_json::to_vec(&b).unwrap()
    } else {
        Vec::new()
    };

    let mut request = match method {
        "GET" => Request::builder().method("GET"),
        "POST" => Request::builder().method("POST"),
        _ => panic!("Unsupported method"),
    };

    request = request
        .uri(path)
        .header("content-type", "application/json");

    if let Some(t) = token {
        request = request.header("authorization", format!("Bearer {}", t));
    }

    let request = request
        .body(Body::from(body_bytes))
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");
    let status = response.status();
    
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");

    let json_body: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({}))
    };

    (status, json_body)
}

#[tokio::test]
async fn test_register_user_success() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, body) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "SecurePassword123!",
            "company_name": "Test Company",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "admin@example.com");
    assert_eq!(body["user"]["role"], "admin");
}

#[tokio::test]
async fn test_register_user_short_password() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, body) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "short",
            "company_name": "Test Company",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_register_user_missing_fields() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "",
            "first_name": "",
            "last_name": "",
            "password": "",
            "company_name": "",
            "company_address": ""
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_user_success() {
    let (mut app, _temp) = setup_test_app().await;

    let register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "user@example.com",
            "first_name": "Test",
            "last_name": "User",
            "password": "TestPassword123!",
            "company_name": "Test Co",
            "company_address": "456 Oak Ave"
        })),
        None,
    )
    .await;

    assert_eq!(register_response.0, StatusCode::CREATED);

    let (login_status, login_body) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "user@example.com",
            "password": "TestPassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(login_status, StatusCode::OK);
    assert!(login_body["token"].is_string());
    assert_eq!(login_body["user"]["email"], "user@example.com");
}

#[tokio::test]
async fn test_login_user_invalid_password() {
    let (mut app, _temp) = setup_test_app().await;

    let _register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "user@example.com",
            "first_name": "Test",
            "last_name": "User",
            "password": "CorrectPassword123!",
            "company_name": "Test Co",
            "company_address": "456 Oak Ave"
        })),
        None,
    )
    .await;

    let (login_status, _login_body) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "user@example.com",
            "password": "WrongPassword1!"
        })),
        None,
    )
    .await;

    assert_eq!(login_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_user_nonexistent() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "nonexistent@example.com",
            "password": "SomePassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user_without_token() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(&mut app, "GET", "/auth/me", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user_with_valid_token() {
    let (mut app, _temp) = setup_test_app().await;

    let register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "user@example.com",
            "first_name": "Test",
            "last_name": "User",
            "password": "TestPassword123!",
            "company_name": "Test Co",
            "company_address": "456 Oak Ave"
        })),
        None,
    )
    .await;

    let token = register_response.1["token"].as_str().unwrap();

    let (status, body) = make_request(&mut app, "GET", "/auth/me", None, Some(token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "user@example.com");
}

#[tokio::test]
async fn test_get_current_user_with_invalid_token() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(&mut app, "GET", "/auth/me", None, Some("invalid_token")).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invite_user_by_admin() {
    let (mut app, _temp) = setup_test_app().await;

    let register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "AdminPassword123!",
            "company_name": "Test Co",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    let admin_token = register_response.1["token"].as_str().unwrap();

    let (status, body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": "newuser@example.com"
        })),
        Some(admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["email"], "newuser@example.com");
    assert!(body["expires_at"].is_string());
}

#[tokio::test]
async fn test_invite_user_missing_email() {
    let (mut app, _temp) = setup_test_app().await;

    let register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "AdminPassword123!",
            "company_name": "Test Co",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    let admin_token = register_response.1["token"].as_str().unwrap();

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": ""
        })),
        Some(admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invite_user_by_non_admin() {
    let (mut app, temp) = setup_test_app().await;

    let admin_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "AdminPassword123!",
            "company_name": "Test Co",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    assert_eq!(admin_response.0, StatusCode::CREATED);
    let admin_token = admin_response.1["token"].as_str().unwrap();

    let invite_response = make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": "member@example.com"
        })),
        Some(admin_token),
    )
    .await;

    assert_eq!(invite_response.0, StatusCode::CREATED);
    let invitation_id = invite_response.1["id"].as_str().unwrap().to_string();

    let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", temp.path().to_str().unwrap()))
        .await
        .expect("Failed to connect to db");
    
    let invitation = db::get_invitation_by_token(&pool, &invitation_id)
        .await
        .ok()
        .flatten();

    let actual_token = if let Some(inv) = invitation {
        inv.token
    } else {
        let all_invites = sqlx::query_as::<_, (String,)>("SELECT token FROM invitations WHERE email = ?")
            .bind("member@example.com")
            .fetch_one(&pool)
            .await
            .map(|row| row.0)
            .unwrap_or_else(|_| invitation_id.clone());
        all_invites
    };

    let accept_response = make_request(
        &mut app,
        "POST",
        "/auth/invitations/accept",
        Some(json!({
            "token": actual_token,
            "first_name": "Member",
            "last_name": "User",
            "password": "MemberPassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(accept_response.0, StatusCode::CREATED);
    let member_token = accept_response.1["token"].as_str().unwrap();

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": "anotheruser@example.com"
        })),
        Some(member_token),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_accept_invitation_missing_fields() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/accept",
        Some(json!({
            "token": "",
            "first_name": "",
            "last_name": "",
            "password": ""
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_accept_invitation_short_password() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/accept",
        Some(json!({
            "token": "token123",
            "first_name": "Jane",
            "last_name": "Doe",
            "password": "short"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_accept_invitation_invalid_token() {
    let (mut app, _temp) = setup_test_app().await;

    let (status, _body) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/accept",
        Some(json!({
            "token": "invalid_token",
            "first_name": "Jane",
            "last_name": "Doe",
            "password": "SecurePass123!"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_complete_registration_and_login_flow() {
    let (mut app, _temp) = setup_test_app().await;

    let register_response = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@example.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "AdminPassword123!",
            "company_name": "Tech Corp",
            "company_address": "789 Elm St"
        })),
        None,
    )
    .await;

    assert_eq!(register_response.0, StatusCode::CREATED);
    let admin_token = register_response.1["token"].as_str().unwrap().to_string();

    let (login_status, login_body) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "admin@example.com",
            "password": "AdminPassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(login_status, StatusCode::OK);
    assert_eq!(login_body["user"]["role"], "admin");

    let (me_status, me_body) = make_request(&mut app, "GET", "/auth/me", None, Some(&admin_token)).await;

    assert_eq!(me_status, StatusCode::OK);
    assert_eq!(me_body["email"], "admin@example.com");
    assert_eq!(me_body["role"], "admin");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (mut app, _temp) = setup_test_app().await;

    let _first_register = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "user@example.com",
            "first_name": "User",
            "last_name": "One",
            "password": "Password123!",
            "company_name": "Company One",
            "company_address": "123 Main St"
        })),
        None,
    )
    .await;

    let (status, body) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "user@example.com",
            "first_name": "User",
            "last_name": "Two",
            "password": "Password456!",
            "company_name": "Company Two",
            "company_address": "456 Oak Ave"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].is_string());
}
