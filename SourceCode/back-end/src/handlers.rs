use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::Duration;
use uuid::Uuid;
use utoipa::ToSchema;
use crate::{
    AppState, auth::{JwtConfig, generate_invitation_token, hash_password, validate_email, validate_password_policy, verify_password}, db, email, middleware::AuthToken
};

fn extract_ip_from_headers_and_addr(headers: &HeaderMap, addr: &std::net::SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string())
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "SecurePass123!")]
    pub password: String,
    #[schema(example = "Example Corp")]
    pub company_name: String,
    #[schema(example = "123 Main St, City, Country")]
    pub company_address: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct VerifyTokenRequest {
    #[schema(example = "jwt-token-here")]
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InviteUserRequest {
    #[schema(example = "newmember@example.com")]
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    #[schema(example = "invitation-token-here")]
    pub token: String,
    #[schema(example = "Alice")]
    pub first_name: String,
    #[schema(example = "Smith")]
    pub last_name: String,
    #[schema(example = "MemberPass123!")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JwtVerifyResponse {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<String>,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/auth/verify",
    request_body = VerifyTokenRequest,
    responses(
        (status = 200, description = "Token is valid", body = JwtVerifyResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<Json<JwtVerifyResponse>, (StatusCode, Json<serde_json::Value>)> {
    let jwt_config = JwtConfig::new(get_jwt_secret());
    let claims = jwt_config
        .validate_token(&payload.token)
        .map_err(|e| {
            tracing::error!("Token verification failed: {:?}", e);
            (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid or expired token" })))
        })?;
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user during token verification: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    Ok(Json(JwtVerifyResponse {
        email: user.email,
    }))
}


#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Company admin registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn register_company_admin(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();
    
    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.email.is_empty() || payload.first_name.is_empty() 
        || payload.last_name.is_empty() || payload.password.is_empty() 
        || payload.company_name.is_empty() || payload.company_address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields" })),
        ));
    }

    if let Err(e) = validate_email(&payload.email) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    if let Err(e) = validate_password_policy(&payload.password) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    if db::get_user_by_email(&state.sqlite, &payload.email)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking existing user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already exists" })),
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to process password" })))
        })?;

    let mut tx = state.sqlite.begin()
        .await
        .map_err(|e| {
            tracing::error!("Failed to begin transaction: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database transaction error" })))
        })?;

    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let role_str = db::UserRole::Admin.to_string();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&password_hash)
    .bind(&role_str)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user in transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create user" })))
    })?;

    let company_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, address, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&company_id)
    .bind(&payload.company_name)
    .bind(&payload.company_address)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create company in transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create company" })))
    })?;

    sqlx::query("UPDATE users SET company_id = ? WHERE id = ?")
        .bind(&company_id)
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to link user to company: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to link user to company" })))
        })?;

    tx.commit()
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit registration transaction: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to commit transaction" })))
        })?;

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user_id.clone(), 24)
        .map_err(|e| {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" })))
        })?;

    let _ = db::log_security_event(
        &state.sqlite,
        "registration".to_string(),
        Some(user_id.clone()),
        Some(payload.email.clone()),
        Some(ip_address),
        user_agent,
        Some(format!("Company admin registered: {}", payload.company_name)),
        true,
    )
    .await;

    state.metrics.increment_registrations();
    state.metrics.increment_successful_requests();
    tracing::info!("Registration successful for user: {}", user_id);

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse {
                id: user_id,
                email: payload.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                company_id: Some(company_id),
                role: role_str,
            },
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();
    state.metrics.increment_login_attempts();
    
    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing email or password" })),
        ));
    }

    let user = db::get_user_by_email(&state.sqlite, &payload.email)
        .await
        .map_err(|e| {
            tracing::error!("Database error during login lookup: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?;
    
    if user.is_none() {
        let _ = db::log_security_event(
            &state.sqlite,
            "login_failed".to_string(),
            None,
            Some(payload.email.clone()),
            Some(ip_address.clone()),
            user_agent.clone(),
            Some("User not found".to_string()),
            false,
        )
        .await;

        state.metrics.increment_login_failures();
        state.metrics.increment_failed_requests();

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ));
    }

    let user = user.unwrap();

    let password_valid = verify_password(&payload.password, &user.password_hash)
        .map_err(|e| {
            tracing::error!("Password verification error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Authentication failed" })))
        })?;

    if !password_valid {
        let _ = db::log_security_event(
            &state.sqlite,
            "login_failed".to_string(),
            Some(user.id.clone()),
            Some(payload.email.clone()),
            Some(ip_address.clone()),
            user_agent.clone(),
            Some("Invalid password".to_string()),
            false,
        )
        .await;

        state.metrics.increment_login_failures();
        state.metrics.increment_failed_requests();

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ));
    }

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .map_err(|e| {
            tracing::error!("Failed to generate login JWT token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" })))
        })?;

    let _ = db::log_security_event(
        &state.sqlite,
        "login_success".to_string(),
        Some(user.id.clone()),
        Some(payload.email.clone()),
        Some(ip_address),
        user_agent,
        None,
        true,
    )
    .await;

    state.metrics.increment_login_successes();
    state.metrics.increment_successful_requests();
    tracing::info!("Login successful for user: {}", user.id);

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            company_id: user.company_id,
            role: user.role,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn get_current_user(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching current user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        company_id: user.company_id,
        role: user.role,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/invitations/send",
    request_body = InviteUserRequest,
    responses(
        (status = 201, description = "Invitation sent successfully", body = InvitationResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 409, description = "User already invited or exists", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn invite_user(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<InviteUserRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();
    
    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Email is required" })),
        ));
    }

    if let Err(e) = validate_email(&payload.email) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String)>(
        r#"
        SELECT u.id, u.email, u.first_name, u.last_name, u.password_hash, u.company_id, u.role
        FROM users u
        INNER JOIN companies c ON u.company_id = c.id
        WHERE u.id = ? AND u.role = 'admin'
        "#,
    )
    .bind(&claims.user_id)
    .fetch_optional(&state.sqlite)
    .await
    .map_err(|e| {
        tracing::error!("Database error verifying admin user for invitation: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
    })?;

    let (admin_id, admin_email, _admin_first_name, _admin_last_name, _admin_password_hash, company_id_opt, _admin_role) = result.ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "Only company admin can invite users" })),
    ))?;

    let company_id = company_id_opt.ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let token = generate_invitation_token();
    let expires_at = (chrono::Utc::now() + Duration::days(7)).to_rfc3339();

    let invitation = db::create_invitation(
        &state.sqlite,
        company_id,
        payload.email.clone(),
        token,
        expires_at.clone(),
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            tracing::warn!("Duplicate invitation attempt for email: {}", payload.email);
            (StatusCode::CONFLICT, Json(json!({ "error": "User already invited" })))
        } else {
            tracing::error!("Failed to create invitation: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create invitation" })))
        }
    })?;

    let invite_link = format!(
        "{}/accept-invitation?token={}",
        "https://logsmart.app",
        invitation.token
    );
    email::send_invitation_email(&payload.email, &invite_link, "Your Company Name")
        .await
        .map_err(|e| {
            tracing::error!("Failed to send invitation email: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to send invitation email" })))
        })?;

    let _ = db::log_security_event(
        &state.sqlite,
        "invitation_sent".to_string(),
        Some(admin_id.clone()),
        Some(payload.email.clone()),
        Some(ip_address),
        user_agent,
        Some(format!("Invitation sent by {}", admin_email)),
        true,
    )
    .await;

    state.metrics.increment_invitations_sent();
    state.metrics.increment_successful_requests();
    tracing::info!("Invitation sent by {} to {}", admin_email, payload.email);

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            id: invitation.id,
            email: invitation.email,
            expires_at: invitation.expires_at,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/invitations/accept",
    request_body = AcceptInvitationRequest,
    responses(
        (status = 201, description = "Invitation accepted successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Invalid or expired invitation token", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Invitations"
)]
pub async fn accept_invitation(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();
    
    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.token.is_empty() || payload.first_name.is_empty() 
        || payload.last_name.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields" })),
        ));
    }

    if let Err(e) = validate_password_policy(&payload.password) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    let invitation = db::get_invitation_by_token(&state.sqlite, &payload.token)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching invitation by token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired invitation" })),
        ))?;

    let now = chrono::Utc::now();
    let expires_at = chrono::DateTime::parse_from_rfc3339(&invitation.expires_at)
        .ok()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid invitation expiration date" })),
        ))?;

    if now > expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invitation has expired" })),
        ));
    }

    if db::get_user_by_email(&state.sqlite, &invitation.email)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking existing user during invitation acceptance: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already has an account" })),
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|e| {
            tracing::error!("Failed to hash password during invitation acceptance: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to process password" })))
        })?;

    let mut tx = state.sqlite.begin()
        .await
        .map_err(|e| {
            tracing::error!("Failed to begin transaction for invitation acceptance: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database transaction error" })))
        })?;

    let user_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let role_str = db::UserRole::Member.to_string();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&invitation.email)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&password_hash)
    .bind(&invitation.company_id)
    .bind(&role_str)
    .bind(&created_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user in invitation acceptance transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create user" })))
    })?;

    let accept_time = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE invitations
        SET accepted_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&accept_time)
    .bind(&invitation.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to mark invitation as accepted: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to accept invitation" })))
    })?;

    tx.commit()
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit invitation acceptance transaction: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to commit transaction" })))
        })?;

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user_id.clone(), 24)
        .map_err(|e| {
            tracing::error!("Failed to generate JWT token for invitation acceptance: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" })))
        })?;

    let _ = db::log_security_event(
        &state.sqlite,
        "invitation_accepted".to_string(),
        Some(user_id.clone()),
        Some(invitation.email.clone()),
        Some(ip_address),
        user_agent,
        Some(format!("Member joined company {}", invitation.company_id)),
        true,
    )
    .await;

    state.metrics.increment_invitations_accepted();
    state.metrics.increment_successful_requests();
    tracing::info!("Invitation accepted by user: {}", user_id);

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse {
                id: user_id,
                email: invitation.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                company_id: Some(invitation.company_id),
                role: role_str,
            },
        }),
    ))
}

pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "logsmart_secret_key_for_testing".to_string())
}
