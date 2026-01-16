use crate::{
    AppState, db,
    dto::{ErrorResponse, GetCompanyMembersResponse},
    middleware::AuthToken,
};
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/auth/company/members",
    responses(
        (status = 200, description = "Company members retrieved successfully", body = [GetCompanyMembersResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
pub async fn get_company_members(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<GetCompanyMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
    let company_id = db::get_user_company_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user company ID: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User is not associated with a company" })),
        ))?;

    let members = db::get_users_by_company_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company members: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    Ok(Json(members.into()))
}
