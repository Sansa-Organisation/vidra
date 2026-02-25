use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryTier {
    Anonymous,
    Identified,
    Diagnostics,
    Off,
}

impl TelemetryTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            TelemetryTier::Anonymous => "anonymous",
            TelemetryTier::Identified => "identified",
            TelemetryTier::Diagnostics => "diagnostics",
            TelemetryTier::Off => "off",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryConfig {
    pub tier: TelemetryTier,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetrySnapshot {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tier: TelemetryTier,
    pub project_root: String,

    pub receipts_queued: usize,
    pub receipts_sent: usize,

    pub uploads_queued: usize,
    pub uploads_sent: usize,

    pub jobs_queued: usize,
}

fn resolve_home_dir() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("VIDRA_HOME_DIR") {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return Ok(p);
        }
        return Ok(std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p));
    }
    dirs::home_dir().context("failed to resolve home dir")
}

pub fn vidra_root_dir() -> Result<PathBuf> {
    Ok(resolve_home_dir()?.join(".vidra"))
}

pub fn telemetry_root_dir() -> Result<PathBuf> {
    Ok(vidra_root_dir()?.join("telemetry"))
}

pub fn telemetry_config_path() -> Result<PathBuf> {
    Ok(telemetry_root_dir()?.join("config.json"))
}

pub fn receipts_root_dir() -> Result<PathBuf> {
    Ok(vidra_root_dir()?.join("receipts"))
}

pub fn jobs_root_dir() -> Result<PathBuf> {
    Ok(vidra_root_dir()?.join("jobs"))
}

pub fn load_telemetry_config() -> Result<TelemetryConfig> {
    let path = telemetry_config_path()?;
    if !path.exists() {
        return Ok(TelemetryConfig {
            tier: TelemetryTier::Identified,
            updated_at: chrono::Utc::now(),
        });
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read telemetry config: {}", path.display()))?;
    let cfg: TelemetryConfig = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse telemetry config: {}", path.display()))?;
    Ok(cfg)
}

pub fn save_telemetry_config(tier: TelemetryTier) -> Result<TelemetryConfig> {
    let dir = telemetry_root_dir()?;
    std::fs::create_dir_all(&dir).context("failed to create telemetry dir")?;

    let cfg = TelemetryConfig {
        tier,
        updated_at: chrono::Utc::now(),
    };
    let path = telemetry_config_path()?;
    let json =
        serde_json::to_string_pretty(&cfg).context("failed to serialize telemetry config")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write telemetry config: {}", path.display()))?;
    Ok(cfg)
}

pub fn compute_snapshot(project_root: &Path) -> Result<TelemetrySnapshot> {
    let cfg = load_telemetry_config()?;

    let receipts_root = receipts_root_dir()?;
    let receipt_status = crate::sync_tools::receipt_sync_status(&receipts_root)
        .unwrap_or(crate::sync_tools::ReceiptSyncStatus { queued: 0, sent: 0 });

    let upload_status = crate::sync_tools::upload_sync_status(project_root)
        .unwrap_or(crate::sync_tools::UploadSyncStatus { queued: 0, sent: 0 });

    let jobs_root = jobs_root_dir()?;
    let jobs_queued = crate::jobs_tools::list_queued_jobs(&jobs_root)
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(TelemetrySnapshot {
        created_at: chrono::Utc::now(),
        tier: cfg.tier,
        project_root: project_root.to_string_lossy().to_string(),
        receipts_queued: receipt_status.queued,
        receipts_sent: receipt_status.sent,
        uploads_queued: upload_status.queued,
        uploads_sent: upload_status.sent,
        jobs_queued,
    })
}

fn add_file_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    name: &str,
    bytes: &[u8],
) -> Result<()> {
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(name, options)
        .with_context(|| format!("failed to start zip entry: {}", name))?;
    zip.write_all(bytes)
        .with_context(|| format!("failed to write zip entry: {}", name))?;
    Ok(())
}

pub fn export_telemetry_zip(project_root: &Path, out_path: &Path) -> Result<()> {
    let cfg = load_telemetry_config()?;
    let snapshot = compute_snapshot(project_root)?;

    let mut file = std::fs::File::create(out_path)
        .with_context(|| format!("failed to create export zip: {}", out_path.display()))?;
    let mut zip = zip::ZipWriter::new(&mut file);

    let cfg_json =
        serde_json::to_vec_pretty(&cfg).context("failed to serialize telemetry config")?;
    add_file_to_zip(&mut zip, "telemetry/config.json", &cfg_json)?;

    let snap_json =
        serde_json::to_vec_pretty(&snapshot).context("failed to serialize telemetry snapshot")?;
    add_file_to_zip(&mut zip, "telemetry/snapshot.json", &snap_json)?;

    // Export current project asset manifest if present.
    let manifest_path = project_root.join(".vidra").join("asset_manifest.json");
    if manifest_path.exists() {
        if let Ok(bytes) = std::fs::read(&manifest_path) {
            add_file_to_zip(&mut zip, "project/asset_manifest.json", &bytes)?;
        }
    }

    // Export receipts (queued and sent) as raw JSON for transparency.
    let receipts_root = receipts_root_dir()?;
    for (subdir, label) in [
        (receipts_root.clone(), "queued"),
        (receipts_root.join("sent"), "sent"),
    ] {
        if !subdir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&subdir)
            .with_context(|| format!("failed to read receipts dir: {}", subdir.display()))?
        {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("receipt.json");
            if let Ok(bytes) = std::fs::read(&path) {
                add_file_to_zip(&mut zip, &format!("receipts/{}/{}", label, name), &bytes)?;
            }
        }
    }

    // Export upload queue metadata (not blobs).
    let uploads_root = project_root.join(".vidra").join("uploads");
    for (sub, label) in [
        (uploads_root.join("queued"), "queued"),
        (uploads_root.join("sent"), "sent"),
    ] {
        if !sub.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&sub)
            .with_context(|| format!("failed to read uploads dir: {}", sub.display()))?
        {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("upload.json");
            if let Ok(bytes) = std::fs::read(&path) {
                add_file_to_zip(&mut zip, &format!("uploads/{}/{}", label, name), &bytes)?;
            }
        }
    }

    zip.finish().context("failed to finalize telemetry zip")?;
    Ok(())
}

pub fn delete_local_telemetry_data() -> Result<()> {
    // Remove ~/.vidra/telemetry (config/export intermediates). Keep receipts/uploads because those are functional queues.
    let root = telemetry_root_dir()?;
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .with_context(|| format!("failed to delete telemetry dir: {}", root.display()))?;
    }

    // Record a local deletion request marker.
    let marker = vidra_root_dir()?.join("telemetry_deletion_requested.txt");
    let msg = format!(
        "telemetry deletion requested at {}\n",
        chrono::Utc::now().to_rfc3339()
    );
    let _ = std::fs::create_dir_all(vidra_root_dir()?);
    let _ = std::fs::write(&marker, msg);
    Ok(())
}

pub fn parse_tier(tier: &str) -> Result<TelemetryTier> {
    match tier {
        "anonymous" => Ok(TelemetryTier::Anonymous),
        "identified" => Ok(TelemetryTier::Identified),
        "diagnostics" => Ok(TelemetryTier::Diagnostics),
        "off" => Ok(TelemetryTier::Off),
        _ => anyhow::bail!(
            "Invalid telemetry tier. Choose: anonymous, identified, diagnostics, off."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_config_roundtrip() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_telemetry_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        let cfg = save_telemetry_config(TelemetryTier::Anonymous).unwrap();
        assert_eq!(cfg.tier, TelemetryTier::Anonymous);
        let read = load_telemetry_config().unwrap();
        assert_eq!(read.tier, TelemetryTier::Anonymous);

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn telemetry_export_zip_creates_entries() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp =
            std::env::temp_dir().join(format!("vidra_telemetry_export_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        // Create a fake project root with a .vidra/uploads queue
        let project_root = tmp.join("proj");
        std::fs::create_dir_all(project_root.join(".vidra").join("uploads").join("queued"))
            .unwrap();
        std::fs::write(
            project_root
                .join(".vidra")
                .join("uploads")
                .join("queued")
                .join("upload_test.json"),
            "{}",
        )
        .unwrap();

        // Create one queued receipt
        let receipts = receipts_root_dir().unwrap();
        std::fs::create_dir_all(&receipts).unwrap();
        std::fs::write(receipts.join("rr_test.json"), "{}").unwrap();

        let out = tmp.join("export.zip");
        export_telemetry_zip(&project_root, &out).unwrap();
        assert!(out.exists());
        assert!(std::fs::metadata(&out).unwrap().len() > 0);

        let f = std::fs::File::open(&out).unwrap();
        let mut z = zip::ZipArchive::new(f).unwrap();
        let mut names = Vec::new();
        for i in 0..z.len() {
            names.push(z.by_index(i).unwrap().name().to_string());
        }
        assert!(names.iter().any(|n| n == "telemetry/config.json"));
        assert!(names.iter().any(|n| n == "telemetry/snapshot.json"));

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
