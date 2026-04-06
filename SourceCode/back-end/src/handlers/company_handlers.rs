use crate::{
    AppState, db,
    dto::ErrorResponse,
    exports_db, images_db,
    logs_db::{self, TemplateDocument},
    middleware::ManageCompanyUser,
    utils::{err_bad_request, err_forbidden, err_internal, err_not_found},
};
use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{StatusCode, header},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CompanyExportData {
    pub log_templates: Vec<TemplateDocument>,
    pub log_entries: Vec<logs_db::LogEntry>,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExportResponse {
    pub message: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub logo_id: Option<String>,
    pub logo_url: Option<String>,
    pub data_exported_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<db::Company> for CompanyResponse {
    fn from(company: db::Company) -> Self {
        let logo_id = company.logo_id.clone();
        let logo_url = logo_id
            .clone()
            .map(|_id| format!("/api/companies/{}/logo", company.id));
        Self {
            id: company.id,
            name: company.name,
            address: company.address,
            logo_id,
            logo_url,
            data_exported_at: company.data_exported_at,
            deleted_at: company.deleted_at,
            deletion_requested_at: company.deletion_requested_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCompanyRequest {
    pub name: String,
    pub address: String,
}

fn infer_content_type(data: &[u8]) -> String {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png".to_string()
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".to_string()
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

#[utoipa::path(
    post,
    path = "/companies/{company_id}/logo",
    request_body = Vec<u8>,
    responses(
        (status = 200, description = "Logo uploaded successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn upload_company_logo(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only manage your own company's logo"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let data = body.to_vec();

    if data.len() > 10 * 1024 * 1024 {
        return Err(err_bad_request("File too large. Maximum size is 10MB"));
    }

    if data.is_empty() {
        return Err(err_bad_request("No file provided"));
    }

    let content_type = infer_content_type(&data);
    if !content_type.starts_with("image/") {
        return Err(err_bad_request("File must be an image"));
    }

    let file_id = images_db::upload_company_logo(&state.mongodb, data, &company_id, &content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload company logo: {:?}", e);
            err_internal("Failed to upload logo")
        })?;

    if let Err(err) = db::update_company_logo_id(&state.postgres, &company_id, Some(&file_id)).await
    {
        tracing::error!("Failed to update company logo: {:?}", err);
        if let Err(delete_err) = images_db::delete_company_logo(&state.mongodb, &file_id).await {
            tracing::error!("Failed to cleanup uploaded logo: {:?}", delete_err);
        }
        return Err(err_internal("Failed to update logo reference"));
    }

    if let Some(old_logo_id) = &company.logo_id
        && let Err(err) = images_db::delete_company_logo(&state.mongodb, old_logo_id).await
    {
        tracing::error!("Failed to delete old company logo: {:?}", err);
    }

    Ok(Json(json!({
        "logo_id": file_id,
        "logo_url": format!("/api/companies/{}/logo", company_id)
    })))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}/logo",
    responses(
        (status = 200, description = "Company logo", content_type = "image/webp"),
        (status = 404, description = "Logo not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn get_company_logo(
    ManageCompanyUser(_claims, _user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, header::HeaderValue); 1],
        Vec<u8>,
    ),
    (StatusCode, Json<serde_json::Value>),
> {
    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let logo_id = company
        .logo_id
        .ok_or_else(|| err_not_found("Logo not found"))?;

    if let Some((content_type, data)) = images_db::get_company_logo(&state.mongodb, &logo_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get company logo: {:?}", e);
            err_internal("Failed to get logo")
        })?
    {
        let content_type_header = header::HeaderValue::from_str(&content_type)
            .unwrap_or(header::HeaderValue::from_static("application/octet-stream"));
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type_header)],
            data,
        ))
    } else {
        Err(err_not_found("Logo not found"))
    }
}

#[utoipa::path(
    delete,
    path = "/companies/{company_id}/logo",
    responses(
        (status = 200, description = "Logo deleted successfully", body = serde_json::Value),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn delete_company_logo(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only manage your own company's logo"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let Some(logo_id) = company.logo_id else {
        return Err(err_not_found("No logo to delete"));
    };

    db::update_company_logo_id(&state.postgres, &company_id, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update company logo: {:?}", e);
            err_internal("Failed to delete logo reference")
        })?;

    if let Err(err) = images_db::delete_company_logo(&state.mongodb, &logo_id).await {
        tracing::error!("Failed to delete company logo from storage: {:?}", err);
        let _ = db::update_company_logo_id(&state.postgres, &company_id, Some(&logo_id)).await;
        return Err(err_internal("Failed to delete logo"));
    }

    Ok(Json(json!({ "message": "Logo deleted successfully" })))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}",
    responses(
        (status = 200, description = "Company details", body = CompanyResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn get_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only view your own company"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    Ok(Json(CompanyResponse::from(company)))
}

#[utoipa::path(
    put,
    path = "/companies/{company_id}",
    request_body = UpdateCompanyRequest,
    responses(
        (status = 200, description = "Company updated successfully", body = CompanyResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn update_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only update your own company"));
    }

    if payload.name.trim().is_empty() {
        return Err(err_bad_request("Company name cannot be empty"));
    }

    if payload.address.trim().is_empty() {
        return Err(err_bad_request("Company address cannot be empty"));
    }

    let company = db::update_company(
        &state.postgres,
        &company_id,
        &payload.name,
        &payload.address,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update company: {:?}", e);
        err_internal("Database error")
    })?;

    // Invalidate cache for all users in the company, including soft-deleted ones
    let all_users = match db::get_all_users_by_company_id(&state.postgres, &company_id).await {
        Ok(users) => users,
        Err(e) => {
            tracing::warn!("Failed to fetch users for cache invalidation: {e:?}");
            Vec::new()
        }
    };

    for user in &all_users {
        state.user_cache.invalidate(&user.id).await;
    }

    Ok(Json(CompanyResponse::from(company)))
}

#[utoipa::path(
    post,
    path = "/companies/{company_id}/export",
    responses(
        (status = 200, description = "Export started, download link will be emailed", body = ExportResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
        (status = 429, description = "Rate limited - one export per week", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn export_company_data(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only export your own company data"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if !state.rate_limit.check_export(&company_id) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "You can only export once per week. Please try again later.",
            })),
        ));
    }

    let company_email = user.email.clone();
    let company_name = company.name.clone();

    let log_templates = match logs_db::get_templates_by_company(&state.mongodb, &company_id).await {
        Ok(templates) => templates,
        Err(e) => {
            tracing::warn!("Failed to fetch log templates for export: {:?}", e);
            vec![]
        }
    };

    let log_entries = match logs_db::get_company_log_entries(&state.mongodb, &company_id).await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to fetch log entries for export: {:?}", e);
            vec![]
        }
    };

    let exported_at = chrono::Utc::now();

    let templates_json = serde_json::to_string_pretty(&log_templates).unwrap_or_default();
    let entries_json = serde_json::to_string_pretty(&log_entries).unwrap_or_default();

    let zip_data = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip.start_file("templates.json", options)?;
            zip.write_all(templates_json.as_bytes())?;

            zip.start_file("log_entries.json", options)?;
            zip.write_all(entries_json.as_bytes())?;

            zip.finish()?;
        }
        Ok(buf)
    })
    .await
    .map_err(|e| {
        tracing::error!("ZIP generation failed: {:?}", e);
        err_internal("Failed to generate export")
    })?
    .map_err(|e| {
        tracing::error!("ZIP generation error: {:?}", e);
        err_internal("Failed to generate export")
    })?;

    let filename = exports_db::save_export(&company_id, &zip_data)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save export file: {:?}", e);
            err_internal("Failed to save export")
        })?;

    db::mark_company_data_exported(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark company data as exported: {:?}", e);
            err_internal("Failed to mark data as exported")
        })?;

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    tokio::spawn(async move {
        if let Err(e) = crate::email::send_export_ready_notification(
            &company_email,
            &company_name,
            &company_id,
            &filename,
            &frontend_url,
        )
        .await
        {
            tracing::error!("Failed to send export notification email: {:?}", e);
        }
    });

    Ok(Json(ExportResponse {
        message: "Export complete. A download link has been sent to your email.".to_string(),
        exported_at,
    }))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}/export/download",
    responses(
        (status = 200, description = "Export file", content_type = "application/zip"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Export not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn download_export(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(_state): State<AppState>,
    axum::extract::Path((company_id, filename)): axum::extract::Path<(String, String)>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden(
            "You can only download your own company exports",
        ));
    }

    if !filename.ends_with(".zip") || !filename.starts_with(&format!("{company_id}_")) {
        return Err(err_bad_request("Invalid export filename"));
    }

    let data = exports_db::get_export(&company_id, &filename)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read export file: {:?}", e);
            err_internal("Failed to read export")
        })?
        .ok_or_else(|| err_not_found("Export file not found or expired"))?;

    let content_length = data.len().to_string();

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_LENGTH, content_length)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(axum::body::Body::from(data))
        .unwrap();

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/companies/{company_id}",
    responses(
        (status = 200, description = "Company deletion requested", body = serde_json::Value),
        (status = 400, description = "Data must be exported first", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn delete_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only delete your own company"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if company.data_exported_at.is_none() {
        return Err(err_bad_request(
            "You must export company data before requesting deletion",
        ));
    }

    if company.deletion_requested_at.is_some() {
        return Err(err_bad_request(
            "A deletion request is already pending. Please check your email to confirm.",
        ));
    }

    let company_email = user.email.clone();
    let company_name = company.name.clone();

    let updated_company =
        db::request_company_deletion(&state.postgres, &company_id, &company_email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to request company deletion: {:?}", e);
                err_internal("Failed to request company deletion")
            })?;

    let deletion_token = updated_company.deletion_token.unwrap_or_default();

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    tokio::spawn(async move {
        if let Err(e) = crate::email::send_company_deletion_request(
            &company_email,
            &company_name,
            &company_id,
            &deletion_token,
            &frontend_url,
        )
        .await
        {
            tracing::error!("Failed to send company deletion request email: {:?}", e);
        }
    });

    Ok(Json(
        json!({ "message": "Deletion requested. Please check your email to confirm." }),
    ))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}/validate-deletion-token",
    responses(
        (status = 200, description = "Token valid, returns company name", body = serde_json::Value),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    tag = "Company"
)]
pub async fn validate_company_deletion_token(
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    axum::extract::Query(token): axum::extract::Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token = token.get("token").and_then(|t| t.as_str()).unwrap_or("");

    if token.is_empty() {
        return Err(err_bad_request("Confirmation token is required"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if company.deletion_token.as_deref() != Some(token) {
        return Err(err_bad_request("Invalid or expired confirmation token"));
    }

    Ok(Json(json!({ "companyName": company.name })))
}

#[utoipa::path(
    post,
    path = "/companies/{company_id}/confirm-deletion",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Company deletion confirmed", body = serde_json::Value),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    tag = "Company"
)]
pub async fn confirm_company_deletion(
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token = payload.get("token").and_then(|t| t.as_str()).unwrap_or("");

    if token.is_empty() {
        return Err(err_bad_request("Confirmation token is required"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if company.deletion_token.as_deref() != Some(token) {
        return Err(err_bad_request("Invalid or expired confirmation token"));
    }

    let updated_company = db::confirm_company_deletion(&state.postgres, &company_id, token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to confirm company deletion: {:?}", e);
            err_internal("Failed to confirm deletion")
        })?;

    // Invalidate cache for all users in the company, including soft-deleted ones
    // Note: This runs after deletion is confirmed, so get_all_users_by_company_id
    // must not filter by deleted_at to work correctly
    let all_users = db::get_all_users_by_company_id(&state.postgres, &company_id)
        .await
        .unwrap_or_default();

    for user in &all_users {
        state.user_cache.invalidate(&user.id).await;
    }

    let user_email = updated_company.deletion_requested_by_email.clone();
    let company_name = updated_company.name.clone();

    tokio::spawn(async move {
        if let Err(e) = crate::email::send_company_deleted_notification(&company_name).await {
            tracing::error!("Failed to send company deletion notification: {:?}", e);
        }
        if let Some(email) = user_email
            && let Err(e) =
                crate::email::send_user_company_deleted_notification(&email, &company_name).await
        {
            tracing::error!("Failed to send user company deletion notification: {:?}", e);
        }
    });

    Ok(Json(
        json!({ "message": "Company has been deleted. Data will be retained for 30 days." }),
    ))
}
