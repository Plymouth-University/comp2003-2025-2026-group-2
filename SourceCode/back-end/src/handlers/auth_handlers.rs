use crate::utils::{extract_ip_from_headers_and_addr, extract_user_agent};
use crate::{
    AppState,
    auth::{validate_email, validate_password_policy},
    db,
    dto::{
        AuthResponse, ErrorResponse, JwtVerifyResponse, LoginRequest, PasswordResetResponse,
        RegisterRequest, RequestPasswordResetRequest, ResetPasswordRequest, UpdateProfileRequest,
        UserResponse, VerifyTokenRequest,
    },
    jwt_manager::JwtManager,
    middleware::AuthToken,
    services,
    utils::AuditLogger,
};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{self, HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use serde_json::json;

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
    let jwt_config = JwtManager::get_config();
    let claims = jwt_config.validate_token(&payload.token).map_err(|e| {
        tracing::error!("Token verification failed: {:?}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired token" })),
        )
    })?;
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error fetching user during token verification: {:?}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    Ok(Json(JwtVerifyResponse { email: user.email }))
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
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.email.is_empty()
        || payload.first_name.is_empty()
        || payload.last_name.is_empty()
        || payload.password.is_empty()
        || payload.company_name.is_empty()
        || payload.company_address.is_empty()
    {
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already exists" })),
        ));
    }

    let (token, user_id, role_str) = services::AuthService::register_admin(
        &state.sqlite,
        &payload.email,
        &payload.first_name,
        &payload.last_name,
        &payload.password,
        &payload.company_name,
        &payload.company_address,
        Some(ip_address),
        user_agent,
    )
    .await
    .map_err(|e| {
        state.metrics.increment_failed_requests();
        tracing::error!("Registration failed: {:?}", e);
        (e.0, Json(e.1))
    })?;

    state.metrics.increment_registrations();
    state.metrics.increment_successful_requests();
    tracing::info!("Registration successful for user: {}", user_id);

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age={}",
        token,
        60 * 60 * 24 * 7
    );

    let mut response = (
        StatusCode::CREATED,
        Json(AuthResponse {
            token: token.clone(),
            user: UserResponse {
                email: payload.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                company_name: Some(payload.company_name),
                role: role_str,
            },
        }),
    )
        .into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        http::HeaderValue::from_str(&cookie).map_err(|e| {
            tracing::error!("Failed to set registration cookie: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to set authentication cookie" })),
            )
        })?,
    );

    Ok(response)
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
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    let (token, user) = services::AuthService::verify_credentials(
        &state.sqlite,
        &payload.email,
        &payload.password,
        Some(ip_address),
        user_agent,
    )
    .await
    .map_err(|e| {
        state.metrics.increment_login_failures();
        state.metrics.increment_failed_requests();
        (e.0, Json(e.1))
    })?;

    state.metrics.increment_login_successes();
    state.metrics.increment_successful_requests();
    tracing::info!("Login successful for user: {}", user.id);

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age={}",
        token,
        60 * 60 * 24 * 7
    );

    let mut response = Json(AuthResponse {
        token: token.clone(),
        user: UserResponse {
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            company_name: user.company_name,
            role: user.role,
        },
    })
    .into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        http::header::HeaderValue::from_str(&cookie).map_err(|e| {
            tracing::error!("Failed to set login cookie: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to set authentication cookie" })),
            )
        })?,
    );

    Ok(response)
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    put,
    path = "/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn update_profile(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::update_user_profile(
        &state.sqlite,
        &claims.user_id,
        payload.first_name,
        payload.last_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update profile: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to update profile" })),
        )
    })?;

    AuditLogger::log_profile_updated(&state.sqlite, claims.user_id, user.email.clone()).await;

    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    path = "/auth/password/request-reset",
    request_body = RequestPasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = PasswordResetResponse),
        (status = 400, description = "Invalid email", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<RequestPasswordResetRequest>,
) -> Result<Json<PasswordResetResponse>, (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();

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

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    services::AuthService::request_password_reset(
        &state.sqlite,
        &payload.email,
        Some(ip_address),
        user_agent,
    )
    .await
    .map_err(|(status, err)| {
        state.metrics.increment_failed_requests();
        (status, Json(err))
    })?;

    state.metrics.increment_successful_requests();

    Ok(Json(PasswordResetResponse {
        message: "If an account exists with this email, a password reset link has been sent."
            .to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/password/reset",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = PasswordResetResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<PasswordResetResponse>, (StatusCode, Json<serde_json::Value>)> {
    services::AuthService::reset_password(&state.sqlite, &payload.token, &payload.new_password)
        .await
        .map_err(|e| {
            tracing::error!("Password reset failed: {:?}", e);
            (e.0, Json(e.1))
        })?;

    state.metrics.increment_successful_requests();

    Ok(Json(PasswordResetResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}
