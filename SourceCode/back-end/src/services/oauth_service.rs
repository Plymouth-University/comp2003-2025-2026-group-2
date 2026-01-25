use crate::{db, jwt_manager::JwtManager, utils::AuditLogger};
use anyhow::Result;
use axum::http::StatusCode;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope, TokenResponse,
    core::{CoreClient, CoreIdTokenClaims, CoreProviderMetadata, CoreResponseType},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Clone)]
pub struct GoogleOAuthClient {
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    provider_metadata: CoreProviderMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub email: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub sub: String,
}

impl GoogleOAuthClient {
    pub async fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        issuer_url: String,
    ) -> Result<Self> {
        let issuer_url = IssuerUrl::new(issuer_url)
            .map_err(|e| anyhow::anyhow!("Failed to create issuer URL: {e}"))?;

        let http_client = openidconnect::reqwest::Client::new();
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to discover Google metadata: {e}"))?;

        let redirect_uri_validated = RedirectUrl::new(redirect_uri.clone())
            .map_err(|e| anyhow::anyhow!("Invalid redirect URI: {e}"))?;

        Ok(Self {
            client_id: ClientId::new(client_id),
            client_secret: ClientSecret::new(client_secret),
            redirect_uri: redirect_uri_validated,
            provider_metadata,
        })
    }

    pub fn initiate_login(&self) -> (String, String, String) {
        let client = CoreClient::from_provider_metadata(
            self.provider_metadata.clone(),
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_uri.clone());

        let (auth_url, csrf_token, nonce) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        (
            auth_url.to_string(),
            csrf_token.secret().clone(),
            nonce.secret().clone(),
        )
    }

    pub async fn exchange_code(
        &self,
        code: String,
        nonce: String,
    ) -> Result<(OAuthUserInfo, CoreIdTokenClaims), (StatusCode, serde_json::Value)> {
        let client = CoreClient::from_provider_metadata(
            self.provider_metadata.clone(),
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_uri.clone());

        let http_client = openidconnect::reqwest::Client::new();

        let token_request = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| {
                tracing::error!("Invalid authorization code: {:?}", e);
                (
                    StatusCode::BAD_REQUEST,
                    json!({ "error": "Invalid authorization code" }),
                )
            })?;

        let token_response = token_request
            .request_async(&http_client)
            .await
            .map_err(|e| {
                tracing::error!("Failed to exchange OAuth code: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to exchange authorization code" }),
                )
            })?;

        let id_token = token_response.id_token().ok_or_else(|| {
            tracing::error!("No ID token in OAuth response");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "No ID token received from Google" }),
            )
        })?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &Nonce::new(nonce))
            .map_err(|e| {
                tracing::error!("Failed to verify ID token: {:?}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    json!({ "error": "Failed to verify ID token" }),
                )
            })?;

        let email = claims
            .email()
            .map(|e| e.as_str().to_string())
            .ok_or_else(|| {
                tracing::error!("No email in ID token claims");
                (
                    StatusCode::BAD_REQUEST,
                    json!({ "error": "Email not provided by Google" }),
                )
            })?;

        let given_name = claims
            .given_name()
            .and_then(|names| names.get(None).map(|name| name.as_str().to_string()))
            .unwrap_or_else(|| "User".to_string());

        let family_name = claims
            .family_name()
            .and_then(|names| names.get(None).map(|name| name.as_str().to_string()))
            .unwrap_or_else(String::new);

        let picture = claims
            .picture()
            .and_then(|pics| pics.get(None).map(|pic| pic.to_string()));

        let sub = claims.subject().to_string();

        Ok((
            OAuthUserInfo {
                email,
                given_name,
                family_name,
                picture,
                sub,
            },
            claims.clone(),
        ))
    }

    pub async fn get_or_create_user(
        &self,
        pool: &PgPool,
        user_info: OAuthUserInfo,
        ip_address: Option<String>,
        user_agent: Option<String>,
        allow_new_account: bool,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        if let Some(existing_user) = db::get_user_by_oauth(pool, "google", &user_info.sub)
            .await
            .map_err(|e| {
                tracing::error!("Database error checking OAuth user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
        {
            AuditLogger::log_oauth_login(
                pool,
                existing_user.id.clone(),
                user_info.email.clone(),
                "google".to_string(),
                true,
                ip_address,
                user_agent,
            )
            .await;
            return Ok(existing_user);
        }

        if let Some(_existing_user) = db::get_user_by_email(pool, &user_info.email)
            .await
            .map_err(|e| {
                tracing::error!("Database error checking email: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
        {
            return Err((
                StatusCode::CONFLICT,
                json!({
                    "error": "An account with this email already exists. Please login with your password or link your Google account in settings.",
                    "existing_account": true
                }),
            ));
        }

        if !allow_new_account {
            return Err((
                StatusCode::FORBIDDEN,
                json!({
                    "error": "No account found. Please create an account first or use an invitation link to join a company.",
                    "requires_invitation": true
                }),
            ));
        }

        let new_user = db::create_oauth_user(
            pool,
            user_info.email.clone(),
            user_info.given_name,
            user_info.family_name,
            "google".to_string(),
            user_info.sub,
            user_info.picture,
            None,
            db::UserRole::Member,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create OAuth user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to create user account" }),
            )
        })?;

        AuditLogger::log_oauth_login(
            pool,
            new_user.id.clone(),
            user_info.email.clone(),
            "google".to_string(),
            true,
            ip_address,
            user_agent,
        )
        .await;

        Ok(new_user)
    }

    pub async fn link_google_account(
        &self,
        pool: &PgPool,
        user_id: &str,
        user_info: OAuthUserInfo,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        if db::get_user_by_oauth(pool, "google", &user_info.sub)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .is_some()
        {
            return Err((
                StatusCode::CONFLICT,
                json!({ "error": "This Google account is already linked to another user" }),
            ));
        }

        db::link_oauth_to_user(
            pool,
            user_id,
            "google".to_string(),
            user_info.sub,
            user_info.picture,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to link OAuth account: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to link account" }),
            )
        })?;

        Ok(())
    }

    pub fn generate_jwt_for_user(
        &self,
        user_id: String,
    ) -> Result<String, (StatusCode, serde_json::Value)> {
        let jwt_config = JwtManager::get_config();
        jwt_config.generate_token(user_id, 24).map_err(|e| {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to generate token" }),
            )
        })
    }
}
