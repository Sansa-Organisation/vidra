use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrandKit {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn resolve_home_dir() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("VIDRA_HOME_DIR") {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return Ok(p);
        }
        return Ok(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(p));
    }
    dirs::home_dir().context("failed to resolve home dir")
}

pub fn brands_root_dir() -> Result<PathBuf> {
    Ok(resolve_home_dir()?.join(".vidra").join("brands"))
}

pub fn brand_kit_path(name: &str) -> Result<PathBuf> {
    Ok(brands_root_dir()?.join(format!("{}.json", name)))
}

pub fn create_brand_kit(name: &str) -> Result<PathBuf> {
    let root = brands_root_dir()?;
    std::fs::create_dir_all(&root).context("failed to create brands dir")?;

    let kit = BrandKit {
        name: name.to_string(),
        created_at: chrono::Utc::now(),
    };
    let path = brand_kit_path(name)?;
    let json = serde_json::to_string_pretty(&kit).context("failed to serialize brand kit")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write brand kit: {}", path.display()))?;
    Ok(path)
}

pub fn list_brand_kits() -> Result<Vec<BrandKit>> {
    let root = brands_root_dir()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut kits = Vec::new();
    for entry in std::fs::read_dir(&root).with_context(|| format!("failed to read brands dir: {}", root.display()))? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read brand kit: {}", path.display()))?;
        if let Ok(kit) = serde_json::from_str::<BrandKit>(&raw) {
            kits.push(kit);
        }
    }

    kits.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(kits)
}

pub fn brand_kit_exists(name: &str) -> Result<bool> {
    Ok(brand_kit_path(name)?.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list_brand_kits() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_brand_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        create_brand_kit("company-primary").unwrap();
        create_brand_kit("social-neon").unwrap();
        assert!(brand_kit_exists("company-primary").unwrap());

        let kits = list_brand_kits().unwrap();
        assert_eq!(kits.len(), 2);
        assert_eq!(kits[0].name, "company-primary");

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
