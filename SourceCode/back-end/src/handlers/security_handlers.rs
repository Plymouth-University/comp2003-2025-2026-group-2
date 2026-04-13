use crate::{
    AppState, db,
    dto::{ErrorResponse, SecurityLogDto, SecurityLogsQuery, SecurityLogsResponse},
    middleware::LogSmartAdminUser,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

const DEFAULT_SECURITY_LOG_LIMIT: i64 = 15;
const MAX_SECURITY_LOG_LIMIT: i64 = 100;
const MAX_SECURITY_LOG_EXPORT_ROWS: i64 = 10_000;

#[utoipa::path(
    get,
    path = "/security/logs",
    params(SecurityLogsQuery),
    responses(
        (status = 200, description = "Security logs retrieved successfully", body = SecurityLogsResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - LogSmart admin only", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn get_security_logs(
    LogSmartAdminUser(_claims, _user): LogSmartAdminUser,
    State(state): State<AppState>,
    Query(params): Query<SecurityLogsQuery>,
) -> impl IntoResponse {
    let limit = params
        .limit
        .unwrap_or(DEFAULT_SECURITY_LOG_LIMIT)
        .clamp(1, MAX_SECURITY_LOG_LIMIT);

    let cursor = match params.cursor.as_deref() {
        Some(cursor) => match db::parse_security_logs_cursor(cursor) {
            Ok(parsed) => Some(parsed),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid cursor" })),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let created_from = match parse_optional_utc_datetime(params.created_from.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err }))).into_response();
        }
    };

    let created_to = match parse_optional_utc_datetime(params.created_to.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err }))).into_response();
        }
    };

    let filters = db::SecurityLogFilters {
        event_type: params.event_type,
        user_id: params.user_id,
        email: params.email,
        ip_address: params.ip_address,
        user_agent: params.user_agent,
        details: params.details,
        success: params.success,
        created_from,
        created_to,
    };

    match db::get_security_logs_page(&state.postgres, &filters, limit, cursor).await {
        Ok(page) => {
            let logs = page.logs.into_iter().map(SecurityLogDto::from).collect();
            Json(SecurityLogsResponse {
                logs,
                next_cursor: page.next_cursor,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to get security logs: {e}") })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/security/logs/export",
    params(SecurityLogsQuery),
    responses(
        (status = 200, description = "CSV export of security logs", content_type = "text/csv"),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - LogSmart admin only", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn export_security_logs_csv(
    LogSmartAdminUser(_claims, _user): LogSmartAdminUser,
    State(state): State<AppState>,
    Query(params): Query<SecurityLogsQuery>,
) -> impl IntoResponse {
    let created_from = match parse_optional_utc_datetime(params.created_from.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err }))).into_response();
        }
    };

    let created_to = match parse_optional_utc_datetime(params.created_to.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err }))).into_response();
        }
    };

    let filters = db::SecurityLogFilters {
        event_type: params.event_type,
        user_id: params.user_id,
        email: params.email,
        ip_address: params.ip_address,
        user_agent: params.user_agent,
        details: params.details,
        success: params.success,
        created_from,
        created_to,
    };

    let rows = match db::get_security_logs_for_export(
        &state.postgres,
        &filters,
        MAX_SECURITY_LOG_EXPORT_ROWS,
    )
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to export security logs: {e}") })),
            )
                .into_response();
        }
    };

    let mut csv = String::from(
        "id,event_type,user_id,email,ip_address,user_agent,details,success,created_at\n",
    );
    for row in rows {
        let line = format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&row.id),
            csv_escape(&row.event_type),
            csv_escape(row.user_id.as_deref().unwrap_or("")),
            csv_escape(row.email.as_deref().unwrap_or("")),
            csv_escape(row.ip_address.as_deref().unwrap_or("")),
            csv_escape(row.user_agent.as_deref().unwrap_or("")),
            csv_escape(row.details.as_deref().unwrap_or("")),
            if row.success { "true" } else { "false" },
            csv_escape(&row.created_at.to_rfc3339()),
        );
        csv.push_str(&line);
    }

    let filename = format!(
        "security-logs-{}.csv",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    let disposition = match HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to build export response" })),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/csv; charset=utf-8"),
            ),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        csv,
    )
        .into_response()
}

fn parse_optional_utc_datetime(
    value: Option<&str>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    match value {
        Some(raw) if !raw.trim().is_empty() => chrono::DateTime::parse_from_rfc3339(raw)
            .map(|dt| Some(dt.with_timezone(&chrono::Utc)))
            .map_err(|_| format!("Invalid RFC3339 datetime: {raw}")),
        _ => Ok(None),
    }
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_optional_utc_datetime_valid() {
        let parsed = parse_optional_utc_datetime(Some("2026-04-13T12:00:00Z"))
            .expect("datetime should parse")
            .expect("value should be present");

        assert_eq!(parsed.to_rfc3339(), "2026-04-13T12:00:00+00:00");
    }

    #[test]
    fn test_parse_optional_utc_datetime_invalid() {
        let parsed = parse_optional_utc_datetime(Some("not-a-date"));
        assert!(parsed.is_err(), "invalid datetime must fail");
    }

    #[test]
    fn test_csv_escape_quotes_and_commas() {
        let escaped = csv_escape("hello, \"world\"");
        assert_eq!(escaped, "\"hello, \"\"world\"\"\"");
    }
}
