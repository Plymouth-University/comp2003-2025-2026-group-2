use crate::{
    AppState, db,
    dto::{
        AddTokenResponse, AuthResponse, ErrorResponse, ListPasskeysResponse,
        PasskeyAuthenticationFinishRequest, PasskeyAuthenticationStartRequest,
        PasskeyAuthenticationStartResponse, PasskeyRegistrationFinishRequest,
        PasskeyRegistrationStartRequest, PasskeyRegistrationStartResponse,
    },
    jwt_manager::JwtManager,
    middleware::{AnyAuthUser, AuditRequestContext},
    utils::{AuditLogger, extract_ip_from_headers_and_addr, extract_user_agent},
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
/// Starts the process of registering a new `WebAuthn` passkey.
///
/// # Errors
/// Returns an error if the user is not found, or if WebAuthn/database operations fail.
pub async fn start_passkey_registration(
    AnyAuthUser(_claims, user): AnyAuthUser,
    AuditRequestContext(audit_ctx): AuditRequestContext,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegistrationStartRequest>,
) -> Result<Json<PasskeyRegistrationStartResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_unique_id = Uuid::parse_str(&user.id).map_err(|e| {
        tracing::error!(
            "Invalid user UUID in database: {:?}, error: {:?}",
            user.id,
            e
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Invalid user ID format" })),
        )
    })?;

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

    let mut options = serde_json::to_value(&ccr.public_key).map_err(|e| {
        tracing::error!("Failed to serialize public key: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    if let Some(obj) = options.as_object_mut() {
        if let Some(auth_sel) = obj.get_mut("authenticatorSelection") {
            if let Some(auth_sel_obj) = auth_sel.as_object_mut() {
                auth_sel_obj.insert("residentKey".to_string(), json!("required"));
                auth_sel_obj.insert("requireResidentKey".to_string(), json!(true));
                auth_sel_obj.insert("userVerification".to_string(), json!("required"));
            }
        } else {
            obj.insert(
                "authenticatorSelection".to_string(),
                json!({
                    "residentKey": "required",
                    "requireResidentKey": true,
                    "userVerification": "required"
                }),
            );
        }
    }

    let auth_id = Uuid::new_v4().to_string();
    let challenge_json = serde_json::to_string(&reg_state).map_err(|e| {
        tracing::error!("Serialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to serialize challenge" })),
        )
    })?;

    db::create_passkey_session(
        &state.postgres,
        &auth_id,
        "reg",
        Some(user.id.clone()),
        challenge_json,
        Some(payload.name),
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to store registration session" })),
        )
    })?;

    AuditLogger::log(
        &state.postgres,
        "passkey_registration_started",
        Some(user.id.clone()),
        Some(user.email.clone()),
        crate::audit_ctx!(&audit_ctx, actor: &user),
        Some("Passkey registration challenge created".to_string()),
        true,
    )
    .await;

    Ok(Json(PasskeyRegistrationStartResponse { options, auth_id }))
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
/// Finalizes the `WebAuthn` passkey registration.
///
/// # Errors
/// Returns an error if the session is invalid, `WebAuthn` verification fails, or database update fails.
pub async fn finish_passkey_registration(
    AnyAuthUser(_claims, user): AnyAuthUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegistrationFinishRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let split_auth_id: Vec<&str> = payload.auth_id.split('|').collect();
    let auth_id_key = split_auth_id[0];

    // Retrieve and then delete the session from DB (one-time use)
    let session = db::get_passkey_session(&state.postgres, auth_id_key)
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
            Json(json!({ "error": "Registration session expired or invalid" })),
        ))?;

    // Ensure it's the right type
    if session.session_type != "reg" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid session type" })),
        ));
    }

    // Delete session immediately to prevent replay
    let _ = db::delete_passkey_session(&state.postgres, auth_id_key).await;

    let reg_state: PasskeyRegistration = serde_json::from_str(&session.challenge).map_err(|e| {
        tracing::error!("Deserialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to restore registration state" })),
        )
    })?;

    let stored_name = session.meta.unwrap_or_default();

    let passkey_name = if stored_name.is_empty() {
        "Passkey".to_string()
    } else {
        stored_name
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
    tracing::info!(
        "Registering passkey with credential_id: {}",
        credential_id_str
    );

    let public_key_json = serde_json::to_string(&passkey).map_err(|e| {
        tracing::error!("Serialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save passkey"})),
        )
    })?;

    db::create_passkey(
        &state.postgres,
        &user.id,
        credential_id_str.clone(),
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

    let _ = db::log_security_event(
        &state.postgres,
        "passkey_registration_completed".to_string(),
        Some(user.id.clone()),
        Some(user.email.clone()),
        None,
        None,
        db::SecurityLogMeta {
            actor_role: Some(user.role.to_string()),
            company_id: user.company_id.clone(),
            request_path: Some("/auth/passkey/register/finish".to_string()),
            request_method: Some("POST".to_string()),
            ..db::SecurityLogMeta::default()
        },
        Some("Passkey registration completed".to_string()),
        true,
    )
    .await;

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
/// Starts the passkey login process for a specific user.
///
/// # Errors
/// Returns an error if the user/passkeys are not found, or if WebAuthn/database operations fail.
pub async fn start_passkey_login(
    AuditRequestContext(audit_ctx): AuditRequestContext,
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
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found" })),
            )
        })?;

    if user.deleted_at.is_some() || user.company_deleted_at.is_some() {
        AuditLogger::log(
            &state.postgres,
            "passkey_login_started",
            Some(user.id.clone()),
            Some(user.email.clone()),
            crate::audit_ctx!(&audit_ctx, actor: &user),
            Some("User account is deactivated".to_string()),
            false,
        )
        .await;
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found" })),
        ));
    }

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
        AuditLogger::log(
            &state.postgres,
            "passkey_login_started",
            Some(user.id.clone()),
            Some(user.email.clone()),
            crate::audit_ctx!(&audit_ctx, actor: &user),
            Some("No passkeys found for user".to_string()),
            false,
        )
        .await;
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
    let challenge_json = serde_json::to_string(&auth_state).map_err(|e| {
        tracing::error!("Serialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to serialize challenge" })),
        )
    })?;

    db::create_passkey_session(
        &state.postgres,
        &auth_id,
        "auth",
        Some(user.id.clone()),
        challenge_json,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to store authentication session" })),
        )
    })?;

    AuditLogger::log(
        &state.postgres,
        "passkey_login_started",
        Some(user.id.clone()),
        Some(user.email.clone()),
        crate::audit_ctx!(&audit_ctx, actor: &user),
        Some("Passkey login challenge created".to_string()),
        true,
    )
    .await;

    Ok(Json(PasskeyAuthenticationStartResponse {
        options: serde_json::to_value(&rcr.public_key).map_err(|e| {
            tracing::error!("Failed to serialize public key: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?,
        auth_id,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/discoverable/start",
    responses(
        (status = 200, description = "Discoverable passkey authentication started", body = PasskeyAuthenticationStartResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
/// Starts a discoverable passkey login (resident key) process.
///
/// # Errors
/// Returns an error if `WebAuthn` or database operations fail.
pub async fn start_discoverable_passkey_login(
    AuditRequestContext(audit_ctx): AuditRequestContext,
    State(state): State<AppState>,
) -> Result<Json<PasskeyAuthenticationStartResponse>, (StatusCode, Json<serde_json::Value>)> {
    let (rcr, auth_state) = state
        .webauthn
        .start_discoverable_authentication()
        .map_err(|e| {
            tracing::error!("WebAuthn discoverable auth error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to start discoverable authentication" })),
            )
        })?;

    let auth_id = Uuid::new_v4().to_string();
    // Store with empty user_id or None - will be extracted from credential during finish
    let challenge_json = serde_json::to_string(&auth_state).map_err(|e| {
        tracing::error!("Serialization error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to serialize challenge" })),
        )
    })?;

    db::create_passkey_session(
        &state.postgres,
        &auth_id,
        "disc_auth",
        None,
        challenge_json,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to store discoverable authentication session" })),
        )
    })?;

    AuditLogger::log(
        &state.postgres,
        "passkey_login_discoverable_started",
        None,
        None,
        crate::audit_ctx!(&audit_ctx),
        Some("Discoverable passkey challenge created".to_string()),
        true,
    )
    .await;

    Ok(Json(PasskeyAuthenticationStartResponse {
        options: serde_json::to_value(&rcr.public_key).map_err(|e| {
            tracing::error!("Failed to serialize public key: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?,
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
/// Finalizes the passkey login for a specific user.
///
/// # Errors
/// Returns an error if the session is invalid, `WebAuthn` verification fails, or authentication fails.
pub async fn finish_passkey_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<PasskeyAuthenticationFinishRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Retrieve and delete session
    let session = db::get_passkey_session(&state.postgres, &payload.auth_id)
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
            Json(json!({ "error": "Authentication session expired or invalid" })),
        ))?;

    if session.session_type != "auth" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid session type" })),
        ));
    }

    let _ = db::delete_passkey_session(&state.postgres, &payload.auth_id).await;

    let user_id = session.user_id.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "User ID missing from session" })),
    ))?;

    let auth_state: PasskeyAuthentication =
        serde_json::from_str(&session.challenge).map_err(|e| {
            tracing::error!("Deserialization error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to restore authentication state" })),
            )
        })?;

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

    if user.company_deleted_at.is_some() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Your company has been deleted. Please contact support." })),
        ));
    }

    if let Some(pk) = passkey_record {
        let _ = db::update_passkey_usage(&state.postgres, &pk.id, i64::from(auth_result.counter()))
            .await;
    }

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    let token = JwtManager::get_config()
        .generate_token(user.id.as_str(), 24)
        .map_err(|e| {
            tracing::error!("Token creation error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create token" })),
            )
        })?;

    let cookie_domain = std::env::var("COOKIE_DOMAIN").unwrap_or_default();
    let domain_attr = if cookie_domain.is_empty() {
        String::new()
    } else {
        format!("; Domain={cookie_domain}")
    };

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}{}",
        token,
        60 * 60 * 24 * 7,
        domain_attr
    );

    let mut response = Json(AuthResponse {
        token: token.clone(),
        user: user.clone().into(),
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
        db::SecurityLogMeta {
            actor_role: Some(user.role.to_string()),
            company_id: user.company_id.clone(),
            request_path: Some("/auth/passkey/login/finish".to_string()),
            request_method: Some("POST".to_string()),
            ..db::SecurityLogMeta::default()
        },
        Some("Passkey login successful".to_string()),
        true,
    )
    .await;

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/auth/passkey/login/discoverable/finish",
    request_body = PasskeyAuthenticationFinishRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid authentication data", body = ErrorResponse),
        (status = 404, description = "Authentication session not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
/// Finalizes a discoverable passkey login.
///
/// # Errors
/// Returns an error if the session is invalid, user identification fails, or `WebAuthn` verification fails.
pub async fn finish_discoverable_passkey_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<PasskeyAuthenticationFinishRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Retrieve and delete session
    let session = db::get_passkey_session(&state.postgres, &payload.auth_id)
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
            Json(json!({ "error": "Authentication session expired or invalid" })),
        ))?;

    if session.session_type != "disc_auth" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid session type" })),
        ));
    }

    let _ = db::delete_passkey_session(&state.postgres, &payload.auth_id).await;

    let auth_state: DiscoverableAuthentication =
        serde_json::from_str(&session.challenge).map_err(|e| {
            tracing::error!("Deserialization error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to restore authentication state" })),
            )
        })?;

    let req: PublicKeyCredential = serde_json::from_value(payload.credential).map_err(|e| {
        tracing::error!("Invalid credential format: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid credential format" })),
        )
    })?;

    // First identify the user from the credential
    let (user_unique_id, cred_id) = state
        .webauthn
        .identify_discoverable_authentication(&req)
        .map_err(|e| {
            tracing::error!("WebAuthn identify error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Failed to identify credential" })),
            )
        })?;

    // Convert user_unique_id (which is Uuid) to string
    let user_id = user_unique_id.to_string();

    // Get the credential ID string for looking up the passkey
    let cred_id_str = BASE64_URL_SAFE_NO_PAD.encode(cred_id);
    tracing::info!("Discoverable login found credential_id: {}", cred_id_str);
    tracing::info!("Discoverable login found user_id: {}", user_id);

    // Look up the passkey from database
    let passkey_record = db::get_passkey_by_credential_id(&state.postgres, &cred_id_str)
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
            Json(json!({ "error": "Passkey not found" })),
        ))?;

    // Deserialize the stored passkey
    let passkey: Passkey = serde_json::from_str(&passkey_record.public_key).map_err(|e| {
        tracing::error!("Failed to deserialize passkey: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to load passkey" })),
        )
    })?;

    // Complete authentication
    let auth_result = state
        .webauthn
        .finish_discoverable_authentication(&req, auth_state, &[passkey.into()])
        .map_err(|e| {
            tracing::error!("WebAuthn auth finish error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Failed to verify credential" })),
            )
        })?;

    // Update passkey usage
    let _ = db::update_passkey_usage(
        &state.postgres,
        &passkey_record.id,
        i64::from(auth_result.counter()),
    )
    .await;

    // Get user from database
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

    if user.company_deleted_at.is_some() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Your company has been deleted. Please contact support." })),
        ));
    }

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    let token = JwtManager::get_config()
        .generate_token(user.id.as_str(), 24)
        .map_err(|e| {
            tracing::error!("Token creation error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create token" })),
            )
        })?;

    let cookie_domain = std::env::var("COOKIE_DOMAIN").unwrap_or_default();
    let domain_attr = if cookie_domain.is_empty() {
        String::new()
    } else {
        format!("; Domain={cookie_domain}")
    };

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}{}",
        token,
        60 * 60 * 24 * 7,
        domain_attr
    );

    let mut response = Json(AuthResponse {
        token: token.clone(),
        user: user.clone().into(),
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
        "login_passkey_discoverable".to_string(),
        Some(user.id),
        Some(user.email.clone()),
        Some(ip_address),
        user_agent,
        db::SecurityLogMeta {
            actor_role: Some(user.role.to_string()),
            company_id: user.company_id.clone(),
            request_path: Some("/auth/passkey/login/discoverable/finish".to_string()),
            request_method: Some("POST".to_string()),
            ..db::SecurityLogMeta::default()
        },
        Some("Discoverable passkey login successful".to_string()),
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
/// Lists all passkeys registered for the current user.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn list_passkeys(
    AnyAuthUser(_claims, user): AnyAuthUser,
    State(state): State<AppState>,
) -> Result<Json<ListPasskeysResponse>, (StatusCode, Json<serde_json::Value>)> {
    let passkeys = db::get_passkeys_by_user(&state.postgres, &user.id)
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
/// Deletes a specific passkey for the current user.
///
/// # Errors
/// Returns an error if the database deletion fails.
pub async fn delete_passkey(
    Path(passkey_id): Path<String>,
    AnyAuthUser(_claims, user): AnyAuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    db::delete_passkey(&state.postgres, &passkey_id, &user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting passkey: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    let _ = db::log_security_event(
        &state.postgres,
        "passkey_deleted".to_string(),
        Some(user.id.clone()),
        Some(user.email.clone()),
        None,
        None,
        db::SecurityLogMeta {
            actor_role: Some(user.role.to_string()),
            company_id: user.company_id.clone(),
            target_user_id: Some(user.id.clone()),
            request_path: Some(format!("/auth/passkeys/{passkey_id}")),
            request_method: Some("DELETE".to_string()),
            ..db::SecurityLogMeta::default()
        },
        Some(format!("Passkey {} deleted", passkey_id)),
        true,
    )
    .await;

    Ok(Json(json!({ "message": "Passkey deleted successfully" })))
}
