use crate::handlers::get_jwt_secret;
use crate::{
    AppState,
    auth::{Claims, JwtConfig},
};
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
            let jwt_config = JwtConfig::new(get_jwt_secret());

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
