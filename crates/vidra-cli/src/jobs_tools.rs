use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobSpec {
    pub job_id: String,
    pub project_root: PathBuf,
    pub vidra_file: PathBuf,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<PathBuf>,

    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    #[allow(dead_code)]
    Sent,
    #[allow(dead_code)]
    Failed,
}

#[derive(Debug, Clone)]
pub struct ClaimedJob {
    pub spec: JobSpec,
    #[allow(dead_code)]
    pub state: JobState,
    pub job_file: PathBuf,
}

pub fn jobs_root_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".vidra").join("jobs"))
}

pub fn jobs_queued_dir(jobs_root: &Path) -> PathBuf {
    jobs_root.join("queued")
}

pub fn jobs_running_dir(jobs_root: &Path) -> PathBuf {
    jobs_root.join("running")
}

pub fn jobs_sent_dir(jobs_root: &Path) -> PathBuf {
    jobs_root.join("sent")
}

pub fn jobs_failed_dir(jobs_root: &Path) -> PathBuf {
    jobs_root.join("failed")
}

pub fn ensure_jobs_dirs(jobs_root: &Path) -> Result<()> {
    std::fs::create_dir_all(jobs_queued_dir(jobs_root)).context("failed to create jobs/queued")?;
    std::fs::create_dir_all(jobs_running_dir(jobs_root))
        .context("failed to create jobs/running")?;
    std::fs::create_dir_all(jobs_sent_dir(jobs_root)).context("failed to create jobs/sent")?;
    std::fs::create_dir_all(jobs_failed_dir(jobs_root)).context("failed to create jobs/failed")?;
    Ok(())
}

pub fn write_job_to_dir(dir: &Path, spec: &JobSpec) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).context("failed to create jobs dir")?;
    let filename = format!("job_{}.json", spec.job_id);
    let path = dir.join(filename);
    let content = serde_json::to_string_pretty(spec).context("failed to serialize job spec")?;
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write job file: {}", path.display()))?;
    Ok(path)
}

pub fn list_jobs_in_dir(dir: &Path, state: JobState) -> Result<Vec<ClaimedJob>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read jobs dir: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read job file: {}", path.display()))?;
        let spec: JobSpec = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse job JSON: {}", path.display()))?;
        out.push(ClaimedJob {
            spec,
            state,
            job_file: path,
        });
    }

    out.sort_by_key(|j| j.spec.created_at);
    Ok(out)
}

pub fn list_queued_jobs(jobs_root: &Path) -> Result<Vec<ClaimedJob>> {
    ensure_jobs_dirs(jobs_root)?;
    list_jobs_in_dir(&jobs_queued_dir(jobs_root), JobState::Queued)
}

/// Claim the next queued job by atomically moving it into `running/`.
pub fn claim_next_job(jobs_root: &Path) -> Result<Option<ClaimedJob>> {
    ensure_jobs_dirs(jobs_root)?;
    let mut queued = list_jobs_in_dir(&jobs_queued_dir(jobs_root), JobState::Queued)?;
    let Some(next) = queued.drain(..).next() else {
        return Ok(None);
    };

    let file_name = next
        .job_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("job.json");
    let dest = jobs_running_dir(jobs_root).join(file_name);

    std::fs::rename(&next.job_file, &dest).or_else(|_| {
        std::fs::copy(&next.job_file, &dest)
            .context("failed to copy claimed job")
            .and_then(|_| {
                std::fs::remove_file(&next.job_file).context("failed to remove original job")
            })
    })?;

    Ok(Some(ClaimedJob {
        spec: next.spec,
        state: JobState::Running,
        job_file: dest,
    }))
}

pub fn mark_job_sent(jobs_root: &Path, claimed: &ClaimedJob) -> Result<PathBuf> {
    ensure_jobs_dirs(jobs_root)?;
    let file_name = claimed
        .job_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("job.json");
    let dest = jobs_sent_dir(jobs_root).join(file_name);
    std::fs::rename(&claimed.job_file, &dest).or_else(|_| {
        std::fs::copy(&claimed.job_file, &dest)
            .context("failed to copy sent job")
            .and_then(|_| {
                std::fs::remove_file(&claimed.job_file).context("failed to remove running job")
            })
    })?;
    Ok(dest)
}

pub fn mark_job_failed(
    jobs_root: &Path,
    claimed: &ClaimedJob,
    error_message: &str,
) -> Result<PathBuf> {
    ensure_jobs_dirs(jobs_root)?;
    let file_name = claimed
        .job_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("job.json");

    let dest = jobs_failed_dir(jobs_root).join(file_name);
    std::fs::rename(&claimed.job_file, &dest).or_else(|_| {
        std::fs::copy(&claimed.job_file, &dest)
            .context("failed to copy failed job")
            .and_then(|_| {
                std::fs::remove_file(&claimed.job_file).context("failed to remove running job")
            })
    })?;

    // Best-effort sidecar error message
    let sidecar = dest.with_extension("error.txt");
    let _ = std::fs::write(&sidecar, error_message);

    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_queue_list_and_claim_moves_to_running() {
        let root = std::env::temp_dir().join(format!("vidra_jobs_queue_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let jobs_root = root.join("jobs");
        ensure_jobs_dirs(&jobs_root).unwrap();

        let spec = JobSpec {
            job_id: "a".to_string(),
            project_root: PathBuf::from("/tmp/proj"),
            vidra_file: PathBuf::from("main.vidra"),
            output: None,
            format: None,
            targets: None,
            data: None,
            created_at: chrono::Utc::now(),
        };

        write_job_to_dir(&jobs_queued_dir(&jobs_root), &spec).unwrap();

        let queued = list_queued_jobs(&jobs_root).unwrap();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].spec.job_id, "a");
        assert_eq!(queued[0].state, JobState::Queued);

        let claimed = claim_next_job(&jobs_root).unwrap();
        assert!(claimed.is_some());
        let claimed = claimed.unwrap();
        assert_eq!(claimed.state, JobState::Running);
        assert!(claimed.job_file.starts_with(jobs_running_dir(&jobs_root)));

        let queued2 = list_queued_jobs(&jobs_root).unwrap();
        assert_eq!(queued2.len(), 0);

        let _ = std::fs::remove_dir_all(&root);
    }
}
