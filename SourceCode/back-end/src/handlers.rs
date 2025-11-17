use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::Duration;
use crate::{
    auth::{hash_password, verify_password, validate_email, validate_password_policy, generate_invitation_token, JwtConfig},
    db,
    middleware::AuthToken,
    AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub company_name: String,
    pub company_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationRequest {
    pub token: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<String>,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub expires_at: String,
}

pub async fn register_company_admin(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already exists" })),
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to process password" }))))?;

    let user = db::create_user(
        &state.sqlite,
        payload.email.clone(),
        payload.first_name.clone(),
        payload.last_name.clone(),
        password_hash,
        None,
        db::UserRole::Admin,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create user" }))))?;

    let company = db::create_company(&state.sqlite, payload.company_name, payload.company_address)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create company" }))))?;

    let _ = sqlx::query("UPDATE users SET company_id = ? WHERE id = ?")
        .bind(&company.id)
        .bind(&user.id)
        .execute(&state.sqlite)
        .await;

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" }))))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                company_id: Some(company.id),
                role: user.role,
            },
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing email or password" })),
        ));
    }

    let user = db::get_user_by_email(&state.sqlite, &payload.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ))?;

    let password_valid = verify_password(&payload.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Authentication failed" }))))?;

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid email or password" })),
        ));
    }

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" }))))?;

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

pub async fn get_current_user(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
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

pub async fn invite_user(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<InviteUserRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), (StatusCode, Json<serde_json::Value>)> {
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

    let admin_user = db::get_user_by_id(&state.sqlite, &claims.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    if !admin_user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company admin can invite users" })),
        ));
    }

    let company_id = admin_user.company_id.ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let _ = db::get_company_by_id(&state.sqlite, &company_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Company not found" })),
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
            (StatusCode::CONFLICT, Json(json!({ "error": "User already invited" })))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create invitation" })))
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            id: invitation.id,
            email: invitation.email,
            expires_at: invitation.expires_at,
        }),
    ))
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" }))))?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already has an account" })),
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to process password" }))))?;

    let user = db::create_user(
        &state.sqlite,
        invitation.email.clone(),
        payload.first_name,
        payload.last_name,
        password_hash,
        Some(invitation.company_id.clone()),
        db::UserRole::Member,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create user" }))))?;

    db::accept_invitation(&state.sqlite, &invitation.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to accept invitation" }))))?;

    let jwt_config = JwtConfig::new(get_jwt_secret());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to generate token" }))))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                company_id: Some(invitation.company_id),
                role: user.role,
            },
        }),
    ))
}

pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "logsmart_secret_key_for_testing".to_string())
}
