use crate::{
    AppState,
    db::{UserRecord, UserRole},
    logs_db, utils,
};
use axum::http::StatusCode;
use serde_json::json;

#[cfg(test)]
mod template_service_tests {
    use super::*;

    // Helper to create a valid template field
    fn valid_field() -> logs_db::TemplateField {
        logs_db::TemplateField {
            field_type: "text".to_string(),
            position: logs_db::Position { x: 10.0, y: 20.0 },
            props: logs_db::TemplateFieldProps {
                text: Some("Test Field".to_string()),
                input_type: Some("text".to_string()),
                min_length: Some(0),
                max_length: Some(100),
                required: Some(false),
                color: Some("red".to_string()),
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_validate_template_layout_valid_field() {
        let layout = vec![valid_field()];
        assert!(TemplateService::validate_template_layout(&layout).is_ok());
    }

    #[test]
    fn test_validate_template_layout_invalid_input_type() {
        let mut field = valid_field();
        field.props.input_type = Some("invalid_type".to_string());
        let layout = vec![field];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (status, err) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(err["error"].to_string().contains("Invalid input type"));
    }

    #[test]
    fn test_validate_template_layout_negative_min_length() {
        let mut field = valid_field();
        field.props.min_length = Some(-1);
        let layout = vec![field];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (status, err) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            err["error"]
                .to_string()
                .contains("min_length must be non-negative")
        );
    }

    #[test]
    fn test_validate_template_layout_negative_max_length() {
        let mut field = valid_field();
        field.props.max_length = Some(-5);
        let layout = vec![field];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (status, err) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            err["error"]
                .to_string()
                .contains("max_length must be non-negative")
        );
    }

    #[test]
    fn test_validate_template_layout_min_greater_than_max() {
        let mut field = valid_field();
        field.props.min_length = Some(100);
        field.props.max_length = Some(50);
        let layout = vec![field];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (status, err) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            err["error"]
                .to_string()
                .contains("min_length cannot be greater than max_length")
        );
    }

    #[test]
    fn test_validate_template_layout_invalid_color() {
        let mut field = valid_field();
        field.props.color = Some("red;display:none".to_string());
        let layout = vec![field];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (status, err) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(err["error"].to_string().contains("Invalid color value"));
    }

    #[test]
    fn test_validate_template_layout_valid_input_types() {
        for input_type in &["text", "int", "float"] {
            let mut field = valid_field();
            field.props.input_type = Some(input_type.to_string());
            let layout = vec![field];
            assert!(
                TemplateService::validate_template_layout(&layout).is_ok(),
                "Failed for input_type: {}",
                input_type
            );
        }
    }

    #[test]
    fn test_validate_template_layout_valid_length_combinations() {
        // No constraints
        let mut field = valid_field();
        field.props.min_length = None;
        field.props.max_length = None;
        let layout = vec![field];
        assert!(TemplateService::validate_template_layout(&layout).is_ok());

        // Only min_length
        let mut field = valid_field();
        field.props.min_length = Some(5);
        field.props.max_length = None;
        let layout = vec![field];
        assert!(TemplateService::validate_template_layout(&layout).is_ok());

        // Only max_length
        let mut field = valid_field();
        field.props.min_length = None;
        field.props.max_length = Some(50);
        let layout = vec![field];
        assert!(TemplateService::validate_template_layout(&layout).is_ok());

        // Both equal
        let mut field = valid_field();
        field.props.min_length = Some(10);
        field.props.max_length = Some(10);
        let layout = vec![field];
        assert!(TemplateService::validate_template_layout(&layout).is_ok());
    }

    #[test]
    fn test_validate_template_layout_multiple_fields() {
        let field1 = valid_field();
        let field2 = valid_field();
        let field3 = valid_field();
        let layout = vec![field1, field2, field3];

        assert!(TemplateService::validate_template_layout(&layout).is_ok());
    }

    #[test]
    fn test_validate_template_layout_error_identifies_field() {
        let field1 = valid_field();
        let mut field2 = valid_field();
        field2.props.input_type = Some("invalid".to_string());
        let field3 = valid_field();
        let layout = vec![field1, field2, field3];

        let result = TemplateService::validate_template_layout(&layout);
        assert!(result.is_err());
        let (_, err) = result.unwrap_err();
        // Error should mention field 1 (0-indexed)
        assert!(err["error"].to_string().contains("Field 1"));
    }

    #[tokio::test]
    async fn test_template_service_basic() {
        assert!(true);
    }
}

pub struct TemplateService;

impl TemplateService {
    /// Validates that all template fields contain safe values and valid constraints.
    /// Checks for CSS injection, input types, and length constraints.
    /// Returns an error if any field contains invalid data.
    fn validate_template_layout(
        template_layout: &logs_db::TemplateLayout,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        for (field_index, field) in template_layout.iter().enumerate() {
            // Validate color field if present
            if let Some(color) = &field.props.color {
                if !utils::is_valid_css_color(color) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        json!({ "error": format!("Field {}: Invalid color value. Colors must be valid CSS values (hex, rgb/rgba, or named colors).", field_index) }),
                    ));
                }
            }

            // Validate font_family field if present
            if let Some(font_family) = &field.props.font_family {
                if !utils::is_valid_font_family(font_family) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        json!({ "error": format!("Field {}: Invalid font family value.", field_index) }),
                    ));
                }
            }

            // Validate text_decoration field if present
            if let Some(text_decoration) = &field.props.text_decoration {
                if !utils::is_valid_text_decoration(text_decoration) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        json!({ "error": format!("Field {}: Invalid text decoration value.", field_index) }),
                    ));
                }
            }

            // Validate input_type if present
            if let Some(input_type) = &field.props.input_type {
                if !utils::is_valid_input_type(input_type) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        json!({ "error": format!("Field {}: Invalid input type '{}'. Must be one of: text, int, float.", field_index, input_type) }),
                    ));
                }
            }

            // Validate length constraints (min_length and max_length)
            if let Err(e) =
                utils::validate_length_constraints(field.props.min_length, field.props.max_length)
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    json!({ "error": format!("Field {}: {}", field_index, e) }),
                ));
            }
        }
        Ok(())
    }
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
        branch_id: Option<String>,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        // Validate template layout for malicious content
        Self::validate_template_layout(&template_layout)?;

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
            branch_id,
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
    ) -> Result<
        (
            String,
            logs_db::TemplateLayout,
            u16,
            Option<String>,
            Option<String>,
        ),
        (StatusCode, serde_json::Value),
    > {
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
            Some(t) => Ok((
                t.template_name,
                t.template_layout,
                t.version,
                t.version_name,
                t.branch_id,
            )),
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
        branch_id: Option<&str>,
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
        let templates =
            logs_db::get_templates_by_company_and_branch(&state.mongodb, company_id, branch_id)
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
        branch_id: Option<&str>, // Caller's branch_id
        is_company_manager: bool,
        target_branch_id: Option<Option<&str>>,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        // Validate template layout for malicious content if provided
        if let Some(layout) = template_layout {
            Self::validate_template_layout(layout)?;
        }

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

        // Authorization check
        if user.is_branch_manager() {
            if current_template.is_company_wide() {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "Branch managers cannot update company-wide templates" }),
                ));
            }
            if current_template.branch_id.as_deref() != branch_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "Branch managers can only update templates for their own branch" }),
                ));
            }
        }

        let resolved_branch_id = if is_company_manager {
            target_branch_id
        } else {
            None
        };

        // 2. Archive current state as a version
        let version_doc = logs_db::TemplateVersionDocument {
            template_name: current_template.template_name.clone(),
            company_id: current_template.company_id.clone(),
            branch_id: current_template.branch_id.clone(),
            version: current_template.version,
            version_name: current_template.version_name,
            template_layout: current_template.template_layout.clone(),
            schedule: current_template.schedule,
            created_at: chrono::Utc::now(),
            created_by: mongodb::bson::Uuid::parse_str(user_id)
                .unwrap_or(current_template.created_by),
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
            resolved_branch_id,
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
        template_name: &str,
        user: &UserRecord,
    ) -> Result<Vec<logs_db::TemplateVersionDocument>, (StatusCode, serde_json::Value)> {
        let company_id = user.company_id.as_ref().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "User record missing company ID" }),
        ))?;
        let template_versions =
            logs_db::get_template_versions(&state.mongodb, company_id, template_name)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch template versions: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Database error" }),
                    )
                });
        let last_version = template_versions
            .as_ref()
            .ok()
            .and_then(|versions| versions.last());
        if user.role == UserRole::BranchManager
            && (last_version.map_or(false, |t| t.branch_id.as_deref() != user.branch_id.as_deref()))
        {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Unauthorized to view versions of this template" }),
            ));
        }
        Ok(template_versions?)
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
        branch_id: Option<&str>,
        is_company_manager: bool,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        // 1. Fetch target version
        let target_version =
            logs_db::get_template_version(&state.mongodb, company_id, template_name, version)
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
            Some(format!("Restored from version {version}")),
            branch_id,
            is_company_manager,
            None,
        )
        .await
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
        user_branch_id: Option<&str>,
        user_role: &UserRole,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let template = logs_db::get_template_by_name(&state.mongodb, old_name, company_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch template for rename: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Template not found" }),
            ))?;

        if *user_role == UserRole::BranchManager
            && (template.is_company_wide() || template.branch_id.as_deref() != user_branch_id)
        {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Unauthorized to rename this template" }),
            ));
        }

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
        user_branch_id: Option<&str>,
        user_role: &UserRole,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let template = logs_db::get_template_by_name(&state.mongodb, template_name, company_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch template for delete: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Template not found" }),
            ))?;

        if *user_role == UserRole::BranchManager
            && (template.is_company_wide() || template.branch_id.as_deref() != user_branch_id)
        {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Unauthorized to delete this template" }),
            ));
        }

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
