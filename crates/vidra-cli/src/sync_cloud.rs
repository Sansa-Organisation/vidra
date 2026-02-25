use anyhow::{Context, Result};
use reqwest::blocking::{multipart, Client};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::sync_tools;

pub fn cloud_base_url_from_env() -> Option<String> {
    std::env::var("VIDRA_CLOUD_URL")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
}

pub struct CloudPushReport {
    pub receipts_uploaded: usize,
    pub uploads_uploaded: usize,
    pub receipt_failures: Vec<String>,
    pub upload_failures: Vec<String>,
}

#[derive(Debug)]
pub struct CloudItemPushResult {
    pub uploaded: usize,
    pub failures: Vec<String>,
}

const MAX_RETRIES: usize = 3;
const BACKOFF_MS: u64 = 50;

fn sleep_backoff(attempt: usize) {
    let ms = BACKOFF_MS.saturating_mul((attempt + 1) as u64);
    std::thread::sleep(Duration::from_millis(ms));
}

fn post_json_with_retries(client: &Client, url: &str, body: &str) -> Result<()> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..MAX_RETRIES {
        let res = client
            .post(url)
            .header("content-type", "application/json")
            .body(body.to_string())
            .send();

        match res {
            Ok(res) => {
                if res.status().is_success() {
                    return Ok(());
                }
                let status = res.status();
                let text = res.text().unwrap_or_default();
                last_err = Some(anyhow::anyhow!("{}: {}", status, text));
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!(e).context("request failed"));
            }
        }
        if attempt + 1 < MAX_RETRIES {
            sleep_backoff(attempt);
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("request failed")))
}

fn get_text_with_retries(client: &Client, url: &str) -> Result<String> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..MAX_RETRIES {
        let res = client.get(url).send();
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    return Ok(res.text().unwrap_or_default());
                }
                let status = res.status();
                let text = res.text().unwrap_or_default();
                last_err = Some(anyhow::anyhow!("{}: {}", status, text));
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!(e).context("request failed"));
            }
        }
        if attempt + 1 < MAX_RETRIES {
            sleep_backoff(attempt);
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("request failed")))
}

fn get_bytes_with_retries(client: &Client, url: &str) -> Result<Vec<u8>> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..MAX_RETRIES {
        let res = client.get(url).send();
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    return Ok(res.bytes().unwrap_or_default().to_vec());
                }
                let status = res.status();
                let text = res.text().unwrap_or_default();
                last_err = Some(anyhow::anyhow!("{}: {}", status, text));
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!(e).context("request failed"));
            }
        }
        if attempt + 1 < MAX_RETRIES {
            sleep_backoff(attempt);
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("request failed")))
}

pub struct CloudPullReport {
    pub receipts_downloaded: usize,
    pub uploads_downloaded: usize,
    pub receipt_failures: Vec<String>,
    pub upload_failures: Vec<String>,
}

pub fn pull_receipts_from_cloud(
    receipts_root: &Path,
    base_url: &str,
) -> Result<CloudItemPushResult> {
    std::fs::create_dir_all(receipts_root).context("failed to create receipts root")?;
    let sent_dir = sync_tools::receipts_sent_dir(receipts_root);
    std::fs::create_dir_all(&sent_dir).context("failed to create sent receipts dir")?;

    let client = Client::new();
    let url = format!("{}/api/v1/receipts", base_url.trim_end_matches('/'));

    let mut downloaded = 0usize;
    let mut failures: Vec<String> = Vec::new();

    let body = get_text_with_retries(&client, &url)?;
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .with_context(|| format!("invalid receipts JSON from {}", url))?;
    let Some(arr) = parsed.as_array() else {
        anyhow::bail!("expected receipts to be a JSON array");
    };

    for item in arr {
        let receipt_id = item
            .get("receipt_id")
            .and_then(|v| v.as_str())
            .unwrap_or("receipt");
        let out_path = sent_dir.join(format!("{}.json", receipt_id));
        if out_path.exists() {
            continue;
        }
        match serde_json::to_string_pretty(item) {
            Ok(s) => {
                if let Err(e) = std::fs::write(&out_path, s) {
                    failures.push(format!("{}: {}", out_path.display(), e));
                } else {
                    downloaded += 1;
                }
            }
            Err(e) => failures.push(format!("{}: {}", receipt_id, e)),
        }
    }

    Ok(CloudItemPushResult {
        uploaded: downloaded,
        failures,
    })
}

pub fn pull_uploads_from_cloud(project_root: &Path, base_url: &str) -> Result<CloudItemPushResult> {
    let uploads_root = sync_tools::uploads_root_dir(project_root);
    let sent_dir = sync_tools::uploads_sent_dir(&uploads_root);
    let blobs_dir = sync_tools::uploads_blobs_dir(&uploads_root);

    std::fs::create_dir_all(&sent_dir).context("failed to create sent uploads dir")?;
    std::fs::create_dir_all(&blobs_dir).context("failed to create upload blobs dir")?;

    let client = Client::new();
    let list_url = format!("{}/api/v1/uploads", base_url.trim_end_matches('/'));
    let mut downloaded = 0usize;
    let mut failures: Vec<String> = Vec::new();

    let body = get_text_with_retries(&client, &list_url)?;
    let entries: Vec<sync_tools::UploadQueueEntry> = serde_json::from_str(&body)
        .with_context(|| format!("invalid uploads JSON from {}", list_url))?;

    for entry in entries {
        let hex = entry
            .blob_sha256
            .strip_prefix("sha256:")
            .unwrap_or(&entry.blob_sha256);
        let blob_path = blobs_dir.join(format!("{}.bin", hex));
        if !blob_path.exists() {
            let blob_url = format!(
                "{}/api/v1/uploads/blob/{}",
                base_url.trim_end_matches('/'),
                hex
            );
            match get_bytes_with_retries(&client, &blob_url) {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&blob_path, bytes) {
                        failures.push(format!("{}: {}", blob_path.display(), e));
                        continue;
                    }
                }
                Err(e) => {
                    failures.push(format!("{}: {}", hex, e));
                    continue;
                }
            }
        }

        let meta_path = sent_dir.join(format!("upload_{}.json", hex));
        if meta_path.exists() {
            continue;
        }
        match serde_json::to_string_pretty(&entry) {
            Ok(s) => {
                if let Err(e) = std::fs::write(&meta_path, s) {
                    failures.push(format!("{}: {}", meta_path.display(), e));
                } else {
                    downloaded += 1;
                }
            }
            Err(e) => failures.push(format!("{}: {}", hex, e)),
        }
    }

    Ok(CloudItemPushResult {
        uploaded: downloaded,
        failures,
    })
}

pub fn pull_all_from_cloud(project_root: &Path) -> Result<CloudPullReport> {
    let Some(base_url) = cloud_base_url_from_env() else {
        return Ok(CloudPullReport {
            receipts_downloaded: 0,
            uploads_downloaded: 0,
            receipt_failures: Vec::new(),
            upload_failures: Vec::new(),
        });
    };

    let receipts = if let Some(receipts_root) = sync_tools::receipts_root_dir() {
        pull_receipts_from_cloud(&receipts_root, &base_url)?
    } else {
        CloudItemPushResult {
            uploaded: 0,
            failures: Vec::new(),
        }
    };
    let uploads = pull_uploads_from_cloud(project_root, &base_url)?;

    Ok(CloudPullReport {
        receipts_downloaded: receipts.uploaded,
        uploads_downloaded: uploads.uploaded,
        receipt_failures: receipts.failures,
        upload_failures: uploads.failures,
    })
}

fn post_multipart_with_retries<F>(client: &Client, url: &str, mut build_form: F) -> Result<()>
where
    F: FnMut() -> Result<multipart::Form>,
{
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..MAX_RETRIES {
        let form = build_form()?;
        let res = client.post(url).multipart(form).send();
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    return Ok(());
                }
                let status = res.status();
                let text = res.text().unwrap_or_default();
                last_err = Some(anyhow::anyhow!("{}: {}", status, text));
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!(e).context("request failed"));
            }
        }
        if attempt + 1 < MAX_RETRIES {
            sleep_backoff(attempt);
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("request failed")))
}

pub fn push_receipts_to_cloud(receipts_root: &Path, base_url: &str) -> Result<CloudItemPushResult> {
    std::fs::create_dir_all(receipts_root).context("failed to create receipts root")?;
    let sent_dir = sync_tools::receipts_sent_dir(receipts_root);
    std::fs::create_dir_all(&sent_dir).context("failed to create sent receipts dir")?;

    let client = Client::new();
    let url = format!("{}/api/v1/receipts", base_url.trim_end_matches('/'));

    let mut uploaded = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(receipts_root).context("failed to read receipts dir")? {
        let path = entry?.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read receipt: {}", path.display()))?;

        if let Err(e) = post_json_with_retries(&client, &url, &raw) {
            failures.push(format!("{}: {}", path.display(), e));
            continue;
        }

        // Mark as sent.
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("receipt.json");
        let dest = sent_dir.join(file_name);
        std::fs::rename(&path, &dest).or_else(|_| {
            std::fs::copy(&path, &dest)
                .context("failed to copy receipt")
                .and_then(|_| {
                    std::fs::remove_file(&path).context("failed to delete original receipt")
                })
        })?;
        uploaded += 1;
    }

    Ok(CloudItemPushResult { uploaded, failures })
}

pub fn push_uploads_to_cloud(project_root: &Path, base_url: &str) -> Result<CloudItemPushResult> {
    let uploads_root = sync_tools::uploads_root_dir(project_root);
    let queued_dir = sync_tools::uploads_queued_dir(&uploads_root);
    let sent_dir = sync_tools::uploads_sent_dir(&uploads_root);
    let blobs_dir = sync_tools::uploads_blobs_dir(&uploads_root);

    std::fs::create_dir_all(&queued_dir).context("failed to create queued uploads dir")?;
    std::fs::create_dir_all(&sent_dir).context("failed to create sent uploads dir")?;
    std::fs::create_dir_all(&blobs_dir).context("failed to create upload blobs dir")?;

    let client = Client::new();
    let url = format!("{}/api/v1/uploads", base_url.trim_end_matches('/'));

    let mut uploaded = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&queued_dir).context("failed to read queued uploads dir")? {
        let meta_path = entry?.path();
        if meta_path.is_dir() {
            continue;
        }
        if meta_path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&meta_path)
            .with_context(|| format!("failed to read upload metadata: {}", meta_path.display()))?;
        let parsed: sync_tools::UploadQueueEntry =
            serde_json::from_str(&raw).with_context(|| {
                format!(
                    "failed to parse upload metadata JSON: {}",
                    meta_path.display()
                )
            })?;

        let blob_hex = parsed
            .blob_sha256
            .strip_prefix("sha256:")
            .unwrap_or(&parsed.blob_sha256);
        let blob_path = blobs_dir.join(format!("{}.bin", blob_hex));
        let blob_bytes = std::fs::read(&blob_path)
            .with_context(|| format!("failed to read upload blob: {}", blob_path.display()))?;

        let file_name = PathBuf::from(&parsed.original_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();

        let raw_clone = raw.clone();
        let file_name_clone = file_name.clone();
        let blob_bytes_clone = blob_bytes.clone();
        let build_form = move || {
            let meta_part = multipart::Part::text(raw_clone.clone())
                .mime_str("application/json")
                .context("failed to create metadata part")?;
            let blob_part = multipart::Part::bytes(blob_bytes_clone.clone())
                .file_name(file_name_clone.clone())
                .mime_str("application/octet-stream")
                .context("failed to create blob part")?;
            Ok(multipart::Form::new()
                .part("metadata", meta_part)
                .part("blob", blob_part))
        };

        if let Err(e) = post_multipart_with_retries(&client, &url, build_form) {
            failures.push(format!("{}: {}", meta_path.display(), e));
            continue;
        }

        // Mark metadata as sent.
        let file_name = meta_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("upload.json");
        let dest = sent_dir.join(file_name);
        std::fs::rename(&meta_path, &dest).or_else(|_| {
            std::fs::copy(&meta_path, &dest)
                .context("failed to copy upload metadata")
                .and_then(|_| {
                    std::fs::remove_file(&meta_path)
                        .context("failed to delete original upload metadata")
                })
        })?;
        uploaded += 1;
    }

    Ok(CloudItemPushResult { uploaded, failures })
}

pub fn push_all_to_cloud(project_root: &Path) -> Result<CloudPushReport> {
    let Some(base_url) = cloud_base_url_from_env() else {
        return Ok(CloudPushReport {
            receipts_uploaded: 0,
            uploads_uploaded: 0,
            receipt_failures: Vec::new(),
            upload_failures: Vec::new(),
        });
    };

    let receipts = if let Some(receipts_root) = sync_tools::receipts_root_dir() {
        push_receipts_to_cloud(&receipts_root, &base_url)?
    } else {
        CloudItemPushResult {
            uploaded: 0,
            failures: Vec::new(),
        }
    };
    let uploads = push_uploads_to_cloud(project_root, &base_url)?;

    Ok(CloudPushReport {
        receipts_uploaded: receipts.uploaded,
        uploads_uploaded: uploads.uploaded,
        receipt_failures: receipts.failures,
        upload_failures: uploads.failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn start_ok_server(expected_requests: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0u8; 16 * 1024];
                let _ = stream.read(&mut buf);
                let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
                let _ = stream.write_all(resp.as_bytes());
            }
        });

        format!("http://{}", addr)
    }

    fn start_pull_server(
        receipts_json: String,
        uploads_json: String,
        blob_hex: String,
        blob_bytes: Vec<u8>,
        expected_requests: usize,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0u8; 32 * 1024];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");
                let path = first_line.split_whitespace().nth(1).unwrap_or("/");

                let (status, body, content_type) = if path == "/api/v1/receipts" {
                    ("200 OK", receipts_json.clone(), "application/json")
                } else if path == "/api/v1/uploads" {
                    ("200 OK", uploads_json.clone(), "application/json")
                } else if path == format!("/api/v1/uploads/blob/{}", blob_hex) {
                    // Raw bytes response
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\n\r\n",
                        blob_bytes.len()
                    );
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(&blob_bytes);
                    continue;
                } else {
                    ("404 Not Found", "not found".to_string(), "text/plain")
                };

                let resp = format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
                    status,
                    body.as_bytes().len(),
                    content_type,
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });

        format!("http://{}", addr)
    }

    #[test]
    fn cloud_push_receipts_moves_to_sent_on_200() {
        let base = start_ok_server(1);
        let root =
            std::env::temp_dir().join(format!("vidra_cloud_receipts_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("rr_test.json"), "{\"a\":1}").unwrap();
        let uploaded = push_receipts_to_cloud(&root, &base).unwrap();
        assert_eq!(uploaded.uploaded, 1);
        assert!(uploaded.failures.is_empty());
        assert!(root.join("sent").join("rr_test.json").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cloud_push_uploads_moves_metadata_to_sent_on_200() {
        let base = start_ok_server(1);

        let project_root =
            std::env::temp_dir().join(format!("vidra_cloud_uploads_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(&project_root).unwrap();

        let file = project_root.join("logo.png");
        std::fs::write(&file, b"pngbytes").unwrap();
        let enq = sync_tools::enqueue_upload_path(&project_root, &file).unwrap();
        assert_eq!(enq, 1);

        let uploaded = push_uploads_to_cloud(&project_root, &base).unwrap();
        assert_eq!(uploaded.uploaded, 1);
        assert!(uploaded.failures.is_empty());

        let uploads_root = sync_tools::uploads_root_dir(&project_root);
        let queued_dir = sync_tools::uploads_queued_dir(&uploads_root);
        let sent_dir = sync_tools::uploads_sent_dir(&uploads_root);
        assert!(sent_dir.exists());

        let queued_left = std::fs::read_dir(&queued_dir)
            .map(|it| it.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        assert_eq!(queued_left, 0);

        let sent_count = std::fs::read_dir(&sent_dir)
            .map(|it| it.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        assert!(sent_count >= 1);

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn cloud_pull_receipts_and_uploads_writes_local_sent_and_blobs() {
        let blob_hex = "abcd".to_string();
        let blob_bytes = b"hello".to_vec();

        let receipts_json = serde_json::json!([
            {
                "receipt_id": "rr_test",
                "project_id": "proj",
                "ir_hash": "sha256:ir",
                "output_hash": "sha256:out",
                "output_format": "mp4",
                "render_duration_ms": 1,
                "frame_count": 1,
                "hardware": {"os":"x","arch":"y"},
                "vlt_id": "vlt",
                "timestamp": "2026-01-01T00:00:00Z",
                "signature": "ed25519:AA=="
            }
        ])
        .to_string();

        let uploads_json = serde_json::to_string(&vec![sync_tools::UploadQueueEntry {
            original_path: "logo.png".to_string(),
            blob_sha256: format!("sha256:{}", blob_hex),
            size_bytes: blob_bytes.len() as u64,
            added_at: chrono::Utc::now(),
        }])
        .unwrap();

        let base = start_pull_server(
            receipts_json,
            uploads_json,
            blob_hex.clone(),
            blob_bytes.clone(),
            3,
        );

        // Pull receipts into a temp receipts root
        let receipts_root =
            std::env::temp_dir().join(format!("vidra_pull_receipts_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&receipts_root);
        std::fs::create_dir_all(&receipts_root).unwrap();
        let r = pull_receipts_from_cloud(&receipts_root, &base).unwrap();
        assert_eq!(r.uploaded, 1);
        assert!(receipts_root.join("sent").join("rr_test.json").exists());

        // Pull uploads into a temp project
        let project_root =
            std::env::temp_dir().join(format!("vidra_pull_uploads_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(&project_root).unwrap();
        let u = pull_uploads_from_cloud(&project_root, &base).unwrap();
        assert_eq!(u.uploaded, 1);
        let uploads_root = sync_tools::uploads_root_dir(&project_root);
        assert!(sync_tools::uploads_blobs_dir(&uploads_root)
            .join(format!("{}.bin", blob_hex))
            .exists());
        assert!(sync_tools::uploads_sent_dir(&uploads_root)
            .join(format!("upload_{}.json", blob_hex))
            .exists());

        let _ = std::fs::remove_dir_all(&receipts_root);
        let _ = std::fs::remove_dir_all(&project_root);
    }
}
