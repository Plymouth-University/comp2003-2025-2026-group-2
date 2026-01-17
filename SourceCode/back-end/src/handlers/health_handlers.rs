use crate::{AppState, db, middleware::AuthToken};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub metrics: db::DatabaseHealthMetrics,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SlowQueriesResponse {
    pub queries: Vec<db::SlowQueryInfo>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct IndexUsageResponse {
    pub indexes: Vec<db::IndexUsageStats>,
    pub unused_indexes: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TableSizesResponse {
    pub tables: Vec<db::TableSizeInfo>,
}

#[utoipa::path(
    get,
    path = "/health/database",
    responses(
        (status = 200, description = "Database health metrics retrieved successfully", body = HealthResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - LogSmart admin only"),
        (status = 500, description = "Server error"),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn get_db_health(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = match db::get_user_by_id(&state.postgres, &claims.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {e}")})),
            )
                .into_response();
        }
    };

    if !user.is_logsmart_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only LogSmart admins can view database health"})),
        )
            .into_response();
    }

    match db::get_database_health(&state.postgres).await {
        Ok(metrics) => Json(HealthResponse {
            status: "healthy".to_string(),
            metrics,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get health metrics: {e}")})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/health/slow-queries",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of slow queries to return (default: 20)")
    ),
    responses(
        (status = 200, description = "Slow queries retrieved successfully", body = SlowQueriesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - LogSmart admin only"),
        (status = 500, description = "Server error"),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn get_db_slow_queries(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user = match db::get_user_by_id(&state.postgres, &claims.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {e}")})),
            )
                .into_response();
        }
    };

    if !user.is_logsmart_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only LogSmart admins can view slow queries"})),
        )
            .into_response();
    }

    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(20);

    match db::get_slow_queries(&state.postgres, limit).await {
        Ok(queries) => Json(SlowQueriesResponse { queries }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get slow queries: {e}")})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/health/index-usage",
    responses(
        (status = 200, description = "Index usage statistics retrieved successfully", body = IndexUsageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - LogSmart admin only"),
        (status = 500, description = "Server error"),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn get_db_index_usage(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = match db::get_user_by_id(&state.postgres, &claims.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {e}")})),
            )
                .into_response();
        }
    };

    if !user.is_logsmart_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only LogSmart admins can view index usage"})),
        )
            .into_response();
    }

    let indexes = match db::get_index_usage(&state.postgres).await {
        Ok(idx) => idx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to get index usage: {e}")})),
            )
                .into_response();
        }
    };

    let unused = match db::check_unused_indexes(&state.postgres).await {
        Ok(un) => un,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to check unused indexes: {e}")})),
            )
                .into_response();
        }
    };

    Json(IndexUsageResponse {
        indexes,
        unused_indexes: unused,
    })
    .into_response()
}

#[utoipa::path(
    get,
    path = "/health/table-sizes",
    responses(
        (status = 200, description = "Table sizes retrieved successfully", body = TableSizesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - LogSmart admin only"),
        (status = 500, description = "Server error"),
    ),
    security(("bearer_auth" = [])),
    tag = "Health Monitoring"
)]
pub async fn get_db_table_sizes(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user = match db::get_user_by_id(&state.postgres, &claims.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {e}")})),
            )
                .into_response();
        }
    };

    if !user.is_logsmart_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only LogSmart admins can view table sizes"})),
        )
            .into_response();
    }

    match db::get_table_sizes(&state.postgres).await {
        Ok(tables) => Json(TableSizesResponse { tables }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get table sizes: {e}")})),
        )
            .into_response(),
    }
}
