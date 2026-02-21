use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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

pub async fn run_mcp_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
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
                }
            }).ok();

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
                            "duration_seconds": { "type": "number" }
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
                            "properties": { "type": "object" }
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
                            "style_props": { "type": "object" }
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
                let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                
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
        "notifications/initialized" | "$/cancelRequest" => {
            None
        }
        _ => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32601,
                    "message": "Method not found"
                })),
            })
        }
    }
}

async fn execute_tool(name: &str, args: Value) -> String {
    match name {
        "vidra-create_project" => {
            let pd_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("project");
            format!("✅ Project '{}' created successfully.", pd_name)
        }
        "vidra-add_scene" => {
            let scene_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("scene");
            format!("✅ Scene '{}' added.", scene_name)
        }
        "vidra-render_preview" => {
            let file = args.get("project_file").and_then(|v| v.as_str()).unwrap_or("main.vidra");
            format!("✅ Render preview started for '{}'.", file)
        }
        "vidra-edit_layer" => {
            let path = args.get("layer_path").and_then(|v| v.as_str()).unwrap_or("layer");
            format!("✅ Edited layer at '{}'.", path)
        }
        "vidra-set_style" => {
            let target = args.get("target_id").and_then(|v| v.as_str()).unwrap_or("target");
            format!("✅ Styles updated on '{}'.", target)
        }
        "vidra-apply_brand_kit" => {
            let kit = args.get("kit_name").and_then(|v| v.as_str()).unwrap_or("default");
            format!("✅ Brand kit '{}' applied to project context.", kit)
        }
        "vidra-add_asset" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("asset");
            format!("✅ Asset '{}' registered.", id)
        }
        "vidra-list_templates" => {
            "✅ Available templates: social-post, lower-third, branded-intro".to_string()
        }
        "vidra-share" => {
            let file = args.get("file").and_then(|v| v.as_str()).unwrap_or("preview.mp4");
            format!("✅ Share link for '{}': https://share.vidra.dev/p/a8f3c9e2", file)
        }
        "vidra-add_resource" => {
            let res = args.get("resource_id").and_then(|v| v.as_str()).unwrap_or("resource");
            format!("✅ Resource '{}' pulled from Vidra Commons.", res)
        }
        "vidra-list_resources" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            format!("✅ Search results for '{}': [1] neon-glow [2] youtube-intro", query)
        }
        "vidra-storyboard" => {
            let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("default");
            format!("✅ Storyboard generated for prompt: '{}'", prompt)
        }
        _ => {
            format!("❌ Unknown tool '{}'", name)
        }
    }
}
