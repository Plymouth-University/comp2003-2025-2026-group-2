use crate::{
    AppState, db,
    dto::{
        ClockEventResponse, ClockStatusResponse, CompanyClockEventResponse,
        CompanyClockEventsResponse, ErrorResponse,
    },
    middleware::{AdminUser, AuthToken},
    services,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;

#[utoipa::path(
    post,
    path = "/clock/in",
    responses(
        (status = 200, description = "Successfully clocked in", body = ClockEventResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Already clocked in", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Clock In/Out"
)]
/// Clocks in the current user.
///
/// # Errors
/// Returns an error if the user is already clocked in or if DB operations fail.
pub async fn clock_in(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ClockEventResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let event = services::ClockService::clock_in(&state.postgres, &claims.user_id, &company_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(ClockEventResponse::from(event)))
}

#[utoipa::path(
    post,
    path = "/clock/out",
    responses(
        (status = 200, description = "Successfully clocked out", body = ClockEventResponse),
        (status = 400, description = "Not currently clocked in", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Clock In/Out"
)]
/// Clocks out the current user.
///
/// # Errors
/// Returns an error if the user is not clocked in or if DB operations fail.
pub async fn clock_out(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ClockEventResponse>, (StatusCode, Json<serde_json::Value>)> {
    let event = services::ClockService::clock_out(&state.postgres, &claims.user_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(ClockEventResponse::from(event)))
}

#[utoipa::path(
    get,
    path = "/clock/status",
    responses(
        (status = 200, description = "Current clock status and recent events", body = ClockStatusResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Clock In/Out"
)]
/// Gets the current clock status and last 5 events for the user.
///
/// # Errors
/// Returns an error if DB operations fail.
pub async fn get_clock_status(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ClockStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let (current, recent) = services::ClockService::get_status(&state.postgres, &claims.user_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

    let is_clocked_in = current.as_ref().is_some_and(|e| e.status == "in");

    let current_event = if is_clocked_in {
        current.map(ClockEventResponse::from)
    } else {
        None
    };

    let recent_events = recent.into_iter().map(ClockEventResponse::from).collect();

    Ok(Json(ClockStatusResponse {
        is_clocked_in,
        current_event,
        recent_events,
    }))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CompanyClockQuery {
    /// ISO 8601 start date filter (inclusive)
    pub from: Option<String>,
    /// ISO 8601 end date filter (inclusive)
    pub to: Option<String>,
    /// Branch ID filter (optional, for company_manager to filter by specific branch)
    pub branch_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/clock/company",
    params(CompanyClockQuery),
    responses(
        (status = 200, description = "All company clock events", body = CompanyClockEventsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden – admin only", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Clock In/Out"
)]
/// Gets all clock in/out events for the admin's company.
///
/// # Errors
/// Returns an error if the user is not an admin or if DB operations fail.
pub async fn get_company_clock_events(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Query(params): Query<CompanyClockQuery>,
) -> Result<Json<CompanyClockEventsResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    // Fetch user to get role and branch_id
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

    let from = params
        .from
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let to = params
        .to
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // If user is branch_manager, restrict to their branch only
    let branch_id_filter = if user.role.to_string() == "branch_manager" {
        user.branch_id
    } else {
        params.branch_id
    };

    let events =
        services::ClockService::get_company_clock_events(&state.postgres, &company_id, from, to, branch_id_filter)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    let events = events
        .into_iter()
        .map(CompanyClockEventResponse::from)
        .collect();
    Ok(Json(CompanyClockEventsResponse { events }))
}
