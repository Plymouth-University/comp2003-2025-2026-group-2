use crate::{
    AppState, db,
    dto::{ClockEventResponse, ClockStatusResponse, ErrorResponse},
    middleware::AuthToken,
    services,
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
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
    let (current, recent) =
        services::ClockService::get_status(&state.postgres, &claims.user_id)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    let is_clocked_in = current
        .as_ref()
        .map_or(false, |e| e.status == "in");

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
