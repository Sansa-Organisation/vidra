// ─── vidra editor server ────────────────────────────────────────────
//
// An extended version of the dev server, designed for the visual editor.
// Reuses compile_and_load(), file watcher, and WebSocket broadcast from
// the dev server pattern, and adds REST API routes for project
// manipulation, rendering, asset management, and MCP relay.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        Multipart, State, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use notify::{EventKind, RecursiveMode, Watcher};
use parking_lot::RwLock;
use serde::Deserialize;
use tokio::sync::broadcast;

use image::{ImageEncoder, RgbaImage};
use vidra_ir::project::Project;
use vidra_render::RenderPipeline;

// ── Shared state ────────────────────────────────────────────────────

struct EditorState {
    project: Option<Project>,
    source_text: Option<String>,
    pipeline: RenderPipeline,
    error: Option<String>,
}

#[derive(Clone)]
struct AppState {
    file_path: PathBuf,
    editor_state: Arc<RwLock<EditorState>>,
    tx: broadcast::Sender<String>,
}

// ── Entry point ─────────────────────────────────────────────────────

pub async fn run_editor_server(file: PathBuf, port: u16, open: bool) -> Result<()> {
    println!("🎬 Starting Vidra Editor Server...");

    let mut initial_state = EditorState {
        project: None,
        source_text: None,
        pipeline: RenderPipeline::new().expect("Failed to init GPU pipeline"),
        error: None,
    };

    // Read source text
    let source = std::fs::read_to_string(&file).unwrap_or_default();
    initial_state.source_text = Some(source);

    match compile_and_load(&file) {
        Ok(proj) => {
            if let Err(e) = initial_state.pipeline.load_assets(&proj) {
                println!("   ✗ Asset load failed: {}", e);
                initial_state.error = Some(e.to_string());
            } else {
                println!("   ✓ Initial compile OK");
                initial_state.project = Some(proj);
            }
        }
        Err(e) => {
            println!("   ✗ Initial compile failed: {}", e);
            initial_state.error = Some(e.to_string());
        }
    }

    let (tx, _rx) = broadcast::channel(16);
    let app_state = AppState {
        file_path: file.clone(),
        editor_state: Arc::new(RwLock::new(initial_state)),
        tx: tx.clone(),
    };

    // File watcher
    let watch_state = app_state.clone();
    let watch_file = file.clone();

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    println!("🔄 File changed, recompiling...");
                    let mut lock = watch_state.editor_state.write();
                    // Re-read source text
                    if let Ok(src) = std::fs::read_to_string(&watch_file) {
                        lock.source_text = Some(src);
                    }
                    match compile_and_load(&watch_file) {
                        Ok(proj) => {
                            if let Err(e) = lock.pipeline.load_assets(&proj) {
                                println!("   ✗ Asset load failed: {}", e);
                                lock.error = Some(e.to_string());
                                let _ = watch_state.tx.send("error".to_string());
                            } else {
                                println!("   ✓ Hot reload OK");
                                lock.project = Some(proj);
                                lock.error = None;
                                let _ = watch_state.tx.send("reload".to_string());
                            }
                        }
                        Err(e) => {
                            println!("   ✗ Hot reload failed: {}", e);
                            lock.error = Some(e.to_string());
                            let _ = watch_state.tx.send("error".to_string());
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(
        file.parent().unwrap_or(std::path::Path::new(".")),
        RecursiveMode::Recursive,
    )?;

    // Build router
    let app = Router::new()
        // WebSocket
        .route("/ws", get(ws_handler))
        // Project API (8.4)
        .route("/api/project", get(get_project).put(put_project))
        .route(
            "/api/project/source",
            get(get_project_source).put(put_project_source),
        )
        .route("/api/project/patch", post(patch_project))
        // Render API (8.5)
        .route("/api/render/frame", post(render_frame))
        .route("/api/render/export", post(render_export))
        // MCP relay (8.6)
        .route("/api/mcp/invoke", post(mcp_invoke))
        // Asset API (8.7)
        .route("/api/assets", get(list_assets))
        .route("/api/assets/upload", post(upload_asset))
        .route("/api/assets/{id}", delete(delete_asset))
        // Serve local assets dir explicitly
        .nest_service(
            "/assets_local",
            tower_http::services::ServeDir::new(
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("assets")
            )
        )
        // LLM proxy stub (8.8)
        .route("/api/ai/chat", post(ai_chat))
        .with_state(app_state)
        // Embedded frontend fallback (SPA routing)
        .fallback(static_handler);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("📡 Editor Server listening on http://{}", addr);

    if open {
        let url = format!("http://127.0.0.1:{}", port);
        let _ = open_browser(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ── Compilation helper (reused from dev_server pattern) ─────────────

fn compile_and_load(file: &PathBuf) -> Result<Project> {
    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let ast = crate::parse_and_resolve_imports(file)?;

    let checker = vidra_lang::TypeChecker::new(file_name.to_string());
    checker.check(&ast).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Type errors:\n  {}", msgs.join("\n  "))
    })?;

    let mut project = vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?;

    let config = vidra_core::VidraConfig::load_from_file(std::path::Path::new("vidra.config.toml"))
        .unwrap_or_default();
    crate::remote_assets::prepare_project_remote_assets(&mut project, &config)?;

    vidra_ir::validate::validate_project(&project).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Validation errors:\n  {}", msgs.join("\n  "))
    })?;

    Ok(project)
}

// ── Embedded frontend ───────────────────────────────────────────────

#[derive(rust_embed::RustEmbed)]
#[folder = "../../packages/vidra-editor/dist/"]
struct Asset;

async fn static_handler(uri: axum::extract::OriginalUri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() || path == "index.html" {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path != "index.html" {
                // SPA fallback for routing
                if let Some(content) = Asset::get("index.html") {
                    let mime = mime_guess::from_path("index.html").first_or_text_plain();
                    return ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response();
                }
            }
            (StatusCode::NOT_FOUND, "404 Not Found").into_response()
        }
    }
}

// ── WebSocket handler ───────────────────────────────────────────────

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    send_metadata(&mut socket, &state).await;

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(m) = msg {
                    if m == "reload" {
                        send_metadata(&mut socket, &state).await;
                    } else if m == "error" {
                        let error_msg = {
                            let lock = state.editor_state.read();
                            lock.error.clone()
                        };
                        if let Some(e) = error_msg {
                            let payload = serde_json::json!({
                                "type": "ERROR",
                                "message": e
                            });
                            let _ = socket.send(Message::Text(payload.to_string().into())).await;
                        }
                    }
                }
            }
            Some(msg) = socket.recv() => {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if json["type"] == "REQUEST_FRAME" {
                            if let Some(frame_idx) = json["frame"].as_u64() {
                                let state_clone = state.clone();
                                let img_res = tokio::task::spawn_blocking(move || {
                                    render_frame_to_jpeg(&state_clone, frame_idx)
                                }).await;
                                match img_res {
                                    Ok(Some(jpeg_bytes)) => {
                                        let _ = socket.send(Message::Binary(jpeg_bytes.into())).await;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn send_metadata(socket: &mut WebSocket, state: &AppState) {
    let meta_str = {
        let lock = state.editor_state.read();
        if let Some(proj) = &lock.project {
            let meta = serde_json::json!({
                "type": "METADATA",
                "width": proj.settings.width,
                "height": proj.settings.height,
                "fps": proj.settings.fps,
                "total_frames": proj.total_frames()
            });
            Some(meta.to_string())
        } else {
            None
        }
    };
    if let Some(msg) = meta_str {
        let _ = socket.send(Message::Text(msg.into())).await;
    }
}

// ── Project API (8.4) ───────────────────────────────────────────────

async fn get_project(State(state): State<AppState>) -> impl IntoResponse {
    let lock = state.editor_state.read();
    if let Some(proj) = &lock.project {
        match serde_json::to_string_pretty(proj) {
            Ok(json) => (StatusCode::OK, Json(serde_json::json!({ "ir": json }))),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ),
        }
    } else {
        let err = lock
            .error
            .clone()
            .unwrap_or_else(|| "No project loaded".into());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err })),
        )
    }
}

async fn put_project(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ir_json = match body.get("ir").and_then(|v| v.as_str()) {
        Some(j) => j.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "missing `ir` field" })),
            )
        }
    };

    let project: Project = match serde_json::from_str(&ir_json) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid IR: {}", e) })),
            )
        }
    };

    {
        let mut lock = state.editor_state.write();
        if let Err(e) = lock.pipeline.load_assets(&project) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
        lock.project = Some(project);
        lock.error = None;
    }

    let _ = state.tx.send("reload".to_string());
    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

async fn get_project_source(State(state): State<AppState>) -> impl IntoResponse {
    let lock = state.editor_state.read();
    let src = lock.source_text.clone().unwrap_or_default();
    (StatusCode::OK, Json(serde_json::json!({ "source": src })))
}

async fn put_project_source(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let source = match body.get("source").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "missing `source` field" })),
            )
        }
    };

    // Write to disk
    if let Err(e) = std::fs::write(&state.file_path, &source) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    // Recompile
    {
        let mut lock = state.editor_state.write();
        lock.source_text = Some(source);
        match compile_and_load(&state.file_path) {
            Ok(proj) => {
                if let Err(e) = lock.pipeline.load_assets(&proj) {
                    lock.error = Some(e.to_string());
                    let _ = state.tx.send("error".to_string());
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    );
                }
                lock.project = Some(proj);
                lock.error = None;
            }
            Err(e) => {
                lock.error = Some(e.to_string());
                let _ = state.tx.send("error".to_string());
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
    }

    let _ = state.tx.send("reload".to_string());
    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct PatchBody {
    layer_id: String,
    properties: serde_json::Value,
}

async fn patch_project(
    State(state): State<AppState>,
    Json(body): Json<PatchBody>,
) -> impl IntoResponse {
    let file = state.file_path.clone();
    match crate::mcp_tools::apply_layer_properties_to_vidra_file(
        &file,
        &body.layer_id,
        &body.properties,
    ) {
        Ok(true) => {
            // Recompile after patch
            let mut lock = state.editor_state.write();
            if let Ok(src) = std::fs::read_to_string(&file) {
                lock.source_text = Some(src);
            }
            match compile_and_load(&file) {
                Ok(proj) => {
                    let _ = lock.pipeline.load_assets(&proj);
                    lock.project = Some(proj);
                    lock.error = None;
                }
                Err(e) => {
                    lock.error = Some(e.to_string());
                }
            }
            drop(lock);
            let _ = state.tx.send("reload".to_string());
            (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "layer not found or no changes" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ── Render API (8.5) ────────────────────────────────────────────────

#[derive(Deserialize)]
struct RenderFrameBody {
    frame: u64,
    #[serde(default = "default_format")]
    format: String,
}
fn default_format() -> String {
    "jpeg".into()
}

async fn render_frame(
    State(state): State<AppState>,
    Json(body): Json<RenderFrameBody>,
) -> Response {
    let state_clone = state.clone();
    let result =
        tokio::task::spawn_blocking(move || render_frame_to_jpeg(&state_clone, body.frame)).await;

    match result {
        Ok(Some(bytes)) => {
            let content_type = if body.format == "png" {
                "image/png"
            } else {
                "image/jpeg"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from(bytes))
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Render failed"))
            .unwrap(),
    }
}

async fn render_export(State(state): State<AppState>) -> impl IntoResponse {
    // Full export is a long-running operation; for now, acknowledge the request.
    // Progress will be streamed via WebSocket in a future iteration.
    let has_project = state.editor_state.read().project.is_some();
    if !has_project {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No project loaded" })),
        );
    }

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "export_queued",
            "message": "Export started. Progress will be streamed via WebSocket."
        })),
    )
}

// ── MCP Relay (8.6) ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct McpInvokeBody {
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

async fn mcp_invoke(
    State(_state): State<AppState>,
    Json(body): Json<McpInvokeBody>,
) -> impl IntoResponse {
    let result = crate::mcp::execute_tool_public(&body.name, body.arguments).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({ "result": result })),
    )
}

// ── Asset API (8.7) ─────────────────────────────────────────────────

async fn list_assets() -> impl IntoResponse {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let assets_dir = root.join("assets");
    let mut assets = Vec::new();

    if assets_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&assets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    assets.push(serde_json::json!({
                        "id": name,
                        "path": path.display().to_string(),
                        "size": size,
                    }));
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "assets": assets })),
    )
}

async fn upload_asset(mut multipart: Multipart) -> impl IntoResponse {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let assets_dir = root.join("assets");
    let _ = std::fs::create_dir_all(&assets_dir);

    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "No file provided" })),
            )
        }
    };

    let filename = field.file_name().unwrap_or("upload").to_string();

    // Basic file type validation
    let ext = std::path::Path::new(&filename)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let allowed = [
        "png", "jpg", "jpeg", "gif", "webp", "svg", "mp4", "webm", "mp3", "wav", "ogg", "ttf",
        "otf", "woff2", "html",
    ];
    if !allowed.contains(&ext.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("File type .{} not allowed", ext) })),
        );
    }

    let data: axum::body::Bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("{}", e) })),
            )
        }
    };

    // Size limit: 50MB
    if data.len() > 50 * 1024 * 1024 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "File too large (50MB limit)" })),
        );
    }

    let out_path = assets_dir.join(&filename);
    if let Err(e) = std::fs::write(&out_path, &data) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "id": filename,
            "path": out_path.display().to_string(),
            "size": data.len(),
        })),
    )
}

async fn delete_asset(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let path = root.join("assets").join(&id);

    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Asset '{}' not found", id) })),
        );
    }

    match std::fs::remove_file(&path) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "deleted": id })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ── LLM Proxy stub (8.8) ───────────────────────────────────────────

#[derive(Deserialize)]
struct AiChatBody {
    messages: Vec<serde_json::Value>,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default)]
    #[allow(dead_code)]
    provider: String,
}
fn default_model() -> String {
    "gpt-4".into()
}

async fn ai_chat(State(state): State<AppState>, Json(body): Json<AiChatBody>) -> impl IntoResponse {
    // Security: API keys come from server-side env vars only.
    let api_key = std::env::var("VIDRA_AI_API_KEY").ok();
    if api_key.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "AI provider not configured. Set VIDRA_AI_API_KEY env var."
            })),
        );
    }

    // Build system prompt with project context
    let project_ctx = {
        let lock = state.editor_state.read();
        if let Some(proj) = &lock.project {
            format!(
                "Current project: {}x{} @ {}fps, {} scenes",
                proj.settings.width,
                proj.settings.height,
                proj.settings.fps,
                proj.scenes.len()
            )
        } else {
            "No project loaded.".to_string()
        }
    };

    // For now, return a stub response; full LLM integration is out-of-scope for this phase.
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "response": format!("AI chat stub. {} messages received. Project context: {}. Model: {}",
                body.messages.len(), project_ctx, body.model),
            "model": body.model,
        })),
    )
}

// ── Helpers ─────────────────────────────────────────────────────────

fn render_frame_to_jpeg(state: &AppState, frame_idx: u64) -> Option<Vec<u8>> {
    let lock = state.editor_state.read();
    let proj = lock.project.as_ref()?;
    if frame_idx >= proj.total_frames() {
        return None;
    }

    match lock.pipeline.render_frame_index(proj, frame_idx) {
        Ok(fb) => {
            let img = RgbaImage::from_raw(fb.width, fb.height, fb.data)?;
            let mut out = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85);
            encoder
                .write_image(&img, fb.width, fb.height, image::ExtendedColorType::Rgba8)
                .ok()?;
            Some(out)
        }
        Err(e) => {
            println!("Render error for frame {}: {}", frame_idx, e);
            None
        }
    }
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/c", "start", url])
        .spawn()?;
    Ok(())
}
