use anyhow::Result;
use chrono::Datelike;
use futures_util::TryStreamExt;
use mongodb::bson::Uuid;
use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
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
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateField {
    pub field_type: String,
    pub position: Position,
    pub props: TemplateFieldProps,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
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
}

pub async fn init_mongodb() -> Result<mongodb::Client> {
    let mongo_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set in environment");
    mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .map_err(Into::into)
}

pub async fn add_template(client: &mongodb::Client, template: &TemplateDocument) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    collection.insert_one(template).await?;
    Ok(())
}

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

    let updated_template = mongodb::bson::doc! {
        "$set": set_doc
    };

    collection.update_one(filter, updated_template).await?;
    Ok(())
}

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
}

pub async fn create_log_entry(
    client: &mongodb::Client,
    entry: &LogEntry,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    collection.insert_one(entry).await?;
    Ok(())
}

pub async fn get_log_entry(
    client: &mongodb::Client,
    entry_id: &str,
) -> Result<Option<LogEntry>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

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

pub async fn submit_log_entry(
    client: &mongodb::Client,
    entry_id: &str,
) -> Result<()> {
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

pub async fn delete_log_entry(
    client: &mongodb::Client,
    entry_id: &str,
) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<LogEntry> = db.collection("log_entries");

    let filter = mongodb::bson::doc! {
        "entry_id": entry_id,
    };

    collection.delete_one(filter).await?;
    Ok(())
}

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
                today.day() == u32::from(day)
            } else {
                false
            }
        }
        Frequency::Yearly => {
            let day_match = if let Some(day) = schedule.day_of_month {
                today.day() == u32::from(day)
            } else {
                false
            };
            let month_match = if let Some(month) = schedule.month_of_year {
                today.month() == u32::from(month)
            } else {
                false
            };
            day_match && month_match
        }
    }
}
