use anyhow::Result;
use chrono::Datelike;
use futures_util::TryStreamExt;
use mongodb::bson::Uuid;
use schemars::JsonSchema;
use utoipa::ToSchema;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, ToSchema, JsonSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, ToSchema, JsonSchema)]
pub struct TemplateFieldProps {
    pub text: Option<String>,
    pub size: Option<String>,
    pub weight: Option<String>,
    pub value: Option<String>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub unit: Option<String>,
    pub selected: Option<String>,
    pub options: Option<Vec<String>>,
    pub editable: Option<bool>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, ToSchema, JsonSchema)]
pub struct TemplateField {
    pub field_type: String,
    pub position: Position,
    pub props: TemplateFieldProps,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema, JsonSchema)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct Schedule {
    pub frequency: Frequency,
    pub days_of_week: Option<Vec<u8>>, // for daily schedule (one per day)
    pub day_of_week: Option<u8>,       // for weekly schedule (one per week)
    pub day_of_month: Option<u8>,      // for monthly schedule (one per month)
    pub month_of_year: Option<u8>,     // for yearly schedule (one per year)
}

pub type TemplateLayout = Vec<TemplateField>;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateDocument {
    pub template_name: String,
    pub template_layout: TemplateLayout,
    pub company_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub schedule: Schedule,
    pub created_by: Uuid,
    #[serde(default = "default_version")]
    pub version: u16,
}

fn default_version() -> u16 {
    1
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateVersionDocument {
    pub template_name: String,
    pub company_id: String,
    pub version: u16,
    pub version_name: Option<String>,
    pub template_layout: TemplateLayout,
    pub schedule: Schedule,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
}

/// Initializes the `MongoDB` client.
///
/// # Errors
/// Returns an error if the connection fails.
///
/// # Panics
/// Panics if `MONGODB_URI` environment variable is not set.
pub async fn init_mongodb() -> Result<mongodb::Client> {
    let mongo_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set in environment");
    mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .map_err(Into::into)
}

/// Adds a new log template to the database.
///
/// # Errors
/// Returns an error if the database operation fails.
pub async fn add_template(client: &mongodb::Client, template: &TemplateDocument) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    collection.insert_one(template).await?;
    Ok(())
}

/// Adds a new template version to the database.
///
/// # Errors
/// Returns an error if the database operation fails.
pub async fn add_template_version(
    client: &mongodb::Client,
    version_doc: &TemplateVersionDocument,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateVersionDocument> =
        db.collection("template_versions");

    collection.insert_one(version_doc).await?;
    Ok(())
}

/// Retrieves all versions for a specific template.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_template_versions(
    client: &mongodb::Client,
    company_id: &str,
    template_name: &str,
) -> Result<Vec<TemplateVersionDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateVersionDocument> =
        db.collection("template_versions");

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
        "template_name": template_name,
    };

    let find_options = mongodb::options::FindOptions::builder()
        .sort(mongodb::bson::doc! { "version": -1 })
        .build();

    let mut cursor = collection.find(filter).with_options(find_options).await?;
    let mut versions = Vec::new();

    while let Some(version) = cursor.try_next().await? {
        versions.push(version);
    }

    Ok(versions)
}

/// Retrieves a specific template version.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_template_version(
    client: &mongodb::Client,
    company_id: &str,
    template_name: &str,
    version: u16,
) -> Result<Option<TemplateVersionDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateVersionDocument> =
        db.collection("template_versions");

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
        "template_name": template_name,
        "version": u32::from(version),
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

/// Retrieves a log template by its name and company ID.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_template_by_name(
    client: &mongodb::Client,
    template_name: &str,
    company_id: &str,
) -> Result<Option<TemplateDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "template_name": template_name,
        "company_id": company_id,
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

/// Retrieves all log templates for a specific company.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_templates_by_company(
    client: &mongodb::Client,
    company_id: &str,
) -> Result<Vec<TemplateDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
    };

    let mut cursor = collection.find(filter).await?;
    let mut templates = Vec::new();

    while let Some(template) = cursor.try_next().await? {
        templates.push(template);
    }

    Ok(templates)
}

/// Updates an existing log template.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn update_template(
    client: &mongodb::Client,
    template_name: &str,
    company_id: &str,
    schedule: Option<&Schedule>,
    layout: Option<&TemplateLayout>,
) -> Result<()> {
    if schedule.is_none() && layout.is_none() {
        return Ok(());
    }
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "template_name": template_name,
        "company_id": company_id,
    };

    let mut set_doc = mongodb::bson::Document::new();

    if let Some(schedule) = schedule {
        set_doc.insert("schedule", mongodb::bson::to_bson(&schedule)?);
    }
    if let Some(layout) = layout {
        set_doc.insert("template_layout", mongodb::bson::to_bson(&layout)?);
    }
    set_doc.insert("updated_at", mongodb::bson::to_bson(&chrono::Utc::now())?);
    
    // Increment version
    let update = mongodb::bson::doc! {
        "$set": set_doc,
        "$inc": { "version": 1 }
    };

    collection.update_one(filter, update).await?;
    Ok(())
}

/// Updates an existing log template with specific version increment.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn update_template_with_version(
    client: &mongodb::Client,
    template_name: &str,
    company_id: &str,
    schedule: Option<&Schedule>,
    layout: Option<&TemplateLayout>,
) -> Result<()> {
    // This is now redundant given update_template handles versioning, 
    // but we can keep the original signature compatible by forwarding
    update_template(client, template_name, company_id, schedule, layout).await
}

/// Renames a log template.
///
/// # Errors
/// Returns an error if a template with the new name already exists or if the database update fails.
pub async fn rename_template(
    client: &mongodb::Client,
    old_name: &str,
    new_name: &str,
    company_id: &str,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let existing_template = collection
        .find_one(mongodb::bson::doc! {
            "template_name": new_name,
            "company_id": company_id,
        })
        .await?
        .is_some();

    if existing_template {
        anyhow::bail!("Template with the new name already exists");
    }

    let filter = mongodb::bson::doc! {
        "template_name": old_name,
        "company_id": company_id,
    };

    let update = mongodb::bson::doc! {
        "$set": {
            "template_name": new_name,
            "updated_at": mongodb::bson::to_bson(&chrono::Utc::now())?,
        }
    };

    collection.update_one(filter, update).await?;
    Ok(())
}

/// Deletes a log template.
///
/// # Errors
/// Returns an error if the database deletion fails.
pub async fn delete_template(
    client: &mongodb::Client,
    template_name: &str,
    company_id: &str,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "template_name": template_name,
        "company_id": company_id,
    };

    collection.delete_one(filter).await?;
    Ok(())
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct LogEntry {
    pub entry_id: String,
    pub template_name: String,
    pub company_id: String,
    pub user_id: String,
    pub entry_data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub period: String,
}

/// Creates a new log entry.
///
/// # Errors
/// Returns an error if the database operation fails.
pub async fn create_log_entry(client: &mongodb::Client, entry: &LogEntry) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    collection.insert_one(entry).await?;
    Ok(())
}

/// Retrieves a log entry by its ID.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_log_entry(client: &mongodb::Client, entry_id: &str) -> Result<Option<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

/// Retrieves all log entries for a user in a company.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_user_log_entries(
    client: &mongodb::Client,
    user_id: &str,
    company_id: &str,
) -> Result<Vec<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "user_id": user_id,
        "company_id": company_id,
    };

    let mut cursor = collection.find(filter).await?;
    let mut entries = Vec::new();

    while let Some(entry) = cursor.try_next().await? {
        entries.push(entry);
    }

    Ok(entries)
}

/// Retrieves log entries for a user, filtered by template.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_user_log_entries_by_template(
    client: &mongodb::Client,
    user_id: &str,
    company_id: &str,
    template_name: &str,
) -> Result<Vec<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "user_id": user_id,
        "company_id": company_id,
        "template_name": template_name,
    };

    let mut cursor = collection.find(filter).await?;
    let mut entries = Vec::new();

    while let Some(entry) = cursor.try_next().await? {
        entries.push(entry);
    }

    Ok(entries)
}

/// Retrieves the most recently submitted log entry for a user and template.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_latest_submitted_entry(
    client: &mongodb::Client,
    user_id: &str,
    company_id: &str,
    template_name: &str,
) -> Result<Option<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "user_id": user_id,
        "company_id": company_id,
        "template_name": template_name,
        "status": "submitted",
    };

    let result = collection
        .find_one(filter)
        .sort(mongodb::bson::doc! { "submitted_at": -1 })
        .await?;
    Ok(result)
}

/// Checks if a log entry exists for the current period and template.
///
/// # Errors
/// Returns an error if the database query fails.
///
/// # Panics
/// Panics if period boundary calculations fail.
pub async fn has_entry_for_current_period(
    client: &mongodb::Client,
    company_id: &str,
    template_name: &str,
    frequency: &Frequency,
) -> Result<bool> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let now = chrono::Utc::now();
    let (period_start, period_end) = match frequency {
        Frequency::Daily => {
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_else(|| panic!("Error finding day start: {now}"))
                .and_utc();
            let end = now
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap_or_else(|| panic!("Error finding day end: {now}"))
                .and_utc();
            (start, end)
        }
        Frequency::Weekly => {
            let days_since_monday = now.weekday().num_days_from_monday();
            let start = (now.date_naive() - chrono::Duration::days(i64::from(days_since_monday)))
                .and_hms_opt(0, 0, 0)
                .unwrap_or_else(|| panic!("Error finding week start date: {now}"))
                .and_utc();
            let end = (start.date_naive() + chrono::Duration::days(6))
                .and_hms_opt(23, 59, 59)
                .unwrap_or_else(|| panic!("Error finding week end date: {now}"))
                .and_utc();
            (start, end)
        }
        Frequency::Monthly => {
            let start = now
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let next_month = if now.month() == 12 {
                now.date_naive()
                    .with_year(now.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
            } else {
                now.date_naive().with_month(now.month() + 1).unwrap()
            };
            let end = (next_month - chrono::Duration::days(1))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
        Frequency::Yearly => {
            let start = now
                .date_naive()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = now
                .date_naive()
                .with_month(12)
                .unwrap()
                .with_day(31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
    };

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
        "template_name": template_name,
        "created_at": {
            "$gte": mongodb::bson::to_bson(&period_start)?,
            "$lte": mongodb::bson::to_bson(&period_end)?,
        },
    };

    let result = collection.find_one(filter).await?;
    Ok(result.is_some())
}

/// Checks if a submitted log entry exists for the current period and template.
///
/// # Errors
/// Returns an error if the database query fails.
///
/// # Panics
/// Panics if period boundary calculations fail.
pub async fn has_submitted_entry_for_current_period(
    client: &mongodb::Client,
    company_id: &str,
    template_name: &str,
    frequency: &Frequency,
) -> Result<bool> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let now = chrono::Utc::now();
    let (period_start, period_end) = match frequency {
        Frequency::Daily => {
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_else(|| panic!("Error finding day start: {now}"))
                .and_utc();
            let end = now
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap_or_else(|| panic!("Error finding day end: {now}"))
                .and_utc();
            (start, end)
        }
        Frequency::Weekly => {
            let days_since_monday = now.weekday().num_days_from_monday();
            let start = (now.date_naive() - chrono::Duration::days(i64::from(days_since_monday)))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = (start.date_naive() + chrono::Duration::days(6))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
        Frequency::Monthly => {
            let start = now
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let next_month = if now.month() == 12 {
                now.date_naive()
                    .with_year(now.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
            } else {
                now.date_naive().with_month(now.month() + 1).unwrap()
            };
            let end = (next_month - chrono::Duration::days(1))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
        Frequency::Yearly => {
            let start = now
                .date_naive()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = now
                .date_naive()
                .with_month(12)
                .unwrap()
                .with_day(31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
    };

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
        "template_name": template_name,
        "status": "submitted",
        "created_at": {
            "$gte": mongodb::bson::to_bson(&period_start)?,
            "$lte": mongodb::bson::to_bson(&period_end)?,
        },
    };

    let result = collection.find_one(filter).await?;
    Ok(result.is_some())
}

/// Retrieves a draft log entry for the current period and template.
///
/// # Errors
/// Returns an error if the database query fails.
///
/// # Panics
/// Panics if period boundary calculations fail.
pub async fn get_draft_entry_for_current_period(
    client: &mongodb::Client,
    user_id: &str,
    company_id: &str,
    template_name: &str,
    frequency: &Frequency,
) -> Result<Option<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let now = chrono::Utc::now();
    let (period_start, period_end) = match frequency {
        Frequency::Daily => {
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_else(|| panic!("Error finding day start: {now}"))
                .and_utc();
            let end = now
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap_or_else(|| panic!("Error finding day end: {now}"))
                .and_utc();
            (start, end)
        }
        Frequency::Weekly => {
            let days_since_monday = now.weekday().num_days_from_monday();
            let start = (now.date_naive() - chrono::Duration::days(i64::from(days_since_monday)))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = (start.date_naive() + chrono::Duration::days(6))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
        Frequency::Monthly => {
            let start = now
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let next_month = if now.month() == 12 {
                now.date_naive()
                    .with_year(now.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
            } else {
                now.date_naive().with_month(now.month() + 1).unwrap()
            };
            let end = (next_month - chrono::Duration::days(1))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
        Frequency::Yearly => {
            let start = now
                .date_naive()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let end = now
                .date_naive()
                .with_month(12)
                .unwrap()
                .with_day(31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            (start, end)
        }
    };

    let filter = mongodb::bson::doc! {
        "user_id": user_id,
        "company_id": company_id,
        "template_name": template_name,
        "status": "draft",
        "created_at": {
            "$gte": mongodb::bson::to_bson(&period_start)?,
            "$lte": mongodb::bson::to_bson(&period_end)?,
        },
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

/// Updates the data of an existing log entry.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn update_log_entry(
    client: &mongodb::Client,
    entry_id: &str,
    entry_data: &serde_json::Value,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    let update = mongodb::bson::doc! {
        "$set": {
            "entry_data": mongodb::bson::to_bson(&entry_data)?,
            "updated_at": mongodb::bson::to_bson(&chrono::Utc::now())?,
        }
    };

    collection.update_one(filter, update).await?;
    Ok(())
}

/// Submits a log entry, marking it as final.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn submit_log_entry(client: &mongodb::Client, entry_id: &str) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    let update = mongodb::bson::doc! {
        "$set": {
            "status": "submitted",
            "submitted_at": mongodb::bson::to_bson(&chrono::Utc::now())?,
            "updated_at": mongodb::bson::to_bson(&chrono::Utc::now())?,
        }
    };

    collection.update_one(filter, update).await?;
    Ok(())
}

/// Returns a submitted log entry to draft status.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn unsubmit_log_entry(client: &mongodb::Client, entry_id: &str) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    let update = mongodb::bson::doc! {
        "$set": {
            "status": "draft",
            "submitted_at": mongodb::bson::Bson::Null,
            "updated_at": mongodb::bson::to_bson(&chrono::Utc::now())?,
        }
    };

    collection.update_one(filter, update).await?;
    Ok(())
}

/// Deletes a log entry.
///
/// # Errors
/// Returns an error if the database deletion fails.
pub async fn delete_log_entry(client: &mongodb::Client, entry_id: &str) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    collection.delete_one(filter).await?;
    Ok(())
}

#[must_use]
pub fn is_form_due_today(schedule: &Schedule) -> bool {
    let today = chrono::Local::now();
    let weekday = today.weekday();

    match schedule.frequency {
        Frequency::Daily => {
            if let Some(days) = &schedule.days_of_week {
                let day_num = match weekday {
                    chrono::Weekday::Mon => 0,
                    chrono::Weekday::Tue => 1,
                    chrono::Weekday::Wed => 2,
                    chrono::Weekday::Thu => 3,
                    chrono::Weekday::Fri => 4,
                    chrono::Weekday::Sat => 5,
                    chrono::Weekday::Sun => 6,
                };
                days.contains(&day_num)
            } else {
                true
            }
        }
        Frequency::Weekly => {
            if let Some(day) = schedule.day_of_week {
                let day_num = match weekday {
                    chrono::Weekday::Mon => 0,
                    chrono::Weekday::Tue => 1,
                    chrono::Weekday::Wed => 2,
                    chrono::Weekday::Thu => 3,
                    chrono::Weekday::Fri => 4,
                    chrono::Weekday::Sat => 5,
                    chrono::Weekday::Sun => 6,
                };
                day_num == day
            } else {
                false
            }
        }
        Frequency::Monthly => {
            if let Some(day) = schedule.day_of_month {
                today.day() >= u32::from(day)
            } else {
                false
            }
        }
        Frequency::Yearly => {
            if let Some(month) = schedule.month_of_year {
                if let Some(day) = schedule.day_of_month {
                    let target_month = u32::from(month);
                    let target_day = u32::from(day);

                    match today.month().cmp(&target_month) {
                        std::cmp::Ordering::Greater => true,
                        std::cmp::Ordering::Less => false,
                        std::cmp::Ordering::Equal => today.day() >= target_day,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

/// Formats the period string for a given frequency.
///
/// # Panics
/// Panics if week boundary calculations fail.
#[must_use]
pub fn format_period_for_frequency(frequency: &Frequency) -> String {
    let now = chrono::Utc::now();

    match frequency {
        Frequency::Daily => now.format("%d/%m/%y").to_string(),
        Frequency::Weekly => {
            let days_since_monday = now.weekday().num_days_from_monday();
            let week_start = (now.date_naive()
                - chrono::Duration::days(i64::from(days_since_monday)))
            .and_hms_opt(0, 0, 0)
            .unwrap_or_else(|| panic!("Error finding week start date: {now}"))
            .and_utc();
            let week_end = (week_start.date_naive() + chrono::Duration::days(6))
                .and_hms_opt(23, 59, 59)
                .unwrap_or_else(|| panic!("Error finding week end date: {now}"))
                .and_utc();

            format!(
                "{}-{}/{}/{}",
                week_start.day(),
                week_end.day(),
                week_end.month(),
                week_end.format("%y")
            )
        }
        Frequency::Monthly => now.format("%m/%y").to_string(),
        Frequency::Yearly => now.format("%Y").to_string(),
    }
}

#[must_use]
pub fn process_template_layout_with_period(
    layout: &TemplateLayout,
    frequency: &Frequency,
) -> TemplateLayout {
    let period = format_period_for_frequency(frequency);

    layout
        .iter()
        .map(|field| {
            let mut processed_field = field.clone();
            if let Some(text) = &field.props.text
                && text.contains("{period}")
            {
                let new_text = text.replace("{period}", &period);
                tracing::info!(
                    "Field type: {}, Original text: '{}', Replaced text: '{}'",
                    field.field_type,
                    text,
                    new_text
                );
                processed_field.props.text = Some(new_text);
            }
            processed_field
        })
        .collect()
}

#[must_use]
pub fn process_template_layout_with_period_string(
    layout: &TemplateLayout,
    period: &str,
) -> TemplateLayout {
    layout
        .iter()
        .map(|field| {
            let mut processed_field = field.clone();
            if let Some(text) = &field.props.text {
                processed_field.props.text = Some(text.replace("{period}", period));
            }
            processed_field
        })
        .collect()
}
