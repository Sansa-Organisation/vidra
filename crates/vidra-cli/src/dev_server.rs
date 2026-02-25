use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, Response},
    routing::get,
    Router,
};
use notify::{EventKind, RecursiveMode, Watcher};
use parking_lot::RwLock;
use tokio::sync::broadcast;

use image::{ImageEncoder, RgbaImage};
use vidra_ir::project::Project;
use vidra_render::RenderPipeline;

/// Shared state for the dev server
struct DevState {
    project: Option<Project>,
    pipeline: RenderPipeline,
    error: Option<String>,
}

#[derive(Clone)]
struct AppState {
    file_path: PathBuf,
    dev_state: Arc<RwLock<DevState>>,
    tx: broadcast::Sender<String>, // broadcast for reloading
}

pub async fn run_dev_server(file: PathBuf) -> Result<()> {
    println!("🚀 Starting Vidra Dev Server...");

    // Initial compile
    let mut initial_state = DevState {
        project: None,
        pipeline: RenderPipeline::new().expect("Failed to init GPU pipeline"),
        error: None,
    };

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
        dev_state: Arc::new(RwLock::new(initial_state)),
        tx: tx.clone(),
    };

    // Start file watcher
    let watch_state = app_state.clone();
    let watch_file = file.clone();

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    println!("🔄 File changed, recompiling...");
                    let mut lock = watch_state.dev_state.write();
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
        file.parent().unwrap_or(Path::new(".")),
        RecursiveMode::Recursive,
    )?;

    let app = Router::new()
        .route("/", get(index_html))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("📡 Dev Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn compile_and_load(file: &PathBuf) -> Result<Project> {
    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let ast = crate::parse_and_resolve_imports(file)?;

    let checker = vidra_lang::TypeChecker::new(file_name.to_string());
    checker.check(&ast).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Type errors:\n  {}", msgs.join("\n  "))
    })?;

    let mut project = vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Best-effort config load (dev server can run from a standalone file).
    let config = vidra_core::VidraConfig::load_from_file(std::path::Path::new("vidra.config.toml"))
        .unwrap_or_default();

    // Ensure any remote assets are fetched into the local cache so the renderer can load them.
    crate::remote_assets::prepare_project_remote_assets(&mut project, &config)?;

    vidra_ir::validate::validate_project(&project).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Validation errors:\n  {}", msgs.join("\n  "))
    })?;

    Ok(project)
}

// --- HTTP Handlers ---

async fn index_html() -> Html<&'static str> {
    Html(include_str!("dev_ui.html"))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Send initial metadata
    send_metadata(&mut socket, &state).await;

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(m) = msg {
                    if m == "reload" {
                        send_metadata(&mut socket, &state).await;
                    } else if m == "error" {
                        let error_msg = {
                            let lock = state.dev_state.read();
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
                                // Render frame in a blocking task so we don't block the async executor
                                let state_clone = state.clone();
                                let img_res = tokio::task::spawn_blocking(move || {
                                    render_frame_to_jpeg(&state_clone, frame_idx)
                                }).await;

                                match img_res {
                                    Ok(Some(jpeg_bytes)) => {
                                        // Send generic response and then binary data
                                        let _ = socket.send(Message::Binary(jpeg_bytes.into())).await;
                                    }
                                    _ => {
                                        // Ignore or send error
                                    }
                                }
                            }
                        } else if json["type"] == "REQUEST_BOUNDS" {
                            if let Some(frame_idx) = json["frame"].as_u64() {
                                let state_clone = state.clone();
                                let res = tokio::task::spawn_blocking(move || {
                                    let lock = state_clone.dev_state.read();
                                    if let Some(proj) = &lock.project {
                                        lock.pipeline.inspect_frame_bounds(proj, frame_idx).ok()
                                    } else {
                                        None
                                    }
                                }).await.ok().flatten();

                                if let Some(bounds) = res {
                                    let payload = serde_json::json!({
                                        "type": "INSPECT_BOUNDS",
                                        "frame": frame_idx,
                                        "bounds": bounds,
                                    });
                                    let _ = socket.send(Message::Text(payload.to_string().into())).await;
                                }
                            }
                        } else if json["type"] == "GO_TO_SOURCE" {
                            if let Some(id) = json["id"].as_str() {
                                // Simulate click element -> source map
                                let file_path = std::fs::canonicalize(&state.file_path).unwrap_or_else(|_| state.file_path.clone());
                                let mut line_no = 1;
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    for (i, line) in content.lines().enumerate() {
                                        // Simple heuristic to find layer declaration
                                        if line.contains(&format!("\"{}\"", id)) || line.contains(&format!("layer {}", id)) {
                                            line_no = i + 1;
                                            break;
                                        }
                                    }
                                }
                                println!("⚡ Jump to Source: Layer '{}' -> {}:{}", id, file_path.display(), line_no);
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
        let lock = state.dev_state.read();
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

fn render_frame_to_jpeg(state: &AppState, frame_idx: u64) -> Option<Vec<u8>> {
    let lock = state.dev_state.read();
    let proj = lock.project.as_ref()?;

    // Bounds check
    if frame_idx >= proj.total_frames() {
        return None;
    }

    // Render the specific frame using the public api
    match lock.pipeline.render_frame_index(proj, frame_idx) {
        Ok(fb) => {
            // Encode to JPEG in memory
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
