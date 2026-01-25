use crate::{
    dto::{ErrorResponse, OAuthCallbackRequest, OAuthInitiateResponse},
    middleware::AuthToken,
    services::oauth_service::OAuthUserInfo,
    utils::{extract_ip_from_headers_and_addr, extract_user_agent, AuditLogger},
    AppState,
};
use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode, HeaderName, HeaderValue},
    response::{IntoResponse, Redirect},
};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;

pub struct OAuthStateStore {
    states: Arc<DashMap<String, (String, bool, chrono::DateTime<chrono::Utc>)>>,
    link_tokens: Arc<DashMap<String, (OAuthUserInfo, chrono::DateTime<chrono::Utc>)>>,
}

impl OAuthStateStore {
    pub fn new() -> Self {
        Self {
            states: Arc::new(DashMap::new()),
            link_tokens: Arc::new(DashMap::new()),
        }
    }

    pub fn store_state(&self, state: String, nonce: String, is_link: bool) {
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);
        self.states.insert(state, (nonce, is_link, expires_at));
        self.cleanup_expired();
    }

    pub fn verify_and_remove(&self, state: &str) -> Option<(String, bool)> {
        self.states.remove(state).and_then(|(_, (nonce, is_link, expires_at))| {
            if chrono::Utc::now() < expires_at {
                Some((nonce, is_link))
            } else {
                None
            }
        })
    }

    pub fn store_link_token(&self, user_info: OAuthUserInfo) -> String {
        use rand::Rng;
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        self.link_tokens.insert(token.clone(), (user_info, expires_at));
        self.cleanup_expired();
        token
    }

    pub fn verify_and_remove_link_token(&self, token: &str) -> Option<OAuthUserInfo> {
        self.link_tokens.remove(token).and_then(|(_, (user_info, expires_at))| {
            if chrono::Utc::now() < expires_at {
                Some(user_info)
            } else {
                None
            }
        })
    }

    fn cleanup_expired(&self) {
        let now = chrono::Utc::now();
        self.states.retain(|_, (_, _, expires_at)| *expires_at > now);
        self.link_tokens.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

impl Default for OAuthStateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthInitiateQuery {
    #[serde(default)]
    mode: Option<String>,
}

#[utoipa::path(
    get,
    path = "/auth/google/initiate",
    params(
        ("mode" = Option<String>, Query, description = "OAuth mode: 'link' or omit for login")
    ),
    responses(
        (status = 200, description = "OAuth flow initiated", body = OAuthInitiateResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn initiate_google_login(
    State(state): State<AppState>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let oauth_client = state.google_oauth.as_ref().ok_or_else(|| {
        tracing::error!("Google OAuth client not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "OAuth not configured" })),
        )
    })?;

    let (auth_url, csrf_state, nonce) = oauth_client.initiate_login();
    let is_link = query.mode.as_deref() == Some("link");

    state.oauth_state_store.store_state(csrf_state, nonce, is_link);

    Ok(Redirect::to(&auth_url).into_response())
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackParams {
    code: String,
    state: String,
}

#[utoipa::path(
    get,
    path = "/auth/google/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Google"),
        ("state" = String, Query, description = "CSRF state token")
    ),
    responses(
        (status = 302, description = "Redirect to dashboard on success or login on error"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Authentication failed", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn google_callback(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<GoogleCallbackParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let oauth_client = state.google_oauth.as_ref().ok_or_else(|| {
        tracing::error!("Google OAuth client not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "OAuth not configured" })),
        )
    })?;

    let (nonce, is_link) = state
        .oauth_state_store
        .verify_and_remove(&params.state)
        .ok_or_else(|| {
            tracing::error!("Invalid or expired OAuth state");
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired state parameter" })),
            )
        })?;

    let ip_address = Some(extract_ip_from_headers_and_addr(&headers, &addr));
    let user_agent = extract_user_agent(&headers);

    let (user_info, _claims) = oauth_client
        .exchange_code(params.code, nonce)
        .await
        .map_err(|(status, value)| (status, Json(value)))?;

    if is_link {
        let link_token = state.oauth_state_store.store_link_token(user_info);
        let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
        let redirect_url = format!(
            "{}/settings?oauth_link_token={}", 
            frontend_url, link_token
        );
        
        return Ok(Redirect::to(&redirect_url).into_response());
    }

    let user = oauth_client
        .get_or_create_user(&state.postgres, user_info, ip_address, user_agent, false)
        .await
        .map_err(|(status, value)| (status, Json(value)))?;

    let token = oauth_client
        .generate_jwt_for_user(user.id.clone())
        .map_err(|(status, value)| (status, Json(value)))?;

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age={}",
        token,
        60 * 60 * 24 * 7
    );

    let redirect_url = format!("{}/dashboard", frontend_url);

    let mut response = Redirect::to(&redirect_url).into_response();
    response.headers_mut().insert(
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_str(&cookie).unwrap()
    );
    
    Ok(response)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthLinkConfirmRequest {
    pub link_token: String,
}

#[utoipa::path(
    post,
    path = "/auth/google/link/confirm",
    request_body = OAuthLinkConfirmRequest,
    responses(
        (status = 200, description = "Google account linked successfully"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Account already linked", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn confirm_google_link(
    State(state): State<AppState>,
    AuthToken(claims): AuthToken,
    Json(payload): Json<OAuthLinkConfirmRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let oauth_client = state.google_oauth.as_ref().ok_or_else(|| {
        tracing::error!("Google OAuth client not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "OAuth not configured" })),
        )
    })?;

    let user_info = state
        .oauth_state_store
        .verify_and_remove_link_token(&payload.link_token)
        .ok_or_else(|| {
            tracing::error!("Invalid or expired OAuth link token");
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired link token" })),
            )
        })?;

    oauth_client
        .link_google_account(&state.postgres, &claims.user_id, user_info)
        .await
        .map_err(|(status, value)| (status, Json(value)))?;

    let user = crate::db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user: {:?}", e);
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

    AuditLogger::log_oauth_account_linked(
        &state.postgres,
        claims.user_id.clone(),
        user.email.clone(),
        "google".to_string(),
    )
    .await;

    Ok(Json(json!({ "message": "Google account linked successfully" })))
}

#[utoipa::path(
    post,
    path = "/auth/google/link",
    request_body = OAuthCallbackRequest,
    responses(
        (status = 200, description = "Google account linked successfully"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Account already linked", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn link_google_account(
    State(state): State<AppState>,
    AuthToken(claims): AuthToken,
    Json(payload): Json<OAuthCallbackRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let oauth_client = state.google_oauth.as_ref().ok_or_else(|| {
        tracing::error!("Google OAuth client not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "OAuth not configured" })),
        )
    })?;

    let (nonce, _is_link) = state
        .oauth_state_store
        .verify_and_remove(&payload.state)
        .ok_or_else(|| {
            tracing::error!("Invalid or expired OAuth state");
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired state parameter" })),
            )
        })?;

    let (user_info, _claims) = oauth_client
        .exchange_code(payload.code, nonce)
        .await
        .map_err(|(status, value)| (status, Json(value)))?;

    oauth_client
        .link_google_account(&state.postgres, &claims.user_id, user_info)
        .await
        .map_err(|(status, value)| (status, Json(value)))?;

    let user = crate::db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user: {:?}", e);
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

    AuditLogger::log_oauth_account_linked(
        &state.postgres,
        claims.user_id.clone(),
        user.email.clone(),
        "google".to_string(),
    )
    .await;

    Ok(Json(json!({ "message": "Google account linked successfully" })))
}
