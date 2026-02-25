use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use vidra_ir::asset::AssetRegistry;

pub struct ReceiptSyncStatus {
    pub queued: usize,
    pub sent: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetManifestEntry {
    pub id: String,
    pub asset_type: String,
    pub path: String,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub is_remote_url: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetManifest {
    pub project_id: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub assets: Vec<AssetManifestEntry>,
}

pub struct AssetManifestStats {
    pub total: usize,
    pub missing: usize,
    pub hashed: usize,
    pub remote_urls: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadQueueEntry {
    pub original_path: String,
    pub blob_sha256: String,
    pub size_bytes: u64,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

pub struct UploadSyncStatus {
    pub queued: usize,
    pub sent: usize,
}

#[derive(Debug, Clone)]
pub struct UploadQueueItem {
    #[allow(dead_code)]
    pub status: &'static str,
    pub entry: UploadQueueEntry,
}

pub fn receipts_root_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".vidra").join("receipts"))
}

pub fn receipts_sent_dir(receipts_root: &Path) -> PathBuf {
    receipts_root.join("sent")
}

pub fn uploads_root_dir(project_root: &Path) -> PathBuf {
    project_root.join(".vidra").join("uploads")
}

pub fn uploads_queued_dir(uploads_root: &Path) -> PathBuf {
    uploads_root.join("queued")
}

pub fn uploads_sent_dir(uploads_root: &Path) -> PathBuf {
    uploads_root.join("sent")
}

pub fn uploads_blobs_dir(uploads_root: &Path) -> PathBuf {
    uploads_root.join("blobs")
}

pub fn upload_sync_status(project_root: &Path) -> Result<UploadSyncStatus> {
    let root = uploads_root_dir(project_root);
    let queued_dir = uploads_queued_dir(&root);
    let sent_dir = uploads_sent_dir(&root);

    let mut queued = 0usize;
    let mut sent = 0usize;

    if queued_dir.exists() {
        for entry in std::fs::read_dir(&queued_dir).context("failed to read queued uploads dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                queued += 1;
            }
        }
    }

    if sent_dir.exists() {
        for entry in std::fs::read_dir(&sent_dir).context("failed to read sent uploads dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                sent += 1;
            }
        }
    }

    Ok(UploadSyncStatus { queued, sent })
}

pub fn receipt_sync_status(receipts_root: &Path) -> Result<ReceiptSyncStatus> {
    let sent_dir = receipts_sent_dir(receipts_root);

    let mut queued = 0usize;
    let mut sent = 0usize;

    if receipts_root.exists() {
        for entry in std::fs::read_dir(receipts_root).context("failed to read receipts dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                queued += 1;
            }
        }
    }

    if sent_dir.exists() {
        for entry in std::fs::read_dir(&sent_dir).context("failed to read sent receipts dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                sent += 1;
            }
        }
    }

    Ok(ReceiptSyncStatus { queued, sent })
}

/// "Push" receipts by moving queued receipts into `sent/`.
///
/// This is intentionally local-only; cloud upload is out of scope for now.
pub fn push_receipts_local(receipts_root: &Path) -> Result<usize> {
    std::fs::create_dir_all(receipts_root).context("failed to create receipts root")?;
    let sent_dir = receipts_sent_dir(receipts_root);
    std::fs::create_dir_all(&sent_dir).context("failed to create sent receipts dir")?;

    let mut moved = 0usize;
    for entry in std::fs::read_dir(receipts_root).context("failed to read receipts dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("receipt.json");
        let dest = sent_dir.join(file_name);

        // If a receipt with same name already exists in sent/, keep the newer one by suffixing.
        let dest = if dest.exists() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("receipt");
            let mut i = 1u32;
            loop {
                let candidate = sent_dir.join(format!("{}_{}.json", stem, i));
                if !candidate.exists() {
                    break candidate;
                }
                i += 1;
            }
        } else {
            dest
        };

        std::fs::rename(&path, &dest).or_else(|_| {
            // Cross-device fallback: copy + delete
            std::fs::copy(&path, &dest)
                .context("failed to copy receipt")
                .and_then(|_| {
                    std::fs::remove_file(&path).context("failed to delete original receipt")
                })
        })?;
        moved += 1;
    }

    Ok(moved)
}

pub fn enqueue_upload_path(project_root: &Path, path: &Path) -> Result<usize> {
    let uploads_root = uploads_root_dir(project_root);
    let queued_dir = uploads_queued_dir(&uploads_root);
    let blobs_dir = uploads_blobs_dir(&uploads_root);
    std::fs::create_dir_all(&queued_dir).context("failed to create queued uploads dir")?;
    std::fs::create_dir_all(&blobs_dir).context("failed to create upload blobs dir")?;

    let mut enqueued = 0usize;
    enqueue_upload_path_inner(project_root, path, &queued_dir, &blobs_dir, &mut enqueued)?;
    Ok(enqueued)
}

fn enqueue_upload_path_inner(
    project_root: &Path,
    path: &Path,
    queued_dir: &Path,
    blobs_dir: &Path,
    enqueued: &mut usize,
) -> Result<()> {
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let vidra_dir = project_root.join(".vidra");
    if canon.starts_with(&vidra_dir) {
        return Ok(());
    }

    if canon.is_dir() {
        for entry in std::fs::read_dir(&canon)
            .with_context(|| format!("failed to read dir: {}", canon.display()))?
        {
            let entry = entry?;
            enqueue_upload_path_inner(
                project_root,
                &entry.path(),
                queued_dir,
                blobs_dir,
                enqueued,
            )?;
        }
        return Ok(());
    }

    if !canon.is_file() {
        return Ok(());
    }

    let bytes = std::fs::read(&canon)
        .with_context(|| format!("failed to read file: {}", canon.display()))?;
    let sha = sha256_bytes(&bytes);
    let blob_path = blobs_dir.join(format!("{}.bin", sha));
    if !blob_path.exists() {
        std::fs::write(&blob_path, &bytes)
            .with_context(|| format!("failed to write upload blob: {}", blob_path.display()))?;
    }

    let meta_path = queued_dir.join(format!("upload_{}.json", sha));
    if meta_path.exists() {
        return Ok(());
    }

    let entry = UploadQueueEntry {
        original_path: canon.to_string_lossy().to_string(),
        blob_sha256: format!("sha256:{}", sha),
        size_bytes: bytes.len() as u64,
        added_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string_pretty(&entry).context("failed to serialize upload entry")?;
    std::fs::write(&meta_path, json)
        .with_context(|| format!("failed to write upload metadata: {}", meta_path.display()))?;

    *enqueued += 1;
    Ok(())
}

pub fn push_uploads_local(project_root: &Path) -> Result<usize> {
    let uploads_root = uploads_root_dir(project_root);
    let queued_dir = uploads_queued_dir(&uploads_root);
    let sent_dir = uploads_sent_dir(&uploads_root);
    std::fs::create_dir_all(&queued_dir).context("failed to create queued uploads dir")?;
    std::fs::create_dir_all(&sent_dir).context("failed to create sent uploads dir")?;

    let mut moved = 0usize;
    for entry in std::fs::read_dir(&queued_dir).context("failed to read queued uploads dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("upload.json");
        let dest = sent_dir.join(file_name);
        let dest = if dest.exists() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("upload");
            let mut i = 1u32;
            loop {
                let candidate = sent_dir.join(format!("{}_{}.json", stem, i));
                if !candidate.exists() {
                    break candidate;
                }
                i += 1;
            }
        } else {
            dest
        };

        std::fs::rename(&path, &dest).or_else(|_| {
            std::fs::copy(&path, &dest)
                .context("failed to copy upload metadata")
                .and_then(|_| {
                    std::fs::remove_file(&path).context("failed to delete original upload metadata")
                })
        })?;
        moved += 1;
    }
    Ok(moved)
}

pub fn list_upload_queue(project_root: &Path) -> Result<Vec<UploadQueueItem>> {
    let uploads_root = uploads_root_dir(project_root);
    let queued_dir = uploads_queued_dir(&uploads_root);
    let sent_dir = uploads_sent_dir(&uploads_root);

    let mut out = Vec::new();
    out.extend(read_upload_entries_from_dir("queued", &queued_dir)?);
    out.extend(read_upload_entries_from_dir("sent", &sent_dir)?);

    out.sort_by(|a, b| a.entry.added_at.cmp(&b.entry.added_at));
    Ok(out)
}

fn read_upload_entries_from_dir(status: &'static str, dir: &Path) -> Result<Vec<UploadQueueItem>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read uploads dir: {}", dir.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read upload metadata: {}", path.display()))?;
        let entry: UploadQueueEntry = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse upload metadata JSON: {}", path.display()))?;
        out.push(UploadQueueItem { status, entry });
    }
    Ok(out)
}

/// Resolve a blob path from the upload queue by either:
/// - matching basename of `original_path` (e.g. `logo.png`)
/// - matching full blob sha (`sha256:...`) or hex digest prefix
///
/// Returns (blob_path, suggested_filename).
pub fn resolve_upload_blob(project_root: &Path, name: &str) -> Result<Option<(PathBuf, String)>> {
    let items = list_upload_queue(project_root)?;
    let uploads_root = uploads_root_dir(project_root);
    let blobs_dir = uploads_blobs_dir(&uploads_root);

    let needle = name.trim();
    let needle_hex = needle
        .strip_prefix("sha256:")
        .unwrap_or(needle)
        .to_lowercase();

    for item in items {
        let original = std::path::PathBuf::from(&item.entry.original_path);
        let base = original
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let blob_hex = item
            .entry
            .blob_sha256
            .strip_prefix("sha256:")
            .unwrap_or(&item.entry.blob_sha256)
            .to_lowercase();

        let matches = base == needle || blob_hex == needle_hex || blob_hex.starts_with(&needle_hex);
        if !matches {
            continue;
        }

        let blob_path = blobs_dir.join(format!("{}.bin", blob_hex));
        if blob_path.exists() {
            return Ok(Some((blob_path, base.to_string())));
        }
    }
    Ok(None)
}

pub fn generate_asset_manifest(
    project_id: &str,
    assets: &AssetRegistry,
) -> Result<(AssetManifest, AssetManifestStats)> {
    let mut out = Vec::new();
    let mut missing = 0usize;
    let mut hashed = 0usize;
    let mut remote_urls = 0usize;

    for asset in assets.all() {
        let path_str = asset.path.to_string_lossy().to_string();
        let is_remote_url = path_str.starts_with("http://") || path_str.starts_with("https://");
        if is_remote_url {
            remote_urls += 1;
        }

        let exists = asset.path.exists();
        if !exists {
            missing += 1;
        }

        let (size_bytes, sha256) = if exists && !asset.path.is_dir() {
            let meta = std::fs::metadata(&asset.path).ok();
            let size = meta.as_ref().map(|m| m.len());
            let hash = sha256_file(&asset.path).ok();
            if hash.is_some() {
                hashed += 1;
            }
            (size, hash)
        } else {
            (None, None)
        };

        out.push(AssetManifestEntry {
            id: asset.id.to_string(),
            asset_type: asset.asset_type.to_string(),
            path: path_str,
            exists,
            size_bytes,
            sha256: sha256.map(|h| format!("sha256:{}", h)),
            is_remote_url,
        });
    }

    // Deterministic ordering
    out.sort_by(|a, b| a.id.cmp(&b.id));

    let manifest = AssetManifest {
        project_id: project_id.to_string(),
        generated_at: chrono::Utc::now(),
        assets: out,
    };
    let stats = AssetManifestStats {
        total: manifest.assets.len(),
        missing,
        hashed,
        remote_urls,
    };
    Ok((manifest, stats))
}

pub fn write_asset_manifest(path: &Path, manifest: &AssetManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create asset manifest dir: {}", parent.display())
        })?;
    }
    let json =
        serde_json::to_string_pretty(manifest).context("failed to serialize asset manifest")?;
    std::fs::write(path, json)
        .with_context(|| format!("failed to write asset manifest: {}", path.display()))?;
    Ok(())
}

pub fn read_asset_manifest(path: &Path) -> Result<AssetManifest> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read asset manifest: {}", path.display()))?;
    let manifest: AssetManifest = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse asset manifest JSON: {}", path.display()))?;
    Ok(manifest)
}

pub fn asset_manifest_stats_from_manifest(manifest: &AssetManifest) -> AssetManifestStats {
    let mut missing = 0usize;
    let mut hashed = 0usize;
    let mut remote_urls = 0usize;
    for a in &manifest.assets {
        if !a.exists {
            missing += 1;
        }
        if a.sha256.is_some() {
            hashed += 1;
        }
        if a.is_remote_url {
            remote_urls += 1;
        }
    }
    AssetManifestStats {
        total: manifest.assets.len(),
        missing,
        hashed,
        remote_urls,
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)
        .with_context(|| format!("failed to open file for hashing: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = f
            .read(&mut buf)
            .context("failed to read file for hashing")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{:02x}", b)).collect())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_moves_receipts_to_sent() {
        let tmp = std::env::temp_dir().join(format!("vidra_sync_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        std::fs::write(tmp.join("rr_a.json"), "{}").unwrap();
        std::fs::write(tmp.join("rr_b.json"), "{}").unwrap();

        let before = receipt_sync_status(&tmp).unwrap();
        assert_eq!(before.queued, 2);
        assert_eq!(before.sent, 0);

        let moved = push_receipts_local(&tmp).unwrap();
        assert_eq!(moved, 2);

        let after = receipt_sync_status(&tmp).unwrap();
        assert_eq!(after.queued, 0);
        assert_eq!(after.sent, 2);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn asset_manifest_hashes_existing_files() {
        let tmp = std::env::temp_dir().join(format!("vidra_asset_manifest_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let p = tmp.join("a.bin");
        std::fs::write(&p, b"hello").unwrap();

        let mut reg = AssetRegistry::new();
        reg.register(vidra_ir::asset::Asset::new(
            vidra_ir::asset::AssetId::new("a"),
            vidra_ir::asset::AssetType::Image,
            p.clone(),
        ));

        let (m, stats) = generate_asset_manifest("proj", &reg).unwrap();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.missing, 0);
        assert_eq!(stats.hashed, 1);
        assert_eq!(
            m.assets[0]
                .sha256
                .as_deref()
                .unwrap()
                .starts_with("sha256:"),
            true
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn asset_manifest_roundtrip_read_write() {
        let tmp =
            std::env::temp_dir().join(format!("vidra_asset_manifest_rw_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let manifest = AssetManifest {
            project_id: "proj".to_string(),
            generated_at: chrono::Utc::now(),
            assets: vec![AssetManifestEntry {
                id: "a".to_string(),
                asset_type: "image".to_string(),
                path: "assets/a.png".to_string(),
                exists: false,
                size_bytes: None,
                sha256: None,
                is_remote_url: false,
            }],
        };

        let out = tmp.join("asset_manifest.json");
        write_asset_manifest(&out, &manifest).unwrap();
        let read = read_asset_manifest(&out).unwrap();
        assert_eq!(read.project_id, "proj");
        assert_eq!(read.assets.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn upload_queue_enqueues_and_pushes() {
        let root = std::env::temp_dir().join(format!("vidra_upload_queue_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("file.txt");
        std::fs::write(&file, b"hello upload").unwrap();

        let enq = enqueue_upload_path(&root, &file).unwrap();
        assert_eq!(enq, 1);
        let s1 = upload_sync_status(&root).unwrap();
        assert_eq!(s1.queued, 1);

        let pushed = push_uploads_local(&root).unwrap();
        assert_eq!(pushed, 1);
        let s2 = upload_sync_status(&root).unwrap();
        assert_eq!(s2.queued, 0);
        assert_eq!(s2.sent, 1);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_upload_blob_by_basename() {
        let root =
            std::env::temp_dir().join(format!("vidra_upload_resolve_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("logo.png");
        std::fs::write(&file, b"pngbytes").unwrap();
        enqueue_upload_path(&root, &file).unwrap();

        let resolved = resolve_upload_blob(&root, "logo.png").unwrap();
        assert!(resolved.is_some());

        let _ = std::fs::remove_dir_all(&root);
    }
}
