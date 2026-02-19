use crate::db;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

pub struct ClockService;

impl ClockService {
    /// Clocks in the user. Returns error if already clocked in.
    ///
    /// # Errors
    /// Returns an error if the user is already clocked in or DB operations fail.
    pub async fn clock_in(
        pool: &PgPool,
        user_id: &str,
        company_id: &str,
    ) -> Result<db::ClockEvent, (StatusCode, serde_json::Value)> {
        // Check if user already has an open clock-in
        let current = db::get_clock_status(pool, user_id).await.map_err(|e| {
            tracing::error!("Database error checking clock status: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Database error"}),
            )
        })?;

        if let Some(ref event) = current
            && event.status == "in"
        {
            return Err((
                StatusCode::CONFLICT,
                json!({"error": "You are already clocked in"}),
            ));
        }

        let event = db::clock_in(pool, user_id, company_id).await.map_err(|e| {
            tracing::error!("Database error clocking in: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Failed to clock in"}),
            )
        })?;

        Ok(event)
    }

    /// Clocks out the user. Returns error if not currently clocked in.
    ///
    /// # Errors
    /// Returns an error if the user is not clocked in or DB operations fail.
    pub async fn clock_out(
        pool: &PgPool,
        user_id: &str,
    ) -> Result<db::ClockEvent, (StatusCode, serde_json::Value)> {
        let event = db::clock_out(pool, user_id).await.map_err(|e| {
            tracing::error!("Database error clocking out: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Failed to clock out"}),
            )
        })?;

        event.ok_or((
            StatusCode::BAD_REQUEST,
            json!({"error": "You are not currently clocked in"}),
        ))
    }

    /// Gets the current clock status and recent events for a user.
    ///
    /// # Errors
    /// Returns an error if DB operations fail.
    pub async fn get_status(
        pool: &PgPool,
        user_id: &str,
    ) -> Result<(Option<db::ClockEvent>, Vec<db::ClockEvent>), (StatusCode, serde_json::Value)>
    {
        let current = db::get_clock_status(pool, user_id).await.map_err(|e| {
            tracing::error!("Database error fetching clock status: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Database error"}),
            )
        })?;

        let recent = db::get_recent_clock_events(pool, user_id, 5)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching recent clock events: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Database error"}),
                )
            })?;

        Ok((current, recent))
    }

    pub async fn get_company_clock_events(
        pool: &PgPool,
        company_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<db::CompanyClockEventRow>, (StatusCode, serde_json::Value)> {
        let events = db::get_company_clock_events(pool, company_id, from, to)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching company clock events: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Database error"}),
                )
            })?;
        Ok(events)
    }
}
