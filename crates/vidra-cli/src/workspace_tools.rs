use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workspace {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceState {
    pub active: Option<String>,
    pub workspaces: Vec<Workspace>,
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

fn vidra_root_dir() -> Result<PathBuf> {
    Ok(resolve_home_dir()?.join(".vidra"))
}

pub fn workspaces_path() -> Result<PathBuf> {
    Ok(vidra_root_dir()?.join("workspaces.json"))
}

pub fn invites_log_path() -> Result<PathBuf> {
    Ok(vidra_root_dir()?.join("workspace_invites.log"))
}

pub fn load_state() -> Result<WorkspaceState> {
    let path = workspaces_path()?;
    if !path.exists() {
        return Ok(WorkspaceState {
            active: None,
            workspaces: Vec::new(),
        });
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read workspaces state: {}", path.display()))?;
    let s = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse workspaces state: {}", path.display()))?;
    Ok(s)
}

pub fn save_state(state: &WorkspaceState) -> Result<()> {
    let root = vidra_root_dir()?;
    std::fs::create_dir_all(&root).context("failed to create ~/.vidra")?;
    let path = workspaces_path()?;
    let json =
        serde_json::to_string_pretty(state).context("failed to serialize workspaces state")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write workspaces state: {}", path.display()))?;
    Ok(())
}

pub fn create_workspace(name: &str) -> Result<WorkspaceState> {
    let mut state = load_state()?;
    if !state.workspaces.iter().any(|w| w.name == name) {
        state.workspaces.push(Workspace {
            name: name.to_string(),
            created_at: chrono::Utc::now(),
        });
        state.workspaces.sort_by(|a, b| a.name.cmp(&b.name));
    }
    state.active = Some(name.to_string());
    save_state(&state)?;
    Ok(state)
}

pub fn switch_workspace(name: &str) -> Result<WorkspaceState> {
    let mut state = load_state()?;
    if !state.workspaces.iter().any(|w| w.name == name) {
        anyhow::bail!("workspace '{}' not found", name);
    }
    state.active = Some(name.to_string());
    save_state(&state)?;
    Ok(state)
}

pub fn list_workspaces() -> Result<WorkspaceState> {
    load_state()
}

pub fn invite_to_active_workspace(email: &str) -> Result<()> {
    let state = load_state()?;
    let Some(active) = state.active else {
        anyhow::bail!("no active workspace (run `vidra workspace create <name>` or `vidra workspace switch <name>`) ");
    };

    let root = vidra_root_dir()?;
    std::fs::create_dir_all(&root).context("failed to create ~/.vidra")?;
    let log_path = invites_log_path()?;
    let line = format!(
        "{}\t{}\t{}\n",
        chrono::Utc::now().to_rfc3339(),
        active,
        email
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open invites log: {}", log_path.display()))?;
    f.write_all(line.as_bytes())
        .with_context(|| format!("failed to append invites log: {}", log_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_create_switch_invite_roundtrip() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_workspace_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        let s = create_workspace("Acme").unwrap();
        assert_eq!(s.active.as_deref(), Some("Acme"));
        assert_eq!(s.workspaces.len(), 1);

        create_workspace("Personal").unwrap();
        let s2 = list_workspaces().unwrap();
        assert_eq!(s2.workspaces.len(), 2);

        switch_workspace("Acme").unwrap();
        invite_to_active_workspace("test@example.com").unwrap();
        assert!(invites_log_path().unwrap().exists());

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
