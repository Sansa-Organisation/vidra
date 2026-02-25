use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

const MAX_RETRIES: usize = 3;
const BACKOFF_MS: u64 = 50;

fn sleep_backoff(attempt: usize) {
    let ms = BACKOFF_MS.saturating_mul((attempt + 1) as u64);
    std::thread::sleep(Duration::from_millis(ms));
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

pub fn fetch_jobs_from_cloud(base_url: &str) -> Result<Vec<crate::jobs_tools::JobSpec>> {
    let client = Client::new();
    let url = format!("{}/api/v1/jobs", base_url.trim_end_matches('/'));
    let body = get_text_with_retries(&client, &url)?;

    // Accept either:
    // - [{...job...}, ...]
    // - {"jobs": [{...job...}, ...]}
    let value: serde_json::Value = serde_json::from_str(&body)
        .with_context(|| format!("invalid jobs JSON from {}", url))?;

    let jobs_value = if let Some(arr) = value.as_array() {
        serde_json::Value::Array(arr.clone())
    } else if let Some(arr) = value.get("jobs").and_then(|v| v.as_array()) {
        serde_json::Value::Array(arr.clone())
    } else {
        anyhow::bail!("expected jobs payload to be array or object with `jobs` array");
    };

    let jobs: Vec<crate::jobs_tools::JobSpec> = serde_json::from_value(jobs_value)
        .context("failed to decode jobs payload")?;
    Ok(jobs)
}

fn existing_job_ids(jobs_root: &Path) -> Result<HashSet<String>> {
    crate::jobs_tools::ensure_jobs_dirs(jobs_root)?;
    let mut ids = HashSet::new();
    for dir in [
        crate::jobs_tools::jobs_queued_dir(jobs_root),
        crate::jobs_tools::jobs_running_dir(jobs_root),
        crate::jobs_tools::jobs_sent_dir(jobs_root),
        crate::jobs_tools::jobs_failed_dir(jobs_root),
    ] {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).with_context(|| format!("failed to read jobs dir: {}", dir.display()))? {
            let path = entry?.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read job file: {}", path.display()))?;
            if let Ok(spec) = serde_json::from_str::<crate::jobs_tools::JobSpec>(&raw) {
                ids.insert(spec.job_id);
            }
        }
    }
    Ok(ids)
}

pub fn enqueue_cloud_jobs(jobs_root: &Path, mut jobs: Vec<crate::jobs_tools::JobSpec>, limit: Option<usize>) -> Result<usize> {
    crate::jobs_tools::ensure_jobs_dirs(jobs_root)?;
    jobs.sort_by_key(|j| j.created_at);

    let existing = existing_job_ids(jobs_root)?;
    let mut added = 0usize;
    for job in jobs {
        if existing.contains(&job.job_id) {
            continue;
        }
        if let Some(max) = limit {
            if added >= max {
                break;
            }
        }
        crate::jobs_tools::write_job_to_dir(&crate::jobs_tools::jobs_queued_dir(jobs_root), &job)?;
        added += 1;
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn start_jobs_server(body: String, expected_requests: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0u8; 16 * 1024];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
                    body.as_bytes().len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });

        format!("http://{}", addr)
    }

    #[test]
    fn fetch_and_enqueue_jobs() {
        let jobs_json = serde_json::json!([
            {
                "job_id": "job_a",
                "project_root": "/tmp/proj_a",
                "vidra_file": "main.vidra",
                "output": null,
                "format": "mp4",
                "targets": null,
                "data": null,
                "created_at": "2026-01-01T00:00:00Z"
            },
            {
                "job_id": "job_b",
                "project_root": "/tmp/proj_b",
                "vidra_file": "main.vidra",
                "output": null,
                "format": "mp4",
                "targets": null,
                "data": null,
                "created_at": "2026-01-01T00:01:00Z"
            }
        ])
        .to_string();

        let base = start_jobs_server(jobs_json, 1);
        let jobs = fetch_jobs_from_cloud(&base).unwrap();
        assert_eq!(jobs.len(), 2);

        let root = std::env::temp_dir().join(format!("vidra_jobs_cloud_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let added = enqueue_cloud_jobs(&root, jobs, Some(1)).unwrap();
        assert_eq!(added, 1);

        let queued = crate::jobs_tools::list_queued_jobs(&root).unwrap();
        assert_eq!(queued.len(), 1);

        let _ = std::fs::remove_dir_all(&root);
    }
}
