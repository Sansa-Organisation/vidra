use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseStatus {
    Known(String),
    Unknown,
    MissingAsset,
}

#[derive(Debug, Clone)]
pub struct LicensedAsset {
    pub path: String,
    pub asset_type: String,
    #[allow(dead_code)]
    pub exists: bool,
    pub status: LicenseStatus,
}

fn candidate_license_sidecars(asset_path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let file_name = asset_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("asset");
    let stem = asset_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);

    let parent = asset_path.parent().unwrap_or(Path::new("."));

    // Common patterns:
    // - assets/logo.png.license.txt
    // - assets/logo.license.txt
    // - assets/logo.license
    // - assets/LICENSE (shared)
    out.push(parent.join(format!("{}.license.txt", file_name)));
    out.push(parent.join(format!("{}.license", file_name)));
    out.push(parent.join(format!("{}.license.txt", stem)));
    out.push(parent.join(format!("{}.license", stem)));
    out.push(parent.join("LICENSE"));
    out.push(parent.join("LICENSE.txt"));
    out
}

fn read_license_string(asset_path: &Path) -> Option<String> {
    for cand in candidate_license_sidecars(asset_path) {
        if !cand.exists() {
            continue;
        }
        if let Ok(raw) = std::fs::read_to_string(&cand) {
            let s = raw.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

pub fn licenses_from_manifest_path(manifest_path: &Path) -> Result<Vec<LicensedAsset>> {
    let manifest = crate::sync_tools::read_asset_manifest(manifest_path)
        .with_context(|| format!("failed to read asset manifest: {}", manifest_path.display()))?;

    let mut out = Vec::new();
    for a in manifest.assets {
        let asset_path = PathBuf::from(&a.path);
        let exists = a.exists && asset_path.exists();

        let status = if !exists {
            LicenseStatus::MissingAsset
        } else if let Some(lic) = read_license_string(&asset_path) {
            LicenseStatus::Known(lic)
        } else {
            LicenseStatus::Unknown
        };

        out.push(LicensedAsset {
            path: a.path,
            asset_type: a.asset_type,
            exists,
            status,
        });
    }

    out.sort_by(|x, y| x.path.cmp(&y.path));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn licenses_detect_sidecar_license_txt() {
        let root = std::env::temp_dir().join(format!("vidra_licenses_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::create_dir_all(root.join(".vidra")).unwrap();

        let asset = root.join("assets").join("logo.png");
        std::fs::write(&asset, b"png").unwrap();
        std::fs::write(root.join("assets").join("logo.license.txt"), "MIT").unwrap();

        let manifest = crate::sync_tools::AssetManifest {
            project_id: "proj".to_string(),
            generated_at: chrono::Utc::now(),
            assets: vec![crate::sync_tools::AssetManifestEntry {
                id: "a".to_string(),
                asset_type: "image".to_string(),
                path: asset.to_string_lossy().to_string(),
                exists: true,
                size_bytes: None,
                sha256: None,
                is_remote_url: false,
            }],
        };
        let manifest_path = root.join(".vidra").join("asset_manifest.json");
        crate::sync_tools::write_asset_manifest(&manifest_path, &manifest).unwrap();

        let res = licenses_from_manifest_path(&manifest_path).unwrap();
        assert_eq!(res.len(), 1);
        assert!(matches!(res[0].status, LicenseStatus::Known(ref s) if s == "MIT"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
