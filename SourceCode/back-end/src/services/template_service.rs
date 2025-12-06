use crate::{AppState, logs_db};
use axum::http::StatusCode;
use serde_json::json;

pub struct TemplateService;

impl TemplateService {
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

    pub async fn get_template(
        state: &AppState,
        company_id: &str,
        template_name: &str,
    ) -> Result<(String, logs_db::TemplateLayout), (StatusCode, serde_json::Value)> {
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
            Some(t) => Ok((t.template_name, t.template_layout)),
            None => Err((
                StatusCode::NOT_FOUND,
                json!({ "error": "Template not found" }),
            )),
        }
    }

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

    pub async fn update_template(
        state: &AppState,
        company_id: &str,
        template_name: &str,
        template_layout: Option<&logs_db::TemplateLayout>,
        schedule: Option<&logs_db::Schedule>,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        logs_db::update_template(
            &state.mongodb,
            template_name,
            company_id,
            schedule,
            template_layout,
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
