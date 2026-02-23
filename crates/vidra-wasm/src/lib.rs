//! # vidra-wasm
//!
//! WebAssembly module for the Vidra video engine.
//! Compiles VidraScript and renders frames in the browser.

mod renderer;

use renderer::WasmRenderer;
use wasm_bindgen::prelude::*;

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

    let total_duration: f64 = project
        .scenes
        .iter()
        .map(|s| s.duration.as_seconds())
        .sum();
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

/// Render a single frame from VidraScript source directly.
///
/// Convenience method that combines parse + compile + render.
#[wasm_bindgen]
pub fn render_frame_from_source(source: &str, frame_index: u32) -> Result<Vec<u8>, JsValue> {
    let ir_json = parse_and_compile(source)?;
    render_frame(&ir_json, frame_index)
}

/// Get the version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
