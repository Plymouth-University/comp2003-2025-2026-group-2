use crate::{AppState, db, logs_db};
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod log_entry_service_tests {
    #[tokio::test]
    async fn test_log_entry_service_basic() {
        assert!(true);
    }
}

pub struct LogEntryService;

impl LogEntryService {
    /// Creates a new log entry draft based on a template.
    ///
    /// # Errors
    /// Returns an error if the user has no company, the template is not found, or an entry already exists for the period.
    pub async fn create_log_entry(
        state: &AppState,
        user_id: &str,
        template_name: &str,
    ) -> Result<String, (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_id(&state.postgres, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "User not found" }),
            ))?;

        let company_id = user.company_id.ok_or((
            StatusCode::FORBIDDEN,
            json!({ "error": "User is not associated with a company" }),
        ))?;

        let template = logs_db::get_template_by_name(&state.mongodb, template_name, &company_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get template: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get template" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Template not found" }),
            ))?;

        // Check if template is for the correct branch
        if let Some(template_branch_id) = &template.branch_id
            && Some(template_branch_id) != user.branch_id.as_ref()
        {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Template is not available for your branch" }),
            ));
        }

        let has_entry = logs_db::has_entry_for_current_period(
            &state.mongodb,
            &company_id,
            template_name,
            &template.schedule.frequency,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check for existing entries: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to check for existing entries" }),
            )
        })?;

        if has_entry {
            let period = match template.schedule.frequency {
                logs_db::Frequency::Daily => "today",
                logs_db::Frequency::Weekly => "this week",
                logs_db::Frequency::Monthly => "this month",
                logs_db::Frequency::Yearly => "this year",
            };
            return Err((
                StatusCode::CONFLICT,
                json!({ "error": format!("A log entry for this template has already been created {}", period) }),
            ));
        }

        let entry_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let period = logs_db::format_period_for_frequency(&template.schedule.frequency);

        let entry = logs_db::LogEntry {
            entry_id: entry_id.clone(),
            template_name: template_name.to_string(),
            company_id,
            branch_id: user.branch_id,
            user_id: user_id.to_string(),
            entry_data: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            submitted_at: None,
            status: "draft".to_string(),
            period,
        };

        logs_db::create_log_entry(&state.mongodb, &entry)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to create log entry" }),
                )
            })?;

        Ok(entry_id)
    }

    /// Retrieves a specific log entry.
    ///
    /// # Errors
    /// Returns an error if the entry is not found or if the user doesn't have permission to view it.
    pub async fn get_log_entry(
        state: &AppState,
        user_id: &str,
        entry_id: &str,
    ) -> Result<logs_db::LogEntry, (StatusCode, serde_json::Value)> {
        let entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        // Check if user owns the entry or has management permissions (including readonly HQ)
        if entry.user_id != user_id {
            let user = db::get_user_by_id(&state.postgres, user_id)
                .await
                .map_err(|e| {
                    tracing::error!("Database error fetching user: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Database error" }),
                    )
                })?
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    json!({ "error": "User not found" }),
                ))?;

            // Allow if user can manage branch or is readonly HQ
            if !user.can_read_manage_branch() {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "You do not have permission to view this entry" }),
                ));
            }

            // Additional check: ensure entry belongs to same company
            let user_company_id = user.company_id.ok_or((
                StatusCode::FORBIDDEN,
                json!({ "error": "User is not associated with a company" }),
            ))?;

            if entry.company_id != user_company_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "You do not have permission to view this entry" }),
                ));
            }
        }

        Ok(entry)
    }

    /// Updates an existing log entry draft.
    ///
    /// # Errors
    /// Returns an error if the entry is not found, user doesn't own it, or update fails.
    pub async fn update_log_entry(
        state: &AppState,
        user_id: &str,
        entry_id: &str,
        entry_data: &serde_json::Value,
    ) -> Result<logs_db::LogEntry, (StatusCode, serde_json::Value)> {
        let entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        if entry.user_id != user_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "You do not have permission to update this entry" }),
            ));
        }

        logs_db::update_log_entry(&state.mongodb, entry_id, entry_data)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to update log entry" }),
                )
            })?;

        let updated_entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch updated log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to fetch updated entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        Ok(updated_entry)
    }

    /// Submits a log entry, marking it as final.
    ///
    /// # Errors
    /// Returns an error if the entry is not found, user doesn't own it, or submission fails.
    pub async fn submit_log_entry(
        state: &AppState,
        user_id: &str,
        entry_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        if entry.user_id != user_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "You do not have permission to submit this entry" }),
            ));
        }

        logs_db::submit_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to submit log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to submit log entry" }),
                )
            })?;

        Ok(())
    }

    /// Returns a submitted log entry to draft status (admin only).
    ///
    /// # Errors
    /// Returns an error if the user is not an admin, entry is not found, or operation fails.
    pub async fn unsubmit_log_entry(
        state: &AppState,
        user_id: &str,
        entry_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_id(&state.postgres, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "User not found" }),
            ))?;

        if !user.can_manage_company() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only admin users can unsubmit log entries" }),
            ));
        }

        let entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        let user_company_id = db::get_user_company_id(&state.postgres, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching user company ID: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?;

        if let Some(company_id) = user_company_id {
            if entry.company_id != company_id && !user.is_logsmart_admin() {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "You do not have permission to unsubmit this entry" }),
                ));
            }
        } else if !user.is_logsmart_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "User is not associated with a company" }),
            ));
        }

        logs_db::unsubmit_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to unsubmit log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to unsubmit log entry" }),
                )
            })?;

        Ok(())
    }

    /// Deletes a log entry.
    ///
    /// # Errors
    /// Returns an error if the entry is not found, user is not authorized, or deletion fails.
    pub async fn delete_log_entry(
        state: &AppState,
        user_id: &str,
        entry_id: &str,
        is_company_admin: bool,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let entry = logs_db::get_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entry" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "Entry not found" })))?;

        if entry.user_id != user_id && !is_company_admin {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "You do not have permission to delete this entry" }),
            ));
        }

        logs_db::delete_log_entry(&state.mongodb, entry_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete log entry: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to delete log entry" }),
                )
            })?;

        Ok(())
    }

    /// Lists all log templates for a company.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn list_due_forms(
        state: &AppState,
        company_id: &str,
        branch_id: Option<&str>,
    ) -> Result<Vec<logs_db::TemplateDocument>, (StatusCode, serde_json::Value)> {
        logs_db::get_templates_by_company_and_branch(&state.mongodb, company_id, branch_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get templates: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get templates" }),
                )
            })
    }

    /// Retrieves all log entries for a user in a company.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub async fn get_user_log_entries(
        state: &AppState,
        user_id: &str,
        company_id: &str,
    ) -> Result<Vec<logs_db::LogEntry>, (StatusCode, serde_json::Value)> {
        logs_db::get_user_log_entries(&state.mongodb, user_id, company_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get log entries: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to get log entries" }),
                )
            })
    }
}
