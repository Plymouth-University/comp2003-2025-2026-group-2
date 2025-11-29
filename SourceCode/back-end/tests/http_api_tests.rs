use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{ConnectInfo, State},
    http::{HeaderMap, Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use back_end::{AppState, db, handlers, middleware::AuthToken};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tempfile::NamedTempFile;
use tower::ServiceExt;

async fn test_register_handler(
    State(state): State<AppState>,
    Json(payload): Json<handlers::RegisterRequest>,
) -> axum::response::Response {
    let mock_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let headers = HeaderMap::new();
    let result = handlers::register_company_admin(
        State(state),
        ConnectInfo(mock_addr),
        headers,
        Json(payload),
    )
    .await;
    match result {
        Ok(ok) => ok.into_response(),
        Err(err) => err.into_response(),
    }
}

async fn test_login_handler(
    State(state): State<AppState>,
    Json(payload): Json<handlers::LoginRequest>,
) -> axum::response::Response {
    let mock_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let headers = HeaderMap::new();
    let result =
        handlers::login(State(state), ConnectInfo(mock_addr), headers, Json(payload)).await;
    match result {
        Ok(ok) => ok.into_response(),
        Err(err) => err.into_response(),
    }
}

async fn test_invite_handler(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<handlers::InviteUserRequest>,
) -> Result<(StatusCode, Json<handlers::InvitationResponse>), (StatusCode, Json<Value>)> {
    let mock_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let headers = HeaderMap::new();
    handlers::invite_user(
        AuthToken(claims),
        State(state),
        ConnectInfo(mock_addr),
        headers,
        Json(payload),
    )
    .await
}

async fn test_accept_invitation_handler(
    State(state): State<AppState>,
    Json(payload): Json<handlers::AcceptInvitationRequest>,
) -> axum::response::Response {
    let mock_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let headers = HeaderMap::new();
    let result =
        handlers::accept_invitation(State(state), ConnectInfo(mock_addr), headers, Json(payload))
            .await;
    match result {
        Ok(ok) => ok.into_response(),
        Err(err) => err.into_response(),
    }
}
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

    let state = AppState {
        sqlite: pool,
        rate_limit: back_end::rate_limit::RateLimitState::new(),
        metrics: back_end::metrics::Metrics::new(),
        mongodb: back_end::logs_db::init_mongodb().await.expect("Failed to initialize MongoDB"),
    };

    let app = Router::new()
        .route("/auth/register", post(test_register_handler))
        .route("/auth/login", post(test_login_handler))
        .route("/auth/me", get(handlers::get_current_user))
        .route("/auth/invitations/send", post(test_invite_handler))
        .route(
            "/auth/invitations/accept",
            post(test_accept_invitation_handler),
        )
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

    request = request.uri(path).header("content-type", "application/json");

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

    let (status, _body) =
        make_request(&mut app, "GET", "/auth/me", None, Some("invalid_token")).await;

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

    let pool = SqlitePool::connect(&format!(
        "sqlite://{}?mode=rwc",
        temp.path().to_str().unwrap()
    ))
    .await
    .expect("Failed to connect to db");

    let invitation = db::get_invitation_by_token(&pool, &invitation_id)
        .await
        .ok()
        .flatten();

    let actual_token = if let Some(inv) = invitation {
        inv.token
    } else {
        let all_invites =
            sqlx::query_as::<_, (String,)>("SELECT token FROM invitations WHERE email = ?")
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

    let (me_status, me_body) =
        make_request(&mut app, "GET", "/auth/me", None, Some(&admin_token)).await;

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

#[tokio::test]
async fn test_security_logging_on_successful_registration() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let (status, _) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "newuser@example.com",
            "first_name": "New",
            "last_name": "User",
            "password": "SecurePass123!",
            "company_name": "Test Company",
            "company_address": "123 Test St"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let user = db::get_user_by_email(&pool, "newuser@example.com")
        .await
        .unwrap()
        .unwrap();
    let user_id = &user.id;
    let logs = db::get_security_logs_by_user(&pool, user_id, 10)
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].event_type, "registration");
    assert_eq!(logs[0].user_id, Some(user_id.to_string()));
    assert_eq!(logs[0].email, Some("newuser@example.com".to_string()));
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_security_logging_on_successful_login() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let _ = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "logintest@example.com",
            "first_name": "Login",
            "last_name": "Test",
            "password": "MyPassword123!",
            "company_name": "Login Company",
            "company_address": "456 Login Ave"
        })),
        None,
    )
    .await;

    let (status, _) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "logintest@example.com",
            "password": "MyPassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let user = db::get_user_by_email(&pool, "logintest@example.com")
        .await
        .unwrap()
        .unwrap();
    let user_id = &user.id;
    let logs = db::get_security_logs_by_user(&pool, user_id, 10)
        .await
        .unwrap();

    assert!(logs.len() >= 2);
    assert_eq!(logs[0].event_type, "login_success");
    assert_eq!(logs[0].user_id, Some(user_id.to_string()));
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_security_logging_on_failed_login_wrong_password() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let _ = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "failtest@example.com",
            "first_name": "Fail",
            "last_name": "Test",
            "password": "CorrectPass123!",
            "company_name": "Fail Company",
            "company_address": "789 Fail Rd"
        })),
        None,
    )
    .await;

    let (status, body) = make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "failtest@example.com",
            "password": "WrongPassword123!"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid email or password")
    );

    let user = db::get_user_by_email(&pool, "failtest@example.com")
        .await
        .unwrap()
        .unwrap();
    let user_id = &user.id;
    let logs = db::get_security_logs_by_user(&pool, user_id, 10)
        .await
        .unwrap();

    let failed_login = logs.iter().find(|l| l.event_type == "login_failed");
    assert!(failed_login.is_some());

    let failed_log = failed_login.unwrap();
    assert_eq!(failed_log.user_id, Some(user_id.to_string()));
    assert_eq!(failed_log.email, Some("failtest@example.com".to_string()));
    assert!(!failed_log.success);
    assert_eq!(failed_log.details, Some("Invalid password".to_string()));
}

#[tokio::test]
async fn test_security_logging_on_failed_login_user_not_found() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let (status, body) = make_request(
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
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid email or password")
    );

    let logs = db::get_recent_security_logs(&pool, Some("login_failed".to_string()), 10)
        .await
        .unwrap();

    let failed_login = logs
        .iter()
        .find(|l| l.email == Some("nonexistent@example.com".to_string()));
    assert!(failed_login.is_some());

    let failed_log = failed_login.unwrap();
    assert_eq!(failed_log.user_id, None);
    assert!(!failed_log.success);
    assert_eq!(failed_log.details, Some("User not found".to_string()));
}

#[tokio::test]
async fn test_security_logging_on_invitation_sent() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let (_, register_response) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "admin@company.com",
            "first_name": "Admin",
            "last_name": "User",
            "password": "AdminPass123!",
            "company_name": "Invite Company",
            "company_address": "321 Invite St"
        })),
        None,
    )
    .await;

    let admin_token = register_response["token"].as_str().unwrap();

    let (status, _) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": "newmember@company.com"
        })),
        Some(admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let admin = db::get_user_by_email(&pool, "admin@company.com")
        .await
        .unwrap()
        .unwrap();
    let admin_id = &admin.id;
    let logs = db::get_security_logs_by_user(&pool, admin_id, 10)
        .await
        .unwrap();

    let invite_log = logs.iter().find(|l| l.event_type == "invitation_sent");
    assert!(invite_log.is_some());

    let log = invite_log.unwrap();
    assert_eq!(log.user_id, Some(admin_id.to_string()));
    assert_eq!(log.email, Some("newmember@company.com".to_string()));
    assert!(log.success);
}

#[tokio::test]
async fn test_security_logging_on_invitation_accepted() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let (_, register_response) = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "boss@company.com",
            "first_name": "Boss",
            "last_name": "User",
            "password": "BossPass123!",
            "company_name": "Accept Company",
            "company_address": "654 Accept Blvd"
        })),
        None,
    )
    .await;

    let admin_token = register_response["token"].as_str().unwrap();

    make_request(
        &mut app,
        "POST",
        "/auth/invitations/send",
        Some(json!({
            "email": "employee@company.com"
        })),
        Some(admin_token),
    )
    .await;

    let invitation_token: String =
        sqlx::query_scalar("SELECT token FROM invitations WHERE email = ?")
            .bind("employee@company.com")
            .fetch_one(&pool)
            .await
            .unwrap();

    let (status, _) = make_request(
        &mut app,
        "POST",
        "/auth/invitations/accept",
        Some(json!({
            "token": invitation_token,
            "first_name": "New",
            "last_name": "Employee",
            "password": "EmployeePass123!"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);

    let employee = db::get_user_by_email(&pool, "employee@company.com")
        .await
        .unwrap()
        .unwrap();
    let employee_id = &employee.id;
    let logs = db::get_security_logs_by_user(&pool, employee_id, 10)
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].event_type, "invitation_accepted");
    assert_eq!(logs[0].user_id, Some(employee_id.to_string()));
    assert_eq!(logs[0].email, Some("employee@company.com".to_string()));
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_security_logs_order_by_time() {
    let (mut app, temp) = setup_test_app().await;
    let db_path = temp.path().to_str().unwrap();
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await.unwrap();

    let _ = make_request(
        &mut app,
        "POST",
        "/auth/register",
        Some(json!({
            "email": "timetest@example.com",
            "first_name": "Time",
            "last_name": "Test",
            "password": "TimePass123!",
            "company_name": "Time Company",
            "company_address": "999 Time Ave"
        })),
        None,
    )
    .await;

    make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "timetest@example.com",
            "password": "TimePass123!"
        })),
        None,
    )
    .await;

    make_request(
        &mut app,
        "POST",
        "/auth/login",
        Some(json!({
            "email": "timetest@example.com",
            "password": "WrongPass123!"
        })),
        None,
    )
    .await;

    let user = db::get_user_by_email(&pool, "timetest@example.com")
        .await
        .unwrap()
        .unwrap();
    let user_id = &user.id;
    let logs = db::get_security_logs_by_user(&pool, user_id, 10)
        .await
        .unwrap();

    assert!(logs.len() >= 3);

    for i in 0..logs.len() - 1 {
        assert!(logs[i].created_at >= logs[i + 1].created_at);
    }
}
