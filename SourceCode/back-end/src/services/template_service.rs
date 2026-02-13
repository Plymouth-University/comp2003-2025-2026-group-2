use crate::{AppState, logs_db};
use axum::http::StatusCode;
use serde_json::json;

#[cfg(test)]
mod template_service_tests {
    #[tokio::test]
    async fn test_template_service_basic() {
        assert!(true);
    }
}

pub struct TemplateService;

impl TemplateService {
    /// Creates a new log template.
    ///
    /// # Errors
    /// Returns an error if a template with the same name already exists or if database operations fail.
    pub async fn create_template(
        state: &AppState,
        company_id: &str,
        template_name: String,
        template_layout: logs_db::TemplateLayout,
        schedule: logs_db::Schedule,
        user_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let existing_template =
            logs_db::get_template_by_name(&state.mongodb, &template_name, company_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check for existing template: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Database error" }),
                    )
                })?;

        if existing_template.is_some() {
            return Err((
                StatusCode::CONFLICT,
                json!({ "error": "A template with this name already exists for your company" }),
            ));
        }

        let template_document = logs_db::TemplateDocument {
            template_name,
            template_layout,
            company_id: company_id.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: mongodb::bson::Uuid::parse_str(user_id).map_err(|e| {
                tracing::error!("Failed to parse user ID as UUID: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Server error" }),
                )
            })?,
            schedule,
            version: 1,
            version_name: None,
        };

        logs_db::add_template(&state.mongodb, &template_document)
            .await
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to add template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to add template" }),
                )
            })?;

        Ok(())
    }

    /// Retrieves a specific log template by its name.
    ///
    /// # Errors
    /// Returns an error if the template is not found or if database lookup fails.
    pub async fn get_template(
        state: &AppState,
        company_id: &str,
        template_name: &str,
    ) -> Result<(String, logs_db::TemplateLayout, u16, Option<String>), (StatusCode, serde_json::Value)> {
        let template = logs_db::get_template_by_name(&state.mongodb, template_name, company_id)
            .await
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to get template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get template" }),
                )
            })?;

        match template {
            Some(t) => Ok((t.template_name, t.template_layout, t.version, t.version_name)),
            None => Err((
                StatusCode::NOT_FOUND,
                json!({ "error": "Template not found" }),
            )),
        }
    }

    /// Retrieves all log templates associated with a company.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn get_all_templates(
        state: &AppState,
        company_id: &str,
    ) -> Result<
        Vec<(
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            String,
            logs_db::Schedule,
        )>,
        (StatusCode, serde_json::Value),
    > {
        let templates = logs_db::get_templates_by_company(&state.mongodb, company_id)
            .await
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to get templates: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get templates" }),
                )
            })?;

        let result = templates
            .into_iter()
            .map(|t| {
                (
                    t.template_name,
                    t.created_at,
                    t.updated_at,
                    t.created_by.to_string(),
                    t.schedule,
                )
            })
            .collect();

        Ok(result)
    }

    /// Updates an existing log template's layout or schedule.
    ///
    /// # Errors
    /// Returns an error if the database update fails.
    pub async fn update_template(
        state: &AppState,
        company_id: &str,
        template_name: &str,
        template_layout: Option<&logs_db::TemplateLayout>,
        schedule: Option<&logs_db::Schedule>,
        user_id: &str,
        version_name: Option<String>,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        // 1. Fetch current template state
        let current_template =
            logs_db::get_template_by_name(&state.mongodb, template_name, company_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch template for versioning: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Database error" }),
                    )
                })?
                .ok_or((
                    StatusCode::NOT_FOUND,
                    json!({ "error": "Template not found" }),
                ))?;

        // 2. Archive current state as a version
        let version_doc = logs_db::TemplateVersionDocument {
            template_name: current_template.template_name.clone(),
            company_id: current_template.company_id.clone(),
            version: current_template.version,
            version_name: current_template.version_name,
            template_layout: current_template.template_layout.clone(),
            schedule: current_template.schedule,
            created_at: chrono::Utc::now(),
            created_by: mongodb::bson::Uuid::parse_str(user_id).unwrap_or(current_template.created_by), 
        };

        logs_db::add_template_version(&state.mongodb, &version_doc)
            .await
            .map_err(|e| {
                tracing::error!("Failed to archive template version: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to create version snapshot" }),
                )
            })?;

        // 3. Update template (this increments version in DB)
        logs_db::update_template(
            &state.mongodb,
            template_name,
            company_id,
            schedule,
            template_layout,
            version_name,
        )
        .await
        .map_err(|e: anyhow::Error| {
            tracing::error!("Failed to update template: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to update template" }),
            )
        })?;
        Ok(())
    }

    /// Retrieves version history for a template.
    ///
    /// # Errors
    /// Returns an error if database query fails.
    pub async fn get_versions(
        state: &AppState,
        company_id: &str,
        template_name: &str,
    ) -> Result<Vec<logs_db::TemplateVersionDocument>, (StatusCode, serde_json::Value)> {
        logs_db::get_template_versions(&state.mongodb, company_id, template_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch template versions: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })
    }

    /// Restores a specific version of a template.
    ///
    /// # Errors
    /// Returns an error if version not found or update fails.
    pub async fn restore_version(
        state: &AppState,
        company_id: &str,
        template_name: &str,
        version: u16,
        user_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        // 1. Fetch target version
        let target_version = logs_db::get_template_version(&state.mongodb, company_id, template_name, version)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch target version: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Version not found" }),
            ))?;

        // 2. Call update_template with the target data
        // This handles archiving the CURRENT state before overwriting it with the OLD state
        Self::update_template(
            state,
            company_id,
            template_name,
            Some(&target_version.template_layout),
            Some(&target_version.schedule),
            user_id,
            Some(format!("Restored from version {}", version)),
        ).await
    }

    /// Renames a log template.
    ///
    /// # Errors
    /// Returns an error if the database update fails.
    pub async fn rename_template(
        state: &AppState,
        company_id: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        logs_db::rename_template(&state.mongodb, old_name, new_name, company_id)
            .await
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to rename template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to rename template" }),
                )
            })?;
        Ok(())
    }

    /// Deletes a log template.
    ///
    /// # Errors
    /// Returns an error if the database deletion fails.
    pub async fn delete_template(
        state: &AppState,
        company_id: &str,
        template_name: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        logs_db::delete_template(&state.mongodb, template_name, company_id)
            .await
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to delete template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to delete template" }),
                )
            })?;
        Ok(())
    }
}
