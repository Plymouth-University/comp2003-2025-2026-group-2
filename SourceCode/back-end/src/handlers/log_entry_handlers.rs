use crate::{
    AppState, db,
    dto::{
        CreateLogEntryRequest, CreateLogEntryResponse, DueFormInfo, DueFormsResponse,
        ErrorResponse, ListLogEntriesResponse, LogEntryResponse, SubmitLogEntryResponse,
        UpdateLogEntryRequest,
    },
    logs_db,
    middleware::AuthToken,
    services,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/logs/entries/due",
    responses(
        (status = 200, description = "Due forms retrieved successfully", body = DueFormsResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Lists all log forms that are due today for the current user's company.
///
/// # Errors
/// Returns an error if the user is not authorized or if the query fails.
pub async fn list_due_forms_today(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<DueFormsResponse>, (StatusCode, Json<serde_json::Value>)> {
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
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        ))?;

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let templates =
        services::LogEntryService::list_due_forms(&state, &company_id, user.branch_id.as_deref())
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    let mut due_forms = Vec::new();

    for template in templates {
        if logs_db::is_form_due_today(&template.schedule) {
            let has_submitted_entry = logs_db::has_submitted_entry_for_current_period(
                &state.mongodb,
                &company_id,
                &template.template_name,
                &template.schedule.frequency,
            )
            .await
            .unwrap_or(false);

            if !has_submitted_entry {
                let last_submitted = logs_db::get_latest_submitted_entry(
                    &state.mongodb,
                    &claims.user_id,
                    &company_id,
                    &template.template_name,
                )
                .await
                .ok()
                .flatten();

                let draft_entry = logs_db::get_draft_entry_for_current_period(
                    &state.mongodb,
                    &claims.user_id,
                    &company_id,
                    &template.template_name,
                    &template.schedule.frequency,
                )
                .await
                .ok()
                .flatten();

                let processed_layout = logs_db::process_template_layout_with_period(
                    &template.template_layout,
                    &template.schedule.frequency,
                );

                let period = logs_db::format_period_for_frequency(&template.schedule.frequency);

                due_forms.push(DueFormInfo {
                    template_name: template.template_name,
                    template_layout: processed_layout,
                    last_submitted: last_submitted
                        .and_then(|e| e.submitted_at.map(|ts| ts.to_rfc3339())),
                    period,
                    status: draft_entry.map(|e| e.status),
                });
            }
        }
    }

    Ok(Json(DueFormsResponse { forms: due_forms }))
}

#[utoipa::path(
    post,
    path = "/logs/entries",
    request_body = CreateLogEntryRequest,
    responses(
        (status = 201, description = "Log entry created successfully", body = CreateLogEntryResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Creates a new log entry draft.
///
/// # Errors
/// Returns an error if the template name is empty or if entry creation fails.
pub async fn create_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<CreateLogEntryRequest>,
) -> Result<(StatusCode, Json<CreateLogEntryResponse>), (StatusCode, Json<serde_json::Value>)> {
    if payload.template_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Template name is required" })),
        ));
    }

    let entry_id = services::LogEntryService::create_log_entry(
        &state,
        &claims.user_id,
        &payload.template_name,
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateLogEntryResponse {
            id: entry_id,
            message: "Log entry created successfully.".to_string(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/logs/entries/{entry_id}",
    responses(
        (status = 200, description = "Log entry retrieved successfully", body = LogEntryResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 403, description = "Forbidden - entry does not belong to user", body = ErrorResponse),
        (status = 404, description = "Entry not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Retrieves a specific log entry by its ID.
///
/// # Errors
/// Returns an error if the entry is not found or if the user is not authorized.
pub async fn get_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
) -> Result<Json<LogEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let entry = services::LogEntryService::get_log_entry(&state, &claims.user_id, &entry_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

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

    let template = logs_db::get_template_by_name(&state.mongodb, &entry.template_name, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get template: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get template" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Template not found" })),
        ))?;

    let processed_layout = logs_db::process_template_layout_with_period_string(
        &template.template_layout,
        &entry.period,
    );

    Ok(Json(LogEntryResponse {
        id: entry.entry_id,
        template_name: entry.template_name,
        template_layout: processed_layout,
        entry_data: entry.entry_data,
        status: entry.status,
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
        submitted_at: entry.submitted_at.map(|ts| ts.to_rfc3339()),
        period: entry.period,
    }))
}

#[utoipa::path(
    put,
    path = "/logs/entries/{entry_id}",
    request_body = UpdateLogEntryRequest,
    responses(
        (status = 200, description = "Log entry updated successfully", body = LogEntryResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 403, description = "Forbidden - entry does not belong to user", body = ErrorResponse),
        (status = 404, description = "Entry not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Updates the data of an existing log entry draft.
///
/// # Errors
/// Returns an error if the entry is not found or if the update fails.
pub async fn update_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
    Json(payload): Json<UpdateLogEntryRequest>,
) -> Result<Json<LogEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let updated_entry = services::LogEntryService::update_log_entry(
        &state,
        &claims.user_id,
        &entry_id,
        &payload.entry_data,
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

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

    let template =
        logs_db::get_template_by_name(&state.mongodb, &updated_entry.template_name, &company_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get template" })),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Template not found" })),
            ))?;

    let processed_layout = logs_db::process_template_layout_with_period_string(
        &template.template_layout,
        &updated_entry.period,
    );

    Ok(Json(LogEntryResponse {
        id: updated_entry.entry_id,
        template_name: updated_entry.template_name,
        template_layout: processed_layout,
        entry_data: updated_entry.entry_data,
        status: updated_entry.status,
        created_at: updated_entry.created_at.to_rfc3339(),
        updated_at: updated_entry.updated_at.to_rfc3339(),
        submitted_at: updated_entry.submitted_at.map(|ts| ts.to_rfc3339()),
        period: updated_entry.period,
    }))
}

#[utoipa::path(
    post,
    path = "/logs/entries/{entry_id}/submit",
    responses(
        (status = 200, description = "Log entry submitted successfully", body = SubmitLogEntryResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 403, description = "Forbidden - entry does not belong to user", body = ErrorResponse),
        (status = 404, description = "Entry not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Submits a log entry, marking it as final.
///
/// # Errors
/// Returns an error if the entry is not found or if submission fails.
pub async fn submit_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
) -> Result<Json<SubmitLogEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    services::LogEntryService::submit_log_entry(&state, &claims.user_id, &entry_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(SubmitLogEntryResponse {
        message: "Log entry submitted successfully.".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/logs/entries/{entry_id}/unsubmit",
    responses(
        (status = 200, description = "Log entry returned to draft successfully", body = SubmitLogEntryResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 403, description = "Forbidden - only admins can unsubmit entries", body = ErrorResponse),
        (status = 404, description = "Entry not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Returns a submitted log entry to draft status (admin only).
///
/// # Errors
/// Returns an error if the entry is not found or if the operation fails.
pub async fn unsubmit_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
) -> Result<Json<SubmitLogEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    services::LogEntryService::unsubmit_log_entry(&state, &claims.user_id, &entry_id)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(SubmitLogEntryResponse {
        message: "Log entry returned to draft successfully.".to_string(),
    }))
}

#[utoipa::path(
    delete,
    path = "/logs/entries/{entry_id}",
    responses(
        (status = 200, description = "Log entry deleted successfully", body = serde_json::Value),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 403, description = "Forbidden - entry does not belong to user", body = ErrorResponse),
        (status = 404, description = "Entry not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Deletes a log entry.
///
/// # Errors
/// Returns an error if the user is not authorized or if deletion fails.
pub async fn delete_log_entry(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        ))?;

    services::LogEntryService::delete_log_entry(
        &state,
        &claims.user_id,
        &entry_id,
        user.can_manage_company(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(json!({ "message": "Log entry deleted successfully" })))
}

#[utoipa::path(
    get,
    path = "/logs/admin/entries",
    responses(
        (status = 200, description = "Company log entries retrieved successfully", body = ListLogEntriesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - only managers can view all entries", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
/// Lists all log entries for the company or branch (managers only).
pub async fn list_company_log_entries(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ListLogEntriesResponse>, (StatusCode, Json<serde_json::Value>)> {
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
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        ))?;

    if !user.can_manage_branch() && !user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can view these logs" })),
        ));
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let entries = if user.is_company_manager() || user.is_logsmart_admin() || user.is_readonly_hq() {
        logs_db::get_company_log_entries(&state.mongodb, &company_id).await
    } else {
        let branch_id = user.branch_id.as_ref().ok_or((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Branch manager has no branch assigned" })),
        ))?;
        logs_db::get_branch_log_entries(&state.mongodb, &company_id, branch_id).await
    }
    .map_err(|e| {
        tracing::error!("Failed to get log entries: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to get log entries" })),
        )
    })?;

    let mut response_entries = Vec::new();
    for e in entries {
        let template = logs_db::get_template_by_name(&state.mongodb, &e.template_name, &company_id)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get template: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get template" })),
                )
            })?;

        let processed_layout = if let Some(template) = template {
            logs_db::process_template_layout_with_period_string(
                &template.template_layout,
                &e.period,
            )
        } else {
            Vec::new()
        };

        response_entries.push(LogEntryResponse {
            id: e.entry_id,
            template_name: e.template_name,
            template_layout: processed_layout,
            entry_data: e.entry_data,
            status: e.status,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
            submitted_at: e.submitted_at.map(|ts| ts.to_rfc3339()),
            period: e.period,
        });
    }

    Ok(Json(ListLogEntriesResponse {
        entries: response_entries,
    }))
}

#[utoipa::path(
    get,
    path = "/logs/entries",
    responses(
        (status = 200, description = "User log entries retrieved successfully", body = ListLogEntriesResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Log Entries"
)]
#[allow(clippy::implicit_hasher)]
/// Lists all log entries for the current user, optionally filtered by template or status.
///
/// # Errors
/// Returns an error if the user is not authorized or if the query fails.
pub async fn list_user_log_entries(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListLogEntriesResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let mut entries =
        services::LogEntryService::get_user_log_entries(&state, &claims.user_id, &company_id)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    if let Some(template_name) = params.get("template_name") {
        entries.retain(|e| e.template_name == *template_name);
    }

    if let Some(status) = params.get("status") {
        entries.retain(|e| e.status == *status);
    }

    let mut response_entries = Vec::new();
    for e in entries {
        let template = logs_db::get_template_by_name(&state.mongodb, &e.template_name, &company_id)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get template: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to get template" })),
                )
            })?;

        let processed_layout = if let Some(template) = template {
            logs_db::process_template_layout_with_period_string(
                &template.template_layout,
                &e.period,
            )
        } else {
            Vec::new()
        };

        response_entries.push(LogEntryResponse {
            id: e.entry_id,
            template_name: e.template_name,
            template_layout: processed_layout,
            entry_data: e.entry_data,
            status: e.status,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
            submitted_at: e.submitted_at.map(|ts| ts.to_rfc3339()),
            period: e.period,
        });
    }

    Ok(Json(ListLogEntriesResponse {
        entries: response_entries,
    }))
}
