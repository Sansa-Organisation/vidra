use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
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

pub fn plugins_root_dir() -> Result<PathBuf> {
    Ok(resolve_home_dir()?.join(".vidra").join("plugins"))
}

pub fn plugin_dir(name: &str) -> Result<PathBuf> {
    Ok(plugins_root_dir()?.join(name))
}

pub fn plugin_manifest_path(name: &str) -> Result<PathBuf> {
    Ok(plugin_dir(name)?.join("plugin.json"))
}

pub fn install_plugin(name: &str, version: Option<&str>) -> Result<PathBuf> {
    let dir = plugin_dir(name)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create plugin dir: {}", dir.display()))?;

    let manifest = PluginManifest {
        name: name.to_string(),
        version: version.unwrap_or("0.1.0").to_string(),
        installed_at: chrono::Utc::now(),
    };
    let path = plugin_manifest_path(name)?;
    let json =
        serde_json::to_string_pretty(&manifest).context("failed to serialize plugin manifest")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write plugin manifest: {}", path.display()))?;
    Ok(path)
}

pub fn remove_plugin(name: &str) -> Result<()> {
    let dir = plugin_dir(name)?;
    if !dir.exists() {
        anyhow::bail!("plugin '{}' is not installed", name);
    }
    std::fs::remove_dir_all(&dir)
        .with_context(|| format!("failed to remove plugin dir: {}", dir.display()))?;
    Ok(())
}

pub fn read_plugin_manifest(name: &str) -> Result<PluginManifest> {
    let path = plugin_manifest_path(name)?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read plugin manifest: {}", path.display()))?;
    let m = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse plugin manifest: {}", path.display()))?;
    Ok(m)
}

pub fn list_plugins() -> Result<Vec<PluginManifest>> {
    let root = plugins_root_dir()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("failed to read plugins dir: {}", root.display()))?
    {
        let p = entry?.path();
        if !p.is_dir() {
            continue;
        }
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let manifest_path = p.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }
        let raw = std::fs::read_to_string(&manifest_path).with_context(|| {
            format!(
                "failed to read plugin manifest: {}",
                manifest_path.display()
            )
        })?;
        if let Ok(m) = serde_json::from_str::<PluginManifest>(&raw) {
            out.push(m);
        }
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_list_info_remove_plugin() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_plugins_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        install_plugin("vidra-color-grade", Some("1.1.0")).unwrap();
        let list = list_plugins().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "vidra-color-grade");

        let info = read_plugin_manifest("vidra-color-grade").unwrap();
        assert_eq!(info.version, "1.1.0");

        remove_plugin("vidra-color-grade").unwrap();
        let list2 = list_plugins().unwrap();
        assert_eq!(list2.len(), 0);

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
