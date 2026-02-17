use crate::{AppState, auth::Claims, db::UserRole, jwt_manager::JwtManager};
use axum::{
    Json,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use serde_json::json;

pub struct AuthToken(pub Claims);

impl FromRequestParts<crate::AppState> for AuthToken {
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &crate::AppState,
    ) -> impl std::future::Future<
        Output = Result<Self, <Self as FromRequestParts<AppState>>::Rejection>,
    > + Send {
        Box::pin(async move {
            let jwt_config = JwtManager::get_config();

            let TypedHeader(Authorization::<Bearer>(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                    .await
                    .map_err(|_| AuthError::MissingToken)?;

            let token = bearer.token();
            let claims = jwt_config
                .validate_token(token)
                .map_err(|_| AuthError::InvalidToken)?;

            Ok(AuthToken(claims))
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing authorization token",
            ),
            AuthError::InvalidToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            ),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

#[derive(Debug)]
pub enum RoleError {
    MissingToken,
    InvalidToken,
    InsufficientPermissions,
}

impl IntoResponse for RoleError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RoleError::MissingToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing authorization token",
            ),
            RoleError::InvalidToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            ),
            RoleError::InsufficientPermissions => (
                axum::http::StatusCode::FORBIDDEN,
                "Insufficient permissions for this operation",
            ),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

pub struct AdminUser(pub Claims);

impl FromRequestParts<crate::AppState> for AdminUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let jwt_config = JwtManager::get_config();

            let TypedHeader(Authorization::<Bearer>(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                    .await
                    .map_err(|_| RoleError::MissingToken)?;

            let token = bearer.token();
            let claims = jwt_config
                .validate_token(token)
                .map_err(|_| RoleError::InvalidToken)?;

            let user = if let Some(user) = state.user_cache.get(&claims.user_id).await {
                user
            } else {
                let user = crate::db::get_user_by_id(&state.postgres, &claims.user_id)
                    .await
                    .map_err(|_| RoleError::InvalidToken)?
                    .ok_or(RoleError::InvalidToken)?;
                state
                    .user_cache
                    .insert(claims.user_id.clone(), user.clone())
                    .await;
                user
            };

            if !user.can_manage_company() {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(AdminUser(claims))
        })
    }
}

pub struct MemberUser(pub Claims);

impl FromRequestParts<crate::AppState> for MemberUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let jwt_config = JwtManager::get_config();

            let TypedHeader(Authorization::<Bearer>(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                    .await
                    .map_err(|_| RoleError::MissingToken)?;

            let token = bearer.token();
            let claims = jwt_config
                .validate_token(token)
                .map_err(|_| RoleError::InvalidToken)?;

            let user = if let Some(user) = state.user_cache.get(&claims.user_id).await {
                user
            } else {
                let user = crate::db::get_user_by_id(&state.postgres, &claims.user_id)
                    .await
                    .map_err(|_| RoleError::InvalidToken)?
                    .ok_or(RoleError::InvalidToken)?;
                state
                    .user_cache
                    .insert(claims.user_id.clone(), user.clone())
                    .await;
                user
            };

            if !matches!(
                user.get_role(),
                UserRole::Staff
                    | UserRole::CompanyManager
                    | UserRole::BranchManager
                    | UserRole::LogSmartAdmin
            ) {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(MemberUser(claims))
        })
    }
}

pub struct LogSmartAdminUser(pub Claims);

impl FromRequestParts<crate::AppState> for LogSmartAdminUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let jwt_config = JwtManager::get_config();

            let TypedHeader(Authorization::<Bearer>(bearer)) =
                TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                    .await
                    .map_err(|_| RoleError::MissingToken)?;

            let token = bearer.token();
            let claims = jwt_config
                .validate_token(token)
                .map_err(|_| RoleError::InvalidToken)?;

            let user = if let Some(user) = state.user_cache.get(&claims.user_id).await {
                user
            } else {
                let user = crate::db::get_user_by_id(&state.postgres, &claims.user_id)
                    .await
                    .map_err(|_| RoleError::InvalidToken)?
                    .ok_or(RoleError::InvalidToken)?;
                state
                    .user_cache
                    .insert(claims.user_id.clone(), user.clone())
                    .await;
                user
            };

            if !user.is_logsmart_admin() {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(LogSmartAdminUser(claims))
        })
    }
}
