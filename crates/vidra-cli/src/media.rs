use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};

use vidra_core::VidraConfig;
use vidra_encode::FfmpegEncoder;
use vidra_ir::asset::{Asset, AssetId, AssetRegistry, AssetType};
use vidra_ir::layer::{Layer, LayerContent};
use vidra_ir::Project;

pub struct MediaPrepareReport {
    pub waveforms_materialized: usize,
}

pub fn prepare_project_media(
    project: &mut Project,
    config: &VidraConfig,
) -> Result<MediaPrepareReport> {
    let cache_root = resolve_cache_root(config)?;

    let mut report = MediaPrepareReport {
        waveforms_materialized: 0,
    };

    let Project { assets, scenes, .. } = project;
    for scene in scenes {
        for layer in &mut scene.layers {
            materialize_layer_media(layer, assets, &cache_root, &mut report)?;
        }
    }

    Ok(report)
}

fn materialize_layer_media(
    layer: &mut Layer,
    assets: &mut AssetRegistry,
    cache_root: &Path,
    report: &mut MediaPrepareReport,
) -> Result<()> {
    if let LayerContent::Waveform {
        asset_id,
        width,
        height,
        color,
    } = &layer.content
    {
        if !FfmpegEncoder::is_available() {
            // Keep placeholder rendering; don't fail the entire render.
            tracing::warn!("ffmpeg not available; skipping waveform materialization");
        } else {
            let audio_path = resolve_asset_path(assets, asset_id).ok_or_else(|| {
                anyhow!(
                    "waveform: failed to resolve audio path for asset_id '{}'",
                    asset_id
                )
            })?;
            let image_asset_id =
                materialize_waveform_png(assets, cache_root, &audio_path, *width, *height, *color)?;

            layer.content = LayerContent::Image {
                asset_id: image_asset_id,
            };
            report.waveforms_materialized += 1;
        }
    }

    for child in &mut layer.children {
        materialize_layer_media(child, assets, cache_root, report)?;
    }

    Ok(())
}

fn resolve_asset_path(assets: &AssetRegistry, asset_id: &AssetId) -> Option<PathBuf> {
    assets
        .get(asset_id)
        .map(|a| a.path.clone())
        .or_else(|| Some(PathBuf::from(asset_id.0.clone())))
}

fn materialize_waveform_png(
    assets: &mut AssetRegistry,
    cache_root: &Path,
    audio_path: &Path,
    width: u32,
    height: u32,
    color: vidra_core::Color,
) -> Result<AssetId> {
    let audio_bytes = std::fs::read(audio_path)
        .with_context(|| format!("failed to read waveform audio: {}", audio_path.display()))?;

    let audio_hash = sha256_hex_bytes(&audio_bytes);
    let rgba = color.to_rgba8();
    let color_hex = format!("{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2]);

    let cache_key = sha256_hex(&format!(
        "waveform|audio_sha256={}|{}x{}|color={}",
        audio_hash, width, height, color_hex
    ));

    let out_dir = cache_root.join("media").join("waveforms");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create waveform cache dir: {}", out_dir.display()))?;

    let out_path = out_dir.join(format!("{}.png", cache_key));

    let asset_id = AssetId::new(format!("media:waveform:{}", cache_key));
    if assets.get(&asset_id).is_none() {
        assets.register(Asset::new(
            asset_id.clone(),
            AssetType::Image,
            out_path.clone(),
        ));
    }

    if out_path.exists() {
        return Ok(asset_id);
    }

    // Generate waveform image via ffmpeg.
    // Note: showwavespic supports many audio formats and yields a static waveform.
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(audio_path)
        .arg("-filter_complex")
        .arg(format!(
            "showwavespic=s={}x{}:colors=0x{}",
            width, height, color_hex
        ))
        .args(["-frames:v", "1"])
        .arg(&out_path)
        .status()
        .with_context(|| "failed to run ffmpeg for waveform")?;

    if !status.success() {
        return Err(anyhow!(
            "ffmpeg waveform generation failed for {}",
            audio_path.display()
        ));
    }

    Ok(asset_id)
}

fn resolve_cache_root(config: &VidraConfig) -> Result<PathBuf> {
    let raw = &config.resources.cache_dir;
    expand_tilde(raw)
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

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_helpers_stable() {
        let a = sha256_hex("x");
        let b = sha256_hex("x");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);

        let c = sha256_hex_bytes(b"x");
        let d = sha256_hex_bytes(b"x");
        assert_eq!(c, d);
    }
}
