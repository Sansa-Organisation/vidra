//! # vidra-wasm
//!
//! WebAssembly module for the Vidra video engine.
//! Compiles VidraScript and renders frames in the browser.

mod renderer;

use renderer::WasmRenderer;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, serde::Deserialize)]
struct CaptionSegment {
    start_s: f64,
    end_s: f64,
    text: String,
}

thread_local! {
    /// Global renderer instance for the main WASM thread.
    static RENDERER: std::cell::RefCell<WasmRenderer> = std::cell::RefCell::new(WasmRenderer::new());
}

fn with_renderer<F, R>(f: F) -> R
where
    F: FnOnce(&mut WasmRenderer) -> R,
{
    RENDERER.with(|r| f(&mut *r.borrow_mut()))
}

/// Initialize the WASM module. Call this once before rendering.
#[wasm_bindgen]
pub fn init() {
    with_renderer(|_| {});
}

/// Parse VidraScript source and compile to IR JSON.
///
/// Returns a JSON string representing the project IR.
/// Throws a JS error if parsing fails.
#[wasm_bindgen]
pub fn parse_and_compile(source: &str) -> Result<String, JsValue> {
    // Lex → tokens
    let tokens = vidra_lang::lexer::Lexer::new(source)
        .tokenize()
        .map_err(|e| JsValue::from_str(&format!("Lex error: {}", e)))?;

    // Parse → AST
    let ast = vidra_lang::parser::Parser::new(tokens, "<wasm>")
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Compile → IR
    let project = vidra_lang::compiler::Compiler::compile(&ast)
        .map_err(|e| JsValue::from_str(&format!("Compile error: {}", e)))?;

    let json = serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    Ok(json)
}

/// Get project metadata from IR JSON.
///
/// Returns a JSON string: { width, height, fps, totalFrames, totalDuration, sceneCount }
#[wasm_bindgen]
pub fn get_project_info(ir_json: &str) -> Result<String, JsValue> {
    let project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let total_duration: f64 = project.scenes.iter().map(|s| s.duration.as_seconds()).sum();
    let total_frames = (total_duration * project.settings.fps).ceil() as u64;

    let info = serde_json::json!({
        "width": project.settings.width,
        "height": project.settings.height,
        "fps": project.settings.fps,
        "totalFrames": total_frames,
        "totalDuration": total_duration,
        "sceneCount": project.scenes.len(),
    });

    Ok(serde_json::to_string(&info).unwrap_or_default())
}

/// Load an image asset (as raw bytes) into the renderer cache.
///
/// Call this before rendering frames that reference the asset.
#[wasm_bindgen]
pub fn load_image_asset(asset_id: &str, data: &[u8]) {
    with_renderer(|r| {
        r.load_image_bytes(asset_id, data);
    });
}

/// Update the current mouse position (in pixel coordinates) for interactive previews.
///
/// Note: this currently does not affect rendering output yet; it is exposed as
/// plumbing for upcoming interactive expressions and event handling.
#[wasm_bindgen]
pub fn set_mouse_position(x: f64, y: f64) {
    with_renderer(|r| {
        r.set_mouse_position(x, y);
    });
}

/// Get the last mouse position set via `set_mouse_position`.
///
/// Returns a JSON string: { x, y }
#[wasm_bindgen]
pub fn get_mouse_position() -> String {
    let (x, y) = with_renderer(|r| r.mouse_position());
    serde_json::json!({ "x": x, "y": y }).to_string()
}

/// Set a numeric runtime state variable used by interactive expressions and event handlers.
#[wasm_bindgen]
pub fn set_state_var(name: &str, value: f64) {
    with_renderer(|r| {
        r.set_state_var(name, value);
    });
}

/// Get a numeric runtime state variable.
///
/// Returns `null` if unset.
#[wasm_bindgen]
pub fn get_state_var(name: &str) -> JsValue {
    let v = with_renderer(|r| r.get_state_var(name));
    match v {
        Some(x) => JsValue::from_f64(x),
        None => JsValue::NULL,
    }
}

/// Dispatch a click event at (x, y) for a given frame.
///
/// Returns a JSON string: { handled: bool, layerId?: string }
#[wasm_bindgen]
pub fn dispatch_click(ir_json: &str, frame_index: u32, x: f64, y: f64) -> Result<String, JsValue> {
    let project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let hit = with_renderer(|r| r.dispatch_click(&project, frame_index as u64, x, y));
    let out = serde_json::json!({
        "handled": hit.is_some(),
        "layerId": hit,
    });
    Ok(out.to_string())
}

/// Render a single frame and return RGBA pixel data.
///
/// Returns a `Vec<u8>` of length `width * height * 4`.
#[wasm_bindgen]
pub fn render_frame(ir_json: &str, frame_index: u32) -> Result<Vec<u8>, JsValue> {
    let project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let fb = with_renderer(|r| r.render_frame(&project, frame_index as u64));

    Ok(fb.data)
}

/// Get the computed transforms and bounds of all web layers at the given frame.
/// Returns a JSON string representing an array of { id, source, x, y, width, height, opacity, scaleX, scaleY }.
#[wasm_bindgen]
pub fn get_web_layers_state(ir_json: &str, frame_index: u32) -> Result<String, JsValue> {
    let project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let json = with_renderer(|r| r.get_web_layers_state(&project, frame_index as u64));

    Ok(json)
}

/// Render a single frame from VidraScript source directly.
///
/// Convenience method that combines parse + compile + render.
#[wasm_bindgen]
pub fn render_frame_from_source(source: &str, frame_index: u32) -> Result<Vec<u8>, JsValue> {
    let ir_json = parse_and_compile(source)?;
    render_frame(&ir_json, frame_index)
}

/// Materialize an `autocaption(...)` layer using caption segments provided by the JS host.
///
/// This enables web / React Native runtimes to do the network call for transcription and then
/// feed the result into Vidra as a deterministic IR update.
///
/// - `ir_json`: the project IR JSON string.
/// - `layer_id`: id of the layer whose content is `AutoCaption`.
/// - `segments_json`: JSON array of objects: { start_s, end_s, text }.
///
/// Returns an updated IR JSON string.
#[wasm_bindgen]
pub fn materialize_autocaption_layer(
    ir_json: &str,
    layer_id: &str,
    segments_json: &str,
) -> Result<String, JsValue> {
    let mut project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;
    let segments: Vec<CaptionSegment> = serde_json::from_str(segments_json)
        .map_err(|e| JsValue::from_str(&format!("segments_json parse error: {}", e)))?;

    let mut updated = false;
    for scene in &mut project.scenes {
        for layer in &mut scene.layers {
            if materialize_autocaption_in_layer(layer, layer_id, &segments)? {
                updated = true;
            }
        }
    }

    if !updated {
        return Err(JsValue::from_str(&format!(
            "no AutoCaption layer with id '{}' found",
            layer_id
        )));
    }

    serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Apply a background-removal patch to an image layer.
///
/// The JS host is responsible for calling the remote API (remove.bg / Clipdrop / etc) and
/// providing the resulting PNG-with-alpha via `load_image_asset(new_asset_id, bytes)`.
///
/// This function updates the IR to:
/// - swap `image(asset_id)` to the new asset id
/// - remove `effect(removeBackground)` from the layer
///
/// Returns updated IR JSON.
#[wasm_bindgen]
pub fn apply_remove_background_patch(
    ir_json: &str,
    layer_id: &str,
    new_asset_id: &str,
) -> Result<String, JsValue> {
    let mut project: vidra_ir::project::Project = serde_json::from_str(ir_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let mut updated = false;
    for scene in &mut project.scenes {
        for layer in &mut scene.layers {
            if apply_removebg_in_layer(layer, layer_id, new_asset_id)? {
                updated = true;
            }
        }
    }

    if !updated {
        return Err(JsValue::from_str(&format!(
            "no layer with id '{}' found",
            layer_id
        )));
    }

    serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

fn apply_removebg_in_layer(
    layer: &mut vidra_ir::layer::Layer,
    target_id: &str,
    new_asset_id: &str,
) -> Result<bool, JsValue> {
    if layer.id.to_string() == target_id {
        let has_removebg = layer
            .effects
            .iter()
            .any(|e| matches!(e, vidra_core::types::LayerEffect::RemoveBackground));
        if !has_removebg {
            return Err(JsValue::from_str(&format!(
                "layer '{}' does not have removeBackground effect",
                target_id
            )));
        }

        match &mut layer.content {
            vidra_ir::layer::LayerContent::Image { asset_id } => {
                *asset_id = vidra_ir::asset::AssetId::new(new_asset_id);
            }
            _ => {
                return Err(JsValue::from_str(&format!(
                    "layer '{}' is not an image() layer",
                    target_id
                )));
            }
        }

        layer
            .effects
            .retain(|e| !matches!(e, vidra_core::types::LayerEffect::RemoveBackground));
        return Ok(true);
    }

    for child in &mut layer.children {
        if apply_removebg_in_layer(child, target_id, new_asset_id)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn materialize_autocaption_in_layer(
    layer: &mut vidra_ir::layer::Layer,
    target_id: &str,
    segments: &[CaptionSegment],
) -> Result<bool, JsValue> {
    if layer.id.to_string() == target_id {
        let fields = match &layer.content {
            vidra_ir::layer::LayerContent::AutoCaption {
                font_family,
                font_size,
                color,
                ..
            } => Some((font_family.clone(), *font_size, *color)),
            _ => None,
        };

        let Some((font_family, font_size, color)) = fields else {
            return Err(JsValue::from_str(&format!(
                "layer '{}' exists but is not AutoCaption",
                target_id
            )));
        };

        if !layer.children.is_empty() {
            // Assume already materialized.
            return Ok(true);
        }

        apply_caption_segments(layer, segments, &font_family, font_size, color);
        return Ok(true);
    }

    for child in &mut layer.children {
        if materialize_autocaption_in_layer(child, target_id, segments)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn apply_caption_segments(
    layer: &mut vidra_ir::layer::Layer,
    segments: &[CaptionSegment],
    font_family: &str,
    font_size: f64,
    color: vidra_core::Color,
) {
    let base_pos = layer.transform.position;
    let base_anchor = layer.transform.anchor;
    let base_scale = layer.transform.scale;

    // Convert this node into a grouping layer.
    layer.content = vidra_ir::layer::LayerContent::Empty;

    for (i, seg) in segments.iter().enumerate() {
        let duration_s = (seg.end_s - seg.start_s).max(0.0);
        if duration_s <= 0.0 {
            continue;
        }

        let mut child = vidra_ir::layer::Layer::new(
            vidra_ir::layer::LayerId::new(format!("caption_{}", i)),
            vidra_ir::layer::LayerContent::Text {
                text: seg.text.trim().to_string(),
                font_family: font_family.to_string(),
                font_size,
                color,
            },
        );

        child.transform.position = base_pos;
        child.transform.anchor = base_anchor;
        child.transform.scale = base_scale;
        child.transform.opacity = 0.0;

        let fade = 0.06_f64.min((duration_s / 2.0).max(0.0));
        let mut anim =
            vidra_ir::animation::Animation::new(vidra_ir::animation::AnimatableProperty::Opacity)
                .with_delay(vidra_core::Duration::from_seconds(seg.start_s));
        anim.add_keyframe(vidra_ir::animation::Keyframe::new(
            vidra_core::Duration::zero(),
            0.0,
        ));
        anim.add_keyframe(
            vidra_ir::animation::Keyframe::new(vidra_core::Duration::from_seconds(fade), 1.0)
                .with_easing(vidra_core::types::Easing::EaseOut),
        );
        anim.add_keyframe(vidra_ir::animation::Keyframe::new(
            vidra_core::Duration::from_seconds((duration_s - fade).max(fade)),
            1.0,
        ));
        anim.add_keyframe(
            vidra_ir::animation::Keyframe::new(vidra_core::Duration::from_seconds(duration_s), 0.0)
                .with_easing(vidra_core::types::Easing::EaseIn),
        );
        child.animations.push(anim);

        layer.children.push(child);
    }
}

/// Get the version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
