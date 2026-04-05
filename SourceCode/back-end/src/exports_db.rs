use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

const EXPORT_RETENTION_DAYS: i64 = 7;

pub struct ExportRecord {
    pub company_id: String,
    pub file_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn exports_dir() -> PathBuf {
    std::env::var("EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("exports"))
}

fn safe_path(dir: &PathBuf, filename: &str) -> Result<PathBuf> {
    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let path = dir.join(safe_name);

    if path.is_absolute() || !path.starts_with(dir) {
        anyhow::bail!("Invalid filename");
    }

    Ok(path)
}

pub async fn init_exports_dir() -> Result<()> {
    let dir = exports_dir();
    fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("Failed to create exports directory: {}", dir.display()))?;
    Ok(())
}

pub async fn save_export(company_id: &str, data: &[u8]) -> Result<String> {
    let dir = exports_dir();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let filename = format!("{company_id}_{timestamp}.zip");
    let file_path = safe_path(&dir, &filename)?;

    fs::write(&file_path, data)
        .await
        .with_context(|| format!("Failed to write export file: {}", file_path.display()))?;

    tracing::info!("Export saved: {} ({} bytes)", filename, data.len());
    Ok(filename)
}

pub async fn get_export(_company_id: &str, filename: &str) -> Result<Option<Vec<u8>>> {
    let dir = exports_dir();
    let file_path = safe_path(&dir, filename)?;

    if !file_path.exists() {
        return Ok(None);
    }

    let content = fs::read(&file_path)
        .await
        .with_context(|| format!("Failed to read export file: {}", file_path.display()))?;

    Ok(Some(content))
}

pub async fn delete_export(filename: &str) -> Result<()> {
    let dir = exports_dir();
    let file_path = safe_path(&dir, filename)?;

    if file_path.exists() {
        fs::remove_file(&file_path)
            .await
            .with_context(|| format!("Failed to delete export file: {}", file_path.display()))?;
    }

    Ok(())
}

pub async fn cleanup_old_exports() -> Result<u32> {
    let dir = exports_dir();
    let cutoff = chrono::Utc::now() - chrono::Duration::days(EXPORT_RETENTION_DAYS);
    let mut deleted = 0u32;

    let mut entries = fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("zip") {
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let modified = metadata.modified().ok().and_then(|t| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(
                t.duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs() as i64),
                0,
            )
        });

        if let Some(modified) = modified
            && modified < cutoff
        {
            if let Err(e) = fs::remove_file(&path).await {
                tracing::warn!("Failed to delete old export {}: {e}", path.display());
            } else {
                deleted += 1;
            }
        }
    }

    if deleted > 0 {
        tracing::info!("Cleaned up {deleted} old export files");
    }

    Ok(deleted)
}

pub fn spawn_cleanup_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_old_exports().await {
                tracing::error!("Export cleanup failed: {e}");
            }
        }
    });
}
