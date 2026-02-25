use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

use vidra_core::VidraConfig;
use vidra_ir::asset::{AssetRegistry, AssetType};
use vidra_ir::Project;

pub struct RemoteAssetsReport {
    pub downloaded: usize,
    pub reused_from_cache: usize,
}

pub fn prepare_project_remote_assets(
    project: &mut Project,
    config: &VidraConfig,
) -> Result<RemoteAssetsReport> {
    let cache_root = resolve_cache_root(config)?;

    let mut report = RemoteAssetsReport {
        downloaded: 0,
        reused_from_cache: 0,
    };

    let client = Client::new();
    prepare_registry_remote_assets(&client, &mut project.assets, &cache_root, &mut report)?;

    Ok(report)
}

fn prepare_registry_remote_assets(
    client: &Client,
    assets: &mut AssetRegistry,
    cache_root: &Path,
    report: &mut RemoteAssetsReport,
) -> Result<()> {
    for asset in assets.all_mut() {
        let path_str = asset.path.to_string_lossy();
        if !is_http_url(&path_str) {
            continue;
        }

        let url = path_str.to_string();
        let ext = infer_extension_from_url(&url)
            .or_else(|| default_extension_for_type(&asset.asset_type).map(|s| s.to_string()));

        let cache_key = sha256_hex(&format!(
            "asset_fetch|type={}|url={}",
            asset.asset_type, url
        ));

        let out_dir = cache_root
            .join("assets")
            .join(asset_type_dir(&asset.asset_type));
        std::fs::create_dir_all(&out_dir)
            .with_context(|| format!("failed to create asset cache dir: {}", out_dir.display()))?;

        let file_name = match ext {
            Some(e) if !e.is_empty() => format!("{}.{}", cache_key, e),
            _ => cache_key.clone(),
        };

        let out_path = out_dir.join(file_name);
        if out_path.exists() {
            asset.path = out_path;
            report.reused_from_cache += 1;
            continue;
        }

        let tmp_path = out_path.with_extension("tmp");

        let res = client
            .get(&url)
            .send()
            .with_context(|| format!("failed to download asset: {}", url))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().unwrap_or_default();
            return Err(anyhow!("remote asset fetch failed: {}: {}", status, body));
        }

        let bytes = res
            .bytes()
            .with_context(|| format!("failed to read bytes for asset: {}", url))?;

        std::fs::write(&tmp_path, &bytes)
            .with_context(|| format!("failed to write downloaded asset: {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &out_path).with_context(|| {
            format!(
                "failed to finalize downloaded asset (rename {} -> {})",
                tmp_path.display(),
                out_path.display()
            )
        })?;

        asset.path = out_path;
        report.downloaded += 1;
    }

    Ok(())
}

fn asset_type_dir(asset_type: &AssetType) -> &'static str {
    match asset_type {
        AssetType::Image => "images",
        AssetType::Video => "video",
        AssetType::Audio => "audio",
        AssetType::Font => "fonts",
        AssetType::Shader => "shaders",
        AssetType::Lut => "luts",
    }
}

fn default_extension_for_type(asset_type: &AssetType) -> Option<&'static str> {
    match asset_type {
        AssetType::Image => Some("png"),
        AssetType::Video => Some("mp4"),
        AssetType::Audio => Some("mp3"),
        AssetType::Font => Some("ttf"),
        AssetType::Shader => Some("wgsl"),
        AssetType::Lut => Some("cube"),
    }
}

fn infer_extension_from_url(url: &str) -> Option<String> {
    let no_frag = url.split('#').next().unwrap_or(url);
    let no_query = no_frag.split('?').next().unwrap_or(no_frag);

    let ext = Path::new(no_query)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.trim().trim_start_matches('.').to_lowercase())?;

    if ext.is_empty() {
        None
    } else {
        Some(ext)
    }
}

fn is_http_url(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://") || s.starts_with("https://")
}

fn resolve_cache_root(config: &VidraConfig) -> Result<PathBuf> {
    expand_tilde(&config.resources.cache_dir)
}

fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" || path.starts_with("~/") {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("failed to resolve home dir"))?;
        if path == "~" {
            return Ok(home);
        }
        return Ok(home.join(path.trim_start_matches("~/")));
    }
    Ok(PathBuf::from(path))
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_extension_from_url_works() {
        assert_eq!(
            infer_extension_from_url("https://x/y.png").as_deref(),
            Some("png")
        );
        assert_eq!(
            infer_extension_from_url("https://x/y.JPG?cache=1").as_deref(),
            Some("jpg")
        );
        assert_eq!(infer_extension_from_url("https://x/y").as_deref(), None);
    }

    #[test]
    fn sha256_hex_is_stable() {
        let a = sha256_hex("hello");
        let b = sha256_hex("hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }
}
