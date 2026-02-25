use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Duplicate the real stdout file descriptor and then redirect fd 1 (stdout)
/// to fd 2 (stderr).  Returns an owned `std::fs::File` wrapping the duplicated
/// fd so the MCP server can write JSON-RPC on the original stdout while every
/// other `println!` / `print!` in the process goes to stderr.
#[cfg(unix)]
pub fn redirect_stdout_to_stderr() -> std::fs::File {
    use std::os::unix::io::FromRawFd;
    unsafe {
        // dup(1) → new fd that points at the real stdout
        let saved = libc::dup(1);
        assert!(saved >= 0, "dup(1) failed");
        // redirect fd 1 → fd 2 (stderr)
        libc::dup2(2, 1);
        std::fs::File::from_raw_fd(saved)
    }
}

#[cfg(not(unix))]
pub fn redirect_stdout_to_stderr() -> std::fs::File {
    // On non-unix, fall back to regular stdout (best-effort).
    // Windows MCP hosts typically use named pipes, not stdio.
    let stdout = std::io::stdout();
    // This is a fallback — it won't truly redirect, but it's the
    // best we can do without platform-specific Windows HANDLEs.
    unsafe {
        use std::os::windows::io::{AsRawHandle, FromRawHandle};
        let h = stdout.as_raw_handle();
        std::fs::File::from_raw_handle(h)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerCapabilities {
    tools: serde_json::Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct InitializeResult {
    protocolVersion: String,
    capabilities: ServerCapabilities,
    serverInfo: ServerInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    name: String,
    version: String,
}

pub async fn run_mcp_server(saved_stdout: std::fs::File) -> Result<()> {
    let stdin = tokio::io::stdin();
    // Use the saved (real) stdout fd for JSON-RPC output.  Regular stdout
    // has already been redirected to stderr by redirect_stdout_to_stderr().
    let mut stdout = tokio::io::BufWriter::new(tokio::fs::File::from_std(saved_stdout));
    let mut reader = BufReader::new(stdin).lines();

    tracing::info!("Starting MCP Server...");

    while let Some(line) = reader.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        // Parse incoming message
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let res = handle_request(&req).await;
            if let Some(mut response) = res {
                response.jsonrpc = "2.0".to_string();
                let out = serde_json::to_string(&response)?;
                stdout.write_all(out.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        } else {
            tracing::error!("Failed to parse JSON-RPC request: {}", line);
        }
    }

    Ok(())
}

async fn handle_request(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => {
            let result = serde_json::to_value(InitializeResult {
                protocolVersion: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: serde_json::Map::new(),
                },
                serverInfo: ServerInfo {
                    name: "vidra-mcp".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            })
            .ok();

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result,
                error: None,
            })
        }
        "tools/list" => {
            let tools = vec![
                serde_json::json!({
                    "name": "vidra-create_project",
                    "description": "Create a new Vidra project with specified parameters",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "width": { "type": "number", "default": 1920 },
                            "height": { "type": "number", "default": 1080 },
                            "fps": { "type": "number", "default": 60 }
                        },
                        "required": ["name"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-add_scene",
                    "description": "Add a new scene to the project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "duration_seconds": { "type": "number" },
                            "project_file": { "type": "string", "default": "main.vidra" }
                        },
                        "required": ["name", "duration_seconds"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-render_preview",
                    "description": "Trigger a local preview render",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_file": { "type": "string" }
                        },
                        "required": ["project_file"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-edit_layer",
                    "description": "Edit properties of a semantic layer path",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "scene_id": { "type": "string" },
                            "layer_path": { "type": "string" },
                            "properties": { "type": "object" },
                            "project_file": { "type": "string", "default": "main.vidra" }
                        },
                        "required": ["scene_id", "layer_path", "properties"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-set_style",
                    "description": "Set style properties for a component or layer",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "target_id": { "type": "string" },
                            "style_props": { "type": "object" },
                            "project_file": { "type": "string", "default": "main.vidra" }
                        },
                        "required": ["target_id", "style_props"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-apply_brand_kit",
                    "description": "Apply a brand kit to the project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "kit_name": { "type": "string" }
                        },
                        "required": ["kit_name"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-add_asset",
                    "description": "Register a new media asset to the project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "path": { "type": "string" },
                            "type": { "type": "string" }
                        },
                        "required": ["id", "path", "type"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-list_templates",
                    "description": "List available video templates",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "vidra-share",
                    "description": "Generate a shareable link for a video file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "file": { "type": "string" }
                        },
                        "required": ["file"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-add_resource",
                    "description": "Pull a resource from Vidra Commons",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "resource_id": { "type": "string" }
                        },
                        "required": ["resource_id"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-list_resources",
                    "description": "Search the resource library",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string" }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "vidra-storyboard",
                    "description": "Generate a visual storyboard from text",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string" }
                        },
                        "required": ["prompt"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-add_web_scene",
                    "description": "Add a web() layer to a scene, capturing a URL or local HTML/React page as video frames",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "scene_id": { "type": "string", "description": "Scene name to add the web layer to" },
                            "source": { "type": "string", "description": "URL or local file path for the web content" },
                            "viewport_width": { "type": "number", "default": 1920 },
                            "viewport_height": { "type": "number", "default": 1080 },
                            "mode": { "type": "string", "enum": ["frame-accurate", "realtime"], "default": "frame-accurate" },
                            "duration_seconds": { "type": "number", "default": 5.0 },
                            "layer_id": { "type": "string", "description": "Layer id (auto-generated if omitted)" },
                            "variables": { "type": "object", "description": "Key-value variables passed to the web content" },
                            "project_file": { "type": "string", "default": "main.vidra" }
                        },
                        "required": ["scene_id", "source"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-edit_web_scene",
                    "description": "Edit properties of an existing web() layer (source, viewport, mode, wait_for, variables)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "layer_id": { "type": "string", "description": "Layer id of the web layer to edit" },
                            "source": { "type": "string" },
                            "viewport_width": { "type": "number" },
                            "viewport_height": { "type": "number" },
                            "mode": { "type": "string", "enum": ["frame-accurate", "realtime"] },
                            "wait_for": { "type": "string" },
                            "variables": { "type": "object" },
                            "project_file": { "type": "string", "default": "main.vidra" }
                        },
                        "required": ["layer_id"]
                    }
                }),
                serde_json::json!({
                    "name": "vidra-generate_web_code",
                    "description": "Generate an HTML file suitable for use as a web() layer source. Writes to web/ directory.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string", "description": "Output filename (e.g. 'hero.html')" },
                            "html_content": { "type": "string", "description": "Full HTML content to write" }
                        },
                        "required": ["filename", "html_content"]
                    }
                }),
            ];

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: Some(serde_json::json!({ "tools": tools })),
                error: None,
            })
        }
        "tools/call" => {
            if let Some(params) = &req.params {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let tool_result = execute_tool(name, args).await;

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: Some(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": tool_result
                            }
                        ]
                    })),
                    error: None,
                })
            } else {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(serde_json::json!({
                        "code": -32602,
                        "message": "Invalid params"
                    })),
                })
            }
        }
        "notifications/initialized" | "$/cancelRequest" => None,
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: Some(serde_json::json!({
                "code": -32601,
                "message": "Method not found"
            })),
        }),
    }
}

/// Public entry point for invoking MCP tools from non-stdio contexts (e.g. the editor server).
pub async fn execute_tool_public(name: &str, args: Value) -> String {
    execute_tool(name, args).await
}

async fn execute_tool(name: &str, args: Value) -> String {
    match name {
        "vidra-create_project" => {
            let pd_name = args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("project");
            let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
            let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(1080) as u32;
            let fps = args.get("fps").and_then(|v| v.as_u64()).unwrap_or(60) as u32;
            match crate::mcp_tools::create_project(pd_name, width, height, fps) {
                Ok(path) => format!("✅ Project '{}' created at {}", pd_name, path.display()),
                Err(e) => format!("❌ Failed to create project '{}': {:#}", pd_name, e),
            }
        }
        "vidra-add_scene" => {
            let scene_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("scene");
            let duration_seconds = args
                .get("duration_seconds")
                .and_then(|v| v.as_f64())
                .unwrap_or(3.0);
            let project_file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");
            let file = std::path::PathBuf::from(project_file);
            match crate::mcp_tools::add_scene_to_vidra_file(&file, scene_name, duration_seconds) {
                Ok(()) => format!("✅ Scene '{}' added to {}", scene_name, file.display()),
                Err(e) => format!("❌ Failed to add scene '{}': {:#}", scene_name, e),
            }
        }
        "vidra-render_preview" => {
            let file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");
            let path = std::path::PathBuf::from(file);
            match crate::cmd_preview(path, false) {
                Ok(()) => "✅ Preview rendered at output/preview.mp4".to_string(),
                Err(e) => format!("❌ Preview render failed: {:#}", e),
            }
        }
        "vidra-edit_layer" => {
            let scene_id = args
                .get("scene_id")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let layer_path = args
                .get("layer_path")
                .and_then(|v| v.as_str())
                .unwrap_or("layer");
            let props = args
                .get("properties")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let project_file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");
            let file = std::path::PathBuf::from(project_file);
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let apply_msg = match crate::mcp_tools::apply_layer_properties_to_vidra_file(
                &file, layer_path, &props,
            ) {
                Ok(true) => format!("updated {}", file.display()),
                Ok(false) => "no direct file change (layer not found or unsupported property keys)"
                    .to_string(),
                Err(e) => format!("direct edit failed: {:#}", e),
            };
            match crate::mcp_tools::record_layer_edit(&root, scene_id, layer_path, props) {
                Ok(path) => format!(
                    "✅ Layer edit recorded for '{}' at {} ({})",
                    layer_path,
                    path.display(),
                    apply_msg
                ),
                Err(e) => format!("❌ Failed to record layer edit '{}': {:#}", layer_path, e),
            }
        }
        "vidra-set_style" => {
            let target = args
                .get("target_id")
                .and_then(|v| v.as_str())
                .unwrap_or("target");
            let style_props = args
                .get("style_props")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let project_file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");
            let file = std::path::PathBuf::from(project_file);
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let apply_msg = match crate::mcp_tools::apply_layer_properties_to_vidra_file(
                &file,
                target,
                &style_props,
            ) {
                Ok(true) => format!("updated {}", file.display()),
                Ok(false) => {
                    "no direct file change (target not found or unsupported style keys)".to_string()
                }
                Err(e) => format!("direct style edit failed: {:#}", e),
            };
            match crate::mcp_tools::record_style_set(&root, target, style_props) {
                Ok(path) => format!(
                    "✅ Style update recorded for '{}' at {} ({})",
                    target,
                    path.display(),
                    apply_msg
                ),
                Err(e) => format!("❌ Failed to record style update '{}': {:#}", target, e),
            }
        }
        "vidra-apply_brand_kit" => {
            let kit = args
                .get("kit_name")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            match crate::cmd_brand_apply(kit) {
                Ok(()) => format!("✅ Brand kit '{}' applied.", kit),
                Err(e) => format!("❌ Failed to apply brand kit '{}': {:#}", kit, e),
            }
        }
        "vidra-add_asset" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("asset");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let asset_type = args
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            if path.is_empty() {
                return "❌ Missing required `path`".to_string();
            }
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let reg = crate::mcp_tools::register_asset(&root, id, path, asset_type);
            let queue = crate::sync_tools::enqueue_upload_path(&root, std::path::Path::new(path));
            match (reg, queue) {
                (Ok(meta), Ok(n)) => format!(
                    "✅ Asset '{}' registered at {} ({} file(s) queued for upload)",
                    id,
                    meta.display(),
                    n
                ),
                (Ok(meta), Err(_)) => format!("✅ Asset '{}' registered at {}", id, meta.display()),
                (Err(e), _) => format!("❌ Failed to register asset '{}': {:#}", id, e),
            }
        }
        "vidra-list_templates" => {
            let names: Vec<&str> = crate::template_manager::available_templates()
                .into_iter()
                .map(|t| t.name)
                .collect();
            format!("✅ Available templates: {}", names.join(", "))
        }
        "vidra-share" => {
            let file = args
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("preview.mp4");
            let path = std::path::PathBuf::from(file);
            if !path.exists() {
                return format!("❌ File not found: {}", path.display());
            }

            let project_root =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let _ = crate::sync_tools::enqueue_upload_path(&project_root, &path)
                .map_err(|e| format!("❌ Failed to queue upload: {:#}", e))
                .map(|_| ());

            let hash = match crate::sha256_file_prefixed(&path) {
                Ok(h) => h,
                Err(e) => return format!("❌ Failed to hash file: {:#}", e),
            };
            let share_id = hash.trim_start_matches("sha256:");
            let link = if let Some(base) = crate::sync_cloud::cloud_base_url_from_env() {
                match crate::sync_cloud::push_uploads_to_cloud(&project_root, &base) {
                    Ok(res) if !res.failures.is_empty() => {
                        return format!(
                            "⚠️ Share queued but upload had failures ({}). Link (may not be live yet): {}/share/{}",
                            res.failures.len(),
                            base,
                            share_id
                        );
                    }
                    Ok(_) => format!("{}/share/{}", base, share_id),
                    Err(e) => {
                        return format!(
                            "⚠️ Share queued locally but cloud upload failed: {:#}. Link (may not be live yet): {}/share/{}",
                            e,
                            base,
                            share_id
                        );
                    }
                }
            } else {
                format!("vidra://share/{}", share_id)
            };

            format!("✅ Share link for '{}': {}", file, link)
        }
        "vidra-add_resource" => {
            let res = args
                .get("resource_id")
                .and_then(|v| v.as_str())
                .unwrap_or("resource");
            // Local-first: treat resources as built-in templates for now.
            let known = crate::template_manager::available_templates()
                .into_iter()
                .any(|t| t.name == res);
            if !known {
                return format!("❌ Unknown resource '{}'. Try: vidra-list_resources", res);
            }
            match crate::template_manager::execute_add(res) {
                Ok(()) => format!("✅ Added resource '{}' to current project.", res),
                Err(e) => format!("❌ Failed to add resource '{}': {:#}", res, e),
            }
        }
        "vidra-list_resources" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let q = query.to_lowercase();
            let templates = crate::template_manager::available_templates();
            let mut matches: Vec<&'static str> = templates
                .into_iter()
                .filter(|t| {
                    q.is_empty() || t.name.contains(&q) || t.description.to_lowercase().contains(&q)
                })
                .map(|t| t.name)
                .collect();
            matches.sort();
            if matches.is_empty() {
                return format!("✅ Search results for '{}': (none)", query);
            }
            format!("✅ Search results for '{}': {}", query, matches.join(", "))
        }
        "vidra-storyboard" => {
            let prompt = args
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let out = args
                .get("output")
                .and_then(|v| v.as_str())
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("storyboard.png"));
            match crate::storyboard_tools::generate_storyboard_png(prompt, &out) {
                Ok(()) => format!("✅ Storyboard saved to '{}'", out.display()),
                Err(e) => format!("❌ Failed to generate storyboard: {:#}", e),
            }
        }
        "vidra-add_web_scene" => {
            let scene_id = args
                .get("scene_id")
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("");
            if source.is_empty() {
                return "❌ Missing required `source` parameter".to_string();
            }
            let vp_w = args
                .get("viewport_width")
                .and_then(|v| v.as_u64())
                .unwrap_or(1920);
            let vp_h = args
                .get("viewport_height")
                .and_then(|v| v.as_u64())
                .unwrap_or(1080);
            let mode = args
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("frame-accurate");
            let layer_id = args
                .get("layer_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("web_{}", scene_id));
            let project_file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");

            // Build the web() layer line
            let mut web_args = format!("source: \"{}\")", source);
            web_args = format!(
                "source: \"{}\", viewport: {}x{}, mode: {}",
                source, vp_w, vp_h, mode
            );

            // Build the layer block
            let layer_block = format!(
                "    Layer(\"{}\") {{\n        web({})\n    }}\n",
                layer_id, web_args
            );

            // Find the scene and insert the layer
            let file = std::path::PathBuf::from(project_file);
            if !file.exists() {
                return format!("❌ Project file not found: {}", file.display());
            }
            let src = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => return format!("❌ Failed to read {}: {}", file.display(), e),
            };

            // Find the closing brace of the target scene
            let scene_marker = format!("Scene(\"{}\"", scene_id);
            if let Some(scene_start) = src.find(&scene_marker) {
                // Find the matching closing brace
                let mut depth = 0;
                let mut insert_pos = None;
                for (i, ch) in src[scene_start..].char_indices() {
                    if ch == '{' {
                        depth += 1;
                    }
                    if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            insert_pos = Some(scene_start + i);
                            break;
                        }
                    }
                }

                if let Some(pos) = insert_pos {
                    let mut new_src = String::new();
                    new_src.push_str(&src[..pos]);
                    new_src.push_str(&layer_block);
                    new_src.push_str(&src[pos..]);
                    if let Err(e) = std::fs::write(&file, &new_src) {
                        return format!("❌ Failed to write {}: {}", file.display(), e);
                    }
                    format!(
                        "✅ Web layer '{}' added to scene '{}' in {}",
                        layer_id,
                        scene_id,
                        file.display()
                    )
                } else {
                    format!("❌ Could not find closing brace for scene '{}'", scene_id)
                }
            } else {
                format!("❌ Scene '{}' not found in {}", scene_id, file.display())
            }
        }
        "vidra-edit_web_scene" => {
            let layer_id = args.get("layer_id").and_then(|v| v.as_str()).unwrap_or("");
            if layer_id.is_empty() {
                return "❌ Missing required `layer_id` parameter".to_string();
            }
            let project_file = args
                .get("project_file")
                .and_then(|v| v.as_str())
                .unwrap_or("main.vidra");
            let file = std::path::PathBuf::from(project_file);

            // Build properties object to pass to the existing edit infrastructure
            let mut props = serde_json::Map::new();
            if let Some(s) = args.get("source") {
                props.insert("source".into(), s.clone());
            }
            if let Some(v) = args.get("viewport_width") {
                props.insert("viewport_width".into(), v.clone());
            }
            if let Some(v) = args.get("viewport_height") {
                props.insert("viewport_height".into(), v.clone());
            }
            if let Some(v) = args.get("mode") {
                props.insert("mode".into(), v.clone());
            }
            if let Some(v) = args.get("wait_for") {
                props.insert("wait_for".into(), v.clone());
            }
            if let Some(v) = args.get("variables") {
                props.insert("variables".into(), v.clone());
            }

            let props_val = Value::Object(props);
            match crate::mcp_tools::apply_layer_properties_to_vidra_file(
                &file, layer_id, &props_val,
            ) {
                Ok(true) => format!("✅ Web layer '{}' updated in {}", layer_id, file.display()),
                Ok(false) => format!(
                    "⚠️ Web layer '{}' not found or no changes applied in {}",
                    layer_id,
                    file.display()
                ),
                Err(e) => format!("❌ Failed to edit web layer '{}': {:#}", layer_id, e),
            }
        }
        "vidra-generate_web_code" => {
            let filename = args
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("scene.html");
            let html = args
                .get("html_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if html.is_empty() {
                return "❌ Missing required `html_content`".to_string();
            }

            let web_dir = std::path::PathBuf::from("web");
            if let Err(e) = std::fs::create_dir_all(&web_dir) {
                return format!("❌ Failed to create web/ directory: {}", e);
            }
            let out_path = web_dir.join(filename);
            match std::fs::write(&out_path, html) {
                Ok(()) => format!("✅ Generated web code at '{}'", out_path.display()),
                Err(e) => format!("❌ Failed to write '{}': {}", out_path.display(), e),
            }
        }
        _ => {
            format!("❌ Unknown tool '{}'", name)
        }
    }
}
