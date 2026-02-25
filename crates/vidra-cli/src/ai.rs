use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use serde_json::json;
use sha2::{Digest, Sha256};

use vidra_core::types::Easing;
use vidra_core::types::LayerEffect;
use vidra_core::VidraConfig;
use vidra_ir::animation::{AnimatableProperty, Animation, Keyframe};
use vidra_ir::asset::AssetRegistry;
use vidra_ir::asset::{Asset, AssetId, AssetType};
use vidra_ir::layer::{Layer, LayerContent};
use vidra_ir::Project;

pub struct AiPrepareReport {
    pub tts_layers_materialized: usize,
    pub autocaption_layers_materialized: usize,
    pub bg_removals_materialized: usize,
}

pub fn prepare_project_ai(project: &mut Project, config: &VidraConfig) -> Result<AiPrepareReport> {
    if !config.ai.enabled {
        return Ok(AiPrepareReport {
            tts_layers_materialized: 0,
            autocaption_layers_materialized: 0,
            bg_removals_materialized: 0,
        });
    }

    let cache_root = resolve_cache_root(config)?;

    let mut report = AiPrepareReport {
        tts_layers_materialized: 0,
        autocaption_layers_materialized: 0,
        bg_removals_materialized: 0,
    };

    let Project { assets, scenes, .. } = project;
    for scene in scenes {
        for layer in &mut scene.layers {
            materialize_layer_ai(layer, assets, config, &cache_root, &mut report)?;
        }
    }

    Ok(report)
}

fn materialize_layer_ai(
    layer: &mut Layer,
    assets: &mut AssetRegistry,
    config: &VidraConfig,
    cache_root: &Path,
    report: &mut AiPrepareReport,
) -> Result<()> {
    // Background removal (image-only): materialize to a cached PNG-with-alpha and swap the asset.
    if layer
        .effects
        .iter()
        .any(|e| matches!(e, LayerEffect::RemoveBackground))
    {
        if let LayerContent::Image { asset_id } = &layer.content {
            let input_path = resolve_asset_path(assets, asset_id).ok_or_else(|| {
                anyhow!(
                    "removeBackground: failed to resolve image path for asset_id '{}'",
                    asset_id
                )
            })?;
            let new_asset_id = materialize_removebg_png(assets, config, cache_root, &input_path)?;

            // Swap the image content to the new alpha image.
            layer.content = LayerContent::Image {
                asset_id: new_asset_id,
            };

            // Drop the effect so render doesn't attempt anything (and so we don't re-materialize).
            layer
                .effects
                .retain(|e| !matches!(e, LayerEffect::RemoveBackground));
            report.bg_removals_materialized += 1;
        } else {
            // For now, only Image layers are supported.
            tracing::warn!("removeBackground is currently supported only on image() layers");
        }
    }

    match &mut layer.content {
        LayerContent::TTS {
            text,
            voice,
            audio_asset_id,
            ..
        } => {
            if audio_asset_id.is_none() {
                let (asset_id, _path) = if let Some(voice_id) = voice.strip_prefix("eleven:") {
                    materialize_elevenlabs_tts(assets, config, cache_root, text, voice_id)
                } else {
                    materialize_openai_tts(assets, config, cache_root, text, voice)
                }
                .with_context(|| format!("failed to materialize TTS for layer '{}'", layer.id))?;
                *audio_asset_id = Some(asset_id);
                report.tts_layers_materialized += 1;
            }
        }
        _ => {}
    }

    // AutoCaption expansion: replace placeholder node with generated timed text layers.
    // Clone the fields we need so we can mutate the layer without borrow conflicts.
    let autocaption_fields = match &layer.content {
        LayerContent::AutoCaption {
            asset_id,
            font_family,
            font_size,
            color,
        } => Some((asset_id.clone(), font_family.clone(), *font_size, *color)),
        _ => None,
    };

    if let Some((asset_id, font_family, font_size, color)) = autocaption_fields {
        // Expand exactly once: if it already has children, assume it was materialized.
        if layer.children.is_empty() {
            let audio_path = resolve_audio_path(assets, &asset_id);
            let audio_path = audio_path.ok_or_else(|| {
                anyhow!(
                    "AutoCaption references audio asset_id '{}' but no file path could be resolved",
                    asset_id
                )
            })?;

            let mut segments = transcribe_openai_segments(config, cache_root, &audio_path)?;
            if config.ai.gemini.caption_refine {
                segments = refine_caption_segments_gemini(config, cache_root, &segments)
                    .context("Gemini caption refinement failed")?;
            }
            apply_caption_segments(layer, &segments, &font_family, font_size, color);
            report.autocaption_layers_materialized += 1;
        }
    }

    for child in &mut layer.children {
        materialize_layer_ai(child, assets, config, cache_root, report)?;
    }

    Ok(())
}

fn resolve_audio_path(assets: &AssetRegistry, asset_id: &AssetId) -> Option<PathBuf> {
    if let Some(asset) = assets.get(asset_id) {
        return Some(asset.path.clone());
    }
    // Fallback: some older IR encodes the path directly as AssetId.
    Some(PathBuf::from(asset_id.0.clone()))
}

fn resolve_asset_path(assets: &AssetRegistry, asset_id: &AssetId) -> Option<PathBuf> {
    assets
        .get(asset_id)
        .map(|a| a.path.clone())
        .or_else(|| Some(PathBuf::from(asset_id.0.clone())))
}

fn materialize_removebg_png(
    assets: &mut AssetRegistry,
    config: &VidraConfig,
    cache_root: &Path,
    image_path: &Path,
) -> Result<AssetId> {
    let api_key = std::env::var(&config.ai.removebg.api_key_env).map_err(|_| {
        anyhow!(
            "AI is enabled but {} is not set (needed for remove.bg background removal)",
            config.ai.removebg.api_key_env
        )
    })?;

    let bytes = std::fs::read(image_path).with_context(|| {
        format!(
            "failed to read image for background removal: {}",
            image_path.display()
        )
    })?;
    let img_hash = sha256_hex_bytes(&bytes);

    let cache_key = sha256_hex(&format!(
        "removebg|base_url={}|img_sha256={}",
        config.ai.removebg.base_url, img_hash
    ));

    let out_dir = cache_root.join("ai").join("bg_remove").join("removebg");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create AI cache dir: {}", out_dir.display()))?;
    let out_path = out_dir.join(format!("{}.png", cache_key));

    let asset_id = AssetId::new(format!("ai:bg:removebg:{}", cache_key));

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

    let file_name = image_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image.png");

    let part = multipart::Part::bytes(bytes)
        .file_name(file_name.to_string())
        .mime_str("application/octet-stream")
        .context("failed to build multipart part")?;

    // remove.bg API expects multipart form fields.
    let form = multipart::Form::new()
        .part("image_file", part)
        .text("size", "auto")
        .text("format", "png");

    let client = Client::new();
    let base = config.ai.removebg.base_url.trim_end_matches('/');
    let url = format!("{}/v1.0/removebg", base);

    let res = client
        .post(url)
        .header("X-Api-Key", api_key)
        .multipart(form)
        .send()
        .context("remove.bg request failed")?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().unwrap_or_default();
        return Err(anyhow!("remove.bg failed: {}: {}", status, text));
    }

    let out_bytes = res.bytes().context("failed to read remove.bg bytes")?;
    std::fs::write(&out_path, &out_bytes)
        .with_context(|| format!("failed to write remove.bg output: {}", out_path.display()))?;

    Ok(asset_id)
}

#[derive(Debug, Clone)]
struct CaptionSegment {
    start_s: f64,
    end_s: f64,
    text: String,
}

fn transcribe_openai_segments(
    config: &VidraConfig,
    cache_root: &Path,
    audio_path: &Path,
) -> Result<Vec<CaptionSegment>> {
    let api_key = std::env::var(&config.ai.openai.api_key_env).map_err(|_| {
        anyhow!(
            "AI is enabled but {} is not set (needed for OpenAI-compatible transcription)",
            config.ai.openai.api_key_env
        )
    })?;

    let bytes = std::fs::read(audio_path)
        .with_context(|| format!("failed to read autocaption audio: {}", audio_path.display()))?;
    let audio_hash = sha256_hex_bytes(&bytes);

    let cache_key = sha256_hex(&format!(
        "openai_transcribe|base_url={}|model={}|audio_sha256={}",
        config.ai.openai.base_url, config.ai.openai.transcribe_model, audio_hash
    ));

    let out_dir = cache_root.join("ai").join("captions").join("openai");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create AI cache dir: {}", out_dir.display()))?;
    let out_path = out_dir.join(format!("{}.verbose.json", cache_key));

    if !out_path.exists() {
        let file_name = audio_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");

        let part = multipart::Part::bytes(bytes)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")
            .context("failed to build multipart part")?;

        let form = multipart::Form::new()
            .part("file", part)
            .text("model", config.ai.openai.transcribe_model.clone())
            .text("response_format", "verbose_json")
            // Ask for segment timestamps (widely supported by OpenAI-compatible servers).
            .text("timestamp_granularities[]", "segment");

        let client = Client::new();
        let base = config.ai.openai.base_url.trim_end_matches('/');
        let url = format!("{}/v1/audio/transcriptions", base);

        let res = client
            .post(url)
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .context("OpenAI transcription request failed")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().unwrap_or_default();
            return Err(anyhow!("OpenAI transcription failed: {}: {}", status, text));
        }

        let text = res.text().context("failed to read transcription body")?;
        std::fs::write(&out_path, text).with_context(|| {
            format!(
                "failed to write transcription cache: {}",
                out_path.display()
            )
        })?;
    }

    let raw = std::fs::read_to_string(&out_path)
        .with_context(|| format!("failed to read transcription cache: {}", out_path.display()))?;
    parse_verbose_json_segments(&raw)
}

fn parse_verbose_json_segments(raw: &str) -> Result<Vec<CaptionSegment>> {
    let v: serde_json::Value = serde_json::from_str(raw).context("invalid verbose_json")?;
    let segments = v
        .get("segments")
        .and_then(|s| s.as_array())
        .ok_or_else(|| anyhow!("verbose_json missing 'segments' array"))?;

    let mut out = Vec::new();
    for seg in segments {
        let start_s = seg.get("start").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let end_s = seg.get("end").and_then(|x| x.as_f64()).unwrap_or(start_s);
        let text = seg
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        out.push(CaptionSegment {
            start_s,
            end_s,
            text,
        });
    }
    Ok(out)
}

fn refine_caption_segments_gemini(
    config: &VidraConfig,
    cache_root: &Path,
    segments: &[CaptionSegment],
) -> Result<Vec<CaptionSegment>> {
    let api_key = std::env::var(&config.ai.gemini.api_key_env).map_err(|_| {
        anyhow!(
            "Gemini caption refinement is enabled but {} is not set",
            config.ai.gemini.api_key_env
        )
    })?;

    let segments_json: Vec<serde_json::Value> = segments
        .iter()
        .map(|s| {
            json!({
                "start_s": s.start_s,
                "end_s": s.end_s,
                "text": s.text,
            })
        })
        .collect();
    let segments_json_str =
        serde_json::to_string(&segments_json).context("failed to serialize segments")?;
    let segments_hash = sha256_hex(&segments_json_str);

    let cache_key = sha256_hex(&format!(
        "gemini_caption_refine|base_url={}|model={}|segments_sha256={}",
        config.ai.gemini.base_url, config.ai.gemini.model, segments_hash
    ));

    let out_dir = cache_root.join("ai").join("captions").join("gemini_refine");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create AI cache dir: {}", out_dir.display()))?;
    let out_path = out_dir.join(format!("{}.json", cache_key));

    if !out_path.exists() {
        let prompt = format!(
            concat!(
                "You are a caption editor.\n\n",
                "Given a JSON array of caption segments with fields start_s, end_s, text, return JSON ONLY as: {{\"texts\": [ ... ]}}.\n",
                "Rules:\n",
                "- Output array length MUST match input length exactly.\n",
                "- Only improve punctuation, casing, and minor whitespace.\n",
                "- Do NOT change meaning, do NOT add/remove words unless fixing obvious typos.\n",
                "- Do NOT add commentary, markdown, or extra fields.\n\n",
                "Input segments JSON:\n{}"
            ),
            segments_json_str
        );

        let body = json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": prompt}]
                }
            ],
            "generationConfig": {
                "temperature": 0.0,
                "response_mime_type": "application/json"
            }
        });

        let client = Client::new();
        let base = config.ai.gemini.base_url.trim_end_matches('/');
        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            base, config.ai.gemini.model
        );

        let res = client
            .post(url)
            .query(&[("key", api_key)])
            .json(&body)
            .send()
            .context("Gemini generateContent request failed")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().unwrap_or_default();
            return Err(anyhow!(
                "Gemini generateContent failed: {}: {}",
                status,
                text
            ));
        }

        let raw: serde_json::Value = res.json().context("failed to parse Gemini response JSON")?;
        let text = raw
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .and_then(|p| p.first())
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("unexpected Gemini response shape (missing candidates[0].content.parts[0].text)"))?;

        // Gemini typically returns JSON-as-text for structured outputs.
        std::fs::write(&out_path, text).with_context(|| {
            format!(
                "failed to write Gemini caption refine cache: {}",
                out_path.display()
            )
        })?;
    }

    let refined_raw = std::fs::read_to_string(&out_path).with_context(|| {
        format!(
            "failed to read Gemini caption refine cache: {}",
            out_path.display()
        )
    })?;
    let refined_json: serde_json::Value = serde_json::from_str(&refined_raw)
        .context("Gemini caption refine output was not valid JSON")?;
    let texts = refined_json
        .get("texts")
        .and_then(|t| t.as_array())
        .ok_or_else(|| anyhow!("Gemini caption refine output missing 'texts' array"))?;

    if texts.len() != segments.len() {
        return Err(anyhow!(
            "Gemini caption refine output length mismatch: expected {}, got {}",
            segments.len(),
            texts.len()
        ));
    }

    let mut out = Vec::with_capacity(segments.len());
    for (seg, text_val) in segments.iter().zip(texts.iter()) {
        let Some(text) = text_val.as_str() else {
            return Err(anyhow!(
                "Gemini caption refine 'texts' must be an array of strings"
            ));
        };
        out.push(CaptionSegment {
            start_s: seg.start_s,
            end_s: seg.end_s,
            text: text.trim().to_string(),
        });
    }
    Ok(out)
}

fn apply_caption_segments(
    layer: &mut Layer,
    segments: &[CaptionSegment],
    font_family: &str,
    font_size: f64,
    color: vidra_core::Color,
) {
    let base_pos = layer.transform.position;
    let base_anchor = layer.transform.anchor;
    let base_scale = layer.transform.scale;

    // Convert this node into a grouping layer.
    layer.content = LayerContent::Empty;

    // Put captions at the same position as the original layer by default.
    for (i, seg) in segments.iter().enumerate() {
        let duration_s = (seg.end_s - seg.start_s).max(0.0);
        if duration_s <= 0.0 {
            continue;
        }

        let mut child = Layer::new(
            vidra_ir::layer::LayerId::new(format!("caption_{}", i)),
            LayerContent::Text {
                text: seg.text.clone(),
                font_family: font_family.to_string(),
                font_size,
                color,
            },
        );

        child.transform.position = base_pos;
        child.transform.anchor = base_anchor;
        child.transform.scale = base_scale;
        child.transform.opacity = 0.0;

        // Opacity envelope: 0 → 1 quickly, hold, then 0.
        let fade = 0.06_f64.min((duration_s / 2.0).max(0.0));
        let mut anim = Animation::new(AnimatableProperty::Opacity)
            .with_delay(vidra_core::Duration::from_seconds(seg.start_s));
        anim.add_keyframe(Keyframe::new(vidra_core::Duration::zero(), 0.0));
        anim.add_keyframe(
            Keyframe::new(vidra_core::Duration::from_seconds(fade), 1.0)
                .with_easing(Easing::EaseOut),
        );
        anim.add_keyframe(Keyframe::new(
            vidra_core::Duration::from_seconds((duration_s - fade).max(fade)),
            1.0,
        ));
        anim.add_keyframe(
            Keyframe::new(vidra_core::Duration::from_seconds(duration_s), 0.0)
                .with_easing(Easing::EaseIn),
        );
        child.animations.push(anim);

        layer.children.push(child);
    }
}

fn materialize_openai_tts(
    assets: &mut AssetRegistry,
    config: &VidraConfig,
    cache_root: &Path,
    text: &str,
    voice: &str,
) -> Result<(AssetId, PathBuf)> {
    let api_key = std::env::var(&config.ai.openai.api_key_env).map_err(|_| {
        anyhow!(
            "AI is enabled but {} is not set (needed for OpenAI-compatible TTS)",
            config.ai.openai.api_key_env
        )
    })?;

    let format = config.ai.openai.tts_format.trim().to_lowercase();
    let ext = match format.as_str() {
        "mp3" => "mp3",
        "wav" => "wav",
        other => {
            return Err(anyhow!(
                "unsupported ai.openai.tts_format: {} (expected mp3|wav)",
                other
            ))
        }
    };

    let cache_key = sha256_hex(&format!(
        "openai_tts|base_url={}|model={}|voice={}|format={}|text={}",
        config.ai.openai.base_url, config.ai.openai.tts_model, voice, ext, text
    ));

    let out_dir = cache_root.join("ai").join("tts").join("openai");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create AI cache dir: {}", out_dir.display()))?;

    let out_path = out_dir.join(format!("{}.{}", cache_key, ext));

    let asset_id = AssetId::new(format!("ai:tts:openai:{}", cache_key));

    // Register asset (even if file already exists) so downstream encode can find it.
    if assets.get(&asset_id).is_none() {
        assets.register(Asset::new(
            asset_id.clone(),
            AssetType::Audio,
            out_path.clone(),
        ));
    }

    if out_path.exists() {
        return Ok((asset_id, out_path));
    }

    let client = Client::new();

    let base = config.ai.openai.base_url.trim_end_matches('/');
    let url = format!("{}/v1/audio/speech", base);

    let body = json!({
        "model": config.ai.openai.tts_model,
        "input": text,
        "voice": voice,
        "format": ext,
    });

    let res = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .context("OpenAI TTS request failed")?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().unwrap_or_default();
        return Err(anyhow!("OpenAI TTS failed: {}: {}", status, text));
    }

    let bytes = res
        .bytes()
        .context("failed to read OpenAI TTS response bytes")?;

    std::fs::write(&out_path, &bytes)
        .with_context(|| format!("failed to write TTS output: {}", out_path.display()))?;

    Ok((asset_id, out_path))
}

fn materialize_elevenlabs_tts(
    assets: &mut AssetRegistry,
    config: &VidraConfig,
    cache_root: &Path,
    text: &str,
    voice_id: &str,
) -> Result<(AssetId, PathBuf)> {
    let api_key = std::env::var(&config.ai.elevenlabs.api_key_env).map_err(|_| {
        anyhow!(
            "AI is enabled but {} is not set (needed for ElevenLabs TTS)",
            config.ai.elevenlabs.api_key_env
        )
    })?;

    let cache_key = sha256_hex(&format!(
        "elevenlabs_tts|base_url={}|voice_id={}|text={}",
        config.ai.elevenlabs.base_url, voice_id, text
    ));

    let out_dir = cache_root.join("ai").join("tts").join("elevenlabs");
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create AI cache dir: {}", out_dir.display()))?;

    let out_path = out_dir.join(format!("{}.mp3", cache_key));
    let asset_id = AssetId::new(format!("ai:tts:elevenlabs:{}", cache_key));

    if assets.get(&asset_id).is_none() {
        assets.register(Asset::new(
            asset_id.clone(),
            AssetType::Audio,
            out_path.clone(),
        ));
    }

    if out_path.exists() {
        return Ok((asset_id, out_path));
    }

    let client = Client::new();
    let base = config.ai.elevenlabs.base_url.trim_end_matches('/');
    let url = format!("{}/v1/text-to-speech/{}", base, voice_id);

    let body = json!({
        "text": text,
        // Keep defaults minimal; ElevenLabs will pick defaults if omitted.
        "output_format": "mp3_44100_128",
    });

    let res = client
        .post(url)
        .header("xi-api-key", api_key)
        .header("accept", "audio/mpeg")
        .json(&body)
        .send()
        .context("ElevenLabs TTS request failed")?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().unwrap_or_default();
        return Err(anyhow!("ElevenLabs TTS failed: {}: {}", status, text));
    }

    let bytes = res
        .bytes()
        .context("failed to read ElevenLabs TTS response bytes")?;

    std::fs::write(&out_path, &bytes)
        .with_context(|| format!("failed to write TTS output: {}", out_path.display()))?;

    Ok((asset_id, out_path))
}

fn resolve_cache_root(config: &VidraConfig) -> Result<PathBuf> {
    let raw = config
        .ai
        .cache_dir
        .as_deref()
        .unwrap_or(&config.resources.cache_dir);

    Ok(expand_tilde(raw)?)
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
    fn sha256_hex_is_stable() {
        let a = sha256_hex("hello");
        let b = sha256_hex("hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn expand_tilde_home() {
        let p = expand_tilde("~").unwrap();
        assert!(p.is_absolute());
    }

    #[test]
    fn parse_segments_from_verbose_json() {
        let raw = r#"{
                    "segments": [
                        {"start": 0.1, "end": 1.2, "text": " hello "},
                        {"start": 1.2, "end": 2.0, "text": "world"}
                    ]
                }"#;
        let segs = parse_verbose_json_segments(raw).unwrap();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "hello");
        assert!((segs[1].start_s - 1.2).abs() < 1e-6);
    }

    #[test]
    fn apply_segments_creates_children() {
        let mut layer = Layer::new(
            vidra_ir::layer::LayerId::new("captions"),
            LayerContent::AutoCaption {
                asset_id: AssetId::new("assets/audio.mp3"),
                font_family: "Inter".to_string(),
                font_size: 42.0,
                color: vidra_core::Color::WHITE,
            },
        );
        layer.transform.position = vidra_core::Point2D::new(10.0, 20.0);

        let segs = vec![CaptionSegment {
            start_s: 0.0,
            end_s: 1.0,
            text: "Hi".to_string(),
        }];

        apply_caption_segments(&mut layer, &segs, "Inter", 42.0, vidra_core::Color::WHITE);
        assert!(matches!(layer.content, LayerContent::Empty));
        assert_eq!(layer.children.len(), 1);
        assert_eq!(layer.children[0].transform.position.x, 10.0);
        assert_eq!(layer.children[0].animations.len(), 1);
    }

    #[test]
    fn gemini_caption_refine_cache_key_changes_with_input() {
        let segs_a = vec![CaptionSegment {
            start_s: 0.0,
            end_s: 1.0,
            text: "hello".to_string(),
        }];
        let segs_b = vec![CaptionSegment {
            start_s: 0.0,
            end_s: 1.0,
            text: "hello!".to_string(),
        }];

        let ja = serde_json::to_string(&vec![
            json!({"start_s": 0.0, "end_s": 1.0, "text": "hello"}),
        ])
        .unwrap();
        let jb = serde_json::to_string(&vec![
            json!({"start_s": 0.0, "end_s": 1.0, "text": "hello!"}),
        ])
        .unwrap();
        assert_ne!(sha256_hex(&ja), sha256_hex(&jb));

        // Keep variables used so the compiler doesn't warn in some configurations.
        assert_eq!(segs_a.len(), 1);
        assert_eq!(segs_b.len(), 1);
    }
}
