use crate::{
    AppState, db,
    dto::{
        AddTokenResponse, AuthResponse, ErrorResponse, ListPasskeysResponse,
        PasskeyAuthenticationFinishRequest, PasskeyAuthenticationStartRequest,
        PasskeyAuthenticationStartResponse, PasskeyRegistrationFinishRequest,
        PasskeyRegistrationStartRequest, PasskeyRegistrationStartResponse, UserResponse,
    },
    jwt_manager::JwtManager,
    middleware::AuthToken,
    utils::{extract_ip_from_headers_and_addr, extract_user_agent},
};
use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD};
use serde_json::json;
use uuid::Uuid;
use webauthn_rs::prelude::*;

#[utoipa::path(
    post,
    path = "/auth/passkey/register/start",
    request_body = PasskeyRegistrationStartRequest,
    responses(
        (status = 200, description = "Passkey registration started", body = PasskeyRegistrationStartResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn start_passkey_registration(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegistrationStartRequest>,
) -> Result<Json<PasskeyRegistrationStartResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    let user_unique_id = Uuid::parse_str(&user.id).unwrap_or_else(|_| Uuid::new_v4());

    let existing_passkeys = db::get_passkeys_by_user(&state.postgres, &user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching passkeys: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    let exclude_credentials: Vec<CredentialID> = existing_passkeys
        .into_iter()
        .filter_map(|pk| {
            let p: Passkey = serde_json::from_str(&pk.public_key).ok()?;
            Some(p.cred_id().clone())
        })
        .collect();

    let (ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(
            user_unique_id,
            &user.email,
            &user.first_name,
            Some(exclude_credentials),
        )
        .map_err(|e| {
            tracing::error!("WebAuthn error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to start registration" })),
            )
        })?;

    let auth_id = Uuid::new_v4().to_string();
    state
        .passkey_reg_state
        .insert(auth_id.clone(), (reg_state, payload.name));

    Ok(Json(PasskeyRegistrationStartResponse {
        options: serde_json::to_value(&ccr.public_key).unwrap(),
        auth_id,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/register/finish",
    request_body = PasskeyRegistrationFinishRequest,
    responses(
        (status = 200, description = "Passkey registered successfully", body = AddTokenResponse),
        (status = 400, description = "Invalid registration data", body = ErrorResponse),
        (status = 404, description = "Registration session not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn finish_passkey_registration(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegistrationFinishRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let split_auth_id: Vec<&str> = payload.auth_id.split('|').collect();
    let auth_id_key = split_auth_id[0];

    let (_, (reg_state, stored_name)) = state.passkey_reg_state.remove(auth_id_key).ok_or((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Registration session expired or invalid" })),
    ))?;

    let passkey_name = if !stored_name.is_empty() {
        stored_name
    } else {
        "Passkey".to_string()
    };

    let req: RegisterPublicKeyCredential =
        serde_json::from_value(payload.credential).map_err(|e| {
            tracing::error!("Invalid credential format: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid credential format" })),
            )
        })?;

    let passkey = state
        .webauthn
        .finish_passkey_registration(&req, &reg_state)
        .map_err(|e| {
            tracing::error!("WebAuthn registration finish error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Failed to verify credential" })),
            )
        })?;

    let credential_id_str = BASE64_URL_SAFE_NO_PAD.encode(passkey.cred_id());
    let public_key_json = serde_json::to_string(&passkey).map_err(|e| {
        tracing::error!("Serialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save passkey"})),
        )
    })?;

    db::create_passkey(
        &state.postgres,
        &claims.user_id,
        credential_id_str,
        public_key_json,
        passkey_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error saving passkey: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database error" })),
        )
    })?;

    Ok(Json(
        json!({ "message": "Passkey registered successfully" }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/start",
    request_body = PasskeyAuthenticationStartRequest,
    responses(
        (status = 200, description = "Passkey authentication started", body = PasskeyAuthenticationStartResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn start_passkey_login(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyAuthenticationStartRequest>,
) -> Result<Json<PasskeyAuthenticationStartResponse>, (StatusCode, Json<serde_json::Value>)> {
    let email = payload.email.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "Email is required" })),
    ))?;

    let user = db::get_user_by_email(&state.postgres, &email)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ))?;

    let passkeys = db::get_passkeys_by_user(&state.postgres, &user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    if passkeys.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No passkeys found for this user" })),
        ));
    }

    let user_passkeys: Vec<Passkey> = passkeys
        .into_iter()
        .filter_map(|pk| serde_json::from_str(&pk.public_key).ok())
        .collect();

    let (rcr, auth_state) = state
        .webauthn
        .start_passkey_authentication(&user_passkeys)
        .map_err(|e| {
            tracing::error!("WebAuthn error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to start authentication" })),
            )
        })?;

    let auth_id = Uuid::new_v4().to_string();
    state
        .passkey_auth_state
        .insert(auth_id.clone(), (user.id, auth_state));

    Ok(Json(PasskeyAuthenticationStartResponse {
        options: serde_json::to_value(&rcr.public_key).unwrap(),
        auth_id,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/finish",
    request_body = PasskeyAuthenticationFinishRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid authentication data", body = ErrorResponse),
        (status = 404, description = "Authentication session not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn finish_passkey_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<PasskeyAuthenticationFinishRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let (_, (user_id, auth_state)) = state.passkey_auth_state.remove(&payload.auth_id).ok_or((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Authentication session expired or invalid" })),
    ))?;

    let req: PublicKeyCredential = serde_json::from_value(payload.credential).map_err(|e| {
        tracing::error!("Invalid credential format: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid credential format" })),
        )
    })?;

    let auth_result = state
        .webauthn
        .finish_passkey_authentication(&req, &auth_state)
        .map_err(|e| {
            tracing::error!("WebAuthn auth finish error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Failed to verify credential" })),
            )
        })?;

    let cred_id_str = BASE64_URL_SAFE_NO_PAD.encode(auth_result.cred_id());

    let passkey_record = db::get_passkey_by_credential_id(&state.postgres, &cred_id_str)
        .await
        .unwrap_or(None);

    if let Some(pk) = passkey_record {
        let _ =
            db::update_passkey_usage(&state.postgres, &pk.id, auth_result.counter() as i64).await;
    }

    let user = db::get_user_by_id(&state.postgres, &user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "User associated with passkey not found" })),
        ))?;

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    let token = JwtManager::get_config()
        .generate_token(user.id.clone(), 24)
        .map_err(|e| {
            tracing::error!("Token creation error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create token" })),
            )
        })?;

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age={}",
        token,
        60 * 60 * 24 * 7
    );

    let mut response = Json(AuthResponse {
        token: token.clone(),
        user: UserResponse {
            email: user.email.clone(),
            first_name: user.first_name,
            last_name: user.last_name,
            company_name: user.company_name,
            role: user.role,
        },
    })
    .into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie).map_err(|e| {
            tracing::error!("Failed to set login cookie: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to set authentication cookie" })),
            )
        })?,
    );

    let _ = db::log_security_event(
        &state.postgres,
        "login_passkey".to_string(),
        Some(user.id),
        Some(user.email.clone()),
        Some(ip_address),
        user_agent,
        Some("Passkey login successful".to_string()),
        true,
    )
    .await;

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/auth/passkeys",
    responses(
        (status = 200, description = "List of passkeys", body = ListPasskeysResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn list_passkeys(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ListPasskeysResponse>, (StatusCode, Json<serde_json::Value>)> {
    let passkeys = db::get_passkeys_by_user(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching passkeys: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    let dtos = passkeys.into_iter().map(Into::into).collect();

    Ok(Json(ListPasskeysResponse { passkeys: dtos }))
}

#[utoipa::path(
    delete,
    path = "/auth/passkeys/{passkey_id}",
    params(
        ("passkey_id" = String, Path, description = "Passkey ID to delete")
    ),
    responses(
        (status = 200, description = "Passkey deleted", body = AddTokenResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn delete_passkey(
    Path(passkey_id): Path<String>,
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    db::delete_passkey(&state.postgres, &passkey_id, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting passkey: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    Ok(Json(json!({ "message": "Passkey deleted successfully" })))
}
