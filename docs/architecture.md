# Vidra Architecture

Vidra is built as a highly modular Rust workspace. Its architecture reflects its primary goal: **turning video into queryable, deterministic software.**

## High-Level Pipeline

1. **Input Surface:** A user writes a `.vidra` text file, or a TypeScript SDK script generates a JSON AST, or an AI agent calls an MCP tool (`vidra.add_scene`).
2. **Parsing & Checking (`vidra-lang`):** The lexer, parser, and type-checker validate the syntax, resolve component scopes, ensure assets exist, and evaluate the AST.
3. **IR Compilation (`vidra-ir`):** The AST is flattened into the universal Vidra Intermediate Representation **(IR)**. This is a JSON-serializable, fully resolved scene graph.
4. **GPU Compositing (`vidra-render`):** The IR is ingested by the real-time wgpu-based renderer. Text is rasterized, assets are decoded via FFmpeg/image, and WebGPU compute shaders execute the effects, blending, and animations per-frame.
5. **Encoding (`vidra-encode`):** The raw frame buffers (RGBA) are piped into FFmpeg to be muxed into an `.mp4`, `.webm`, `.mov`, etc.

## The Crates

*   `vidra-core`: Foundational types — Color, Vector2, Span (for errors), Types, Duration, Config.
*   `vidra-lang`: The frontend of the compiler — lexer, parser, AST, type checker, semantic highlighter, formatter, and IR builder.
*   `vidra-ir`: The **Vidra IR**, defining `Project`, `Scene`, `Layer`, `LayerContent`, `Animation`, `AssetId`, and `CRDT`.
*   `vidra-render`: The GPU backend. Contains the `Compositor`, WebGPU pipelines (`compistor.wgsl`, `effects.wgsl`), `TextServer`, and image caching.
*   `vidra-encode`: The FFmpeg wrapper mapping frames to video files and writing audio streams.
*   `vidra-cli`: The application entry point. Handles `render`, `dev`, `storyboard`, `mcp`, authentication (`vlt.token`), project scaffolding, syncing, and metrics.
*   `vidra-lsp`: The Language Server Protocol implementation, providing syntax highlighting, autocomplete, and diagnostics to extensions like VS Code.

## The Vidra IR (Intermediate Representation)

The core architecture concept you must understand is **the IR**. 
VidraScript is just syntactic sugar. Ultimately, *everything* in Vidra compiles to the `vidra-ir` spec: a flat array of scenes containing stacked, animatable layers.

Why?
1. **Deterministic:** A byte-identical IR produces byte-identical MP4 output. 
2. **Serialization:** The IR is JSON. We can store versions, git diff them, or stream them over WebSockets.
3. **Semantic Addressing:** AI agents and clients edit video by sending JSON Patches like: `{"op": "replace", "path": "/scenes/0/layers/1/content/text", "value": "A new intro!"}` rather than pixel coordinates.
4. **CRDT Multiplayer:** Multiple users edit the same IR document without conflict.

## Live Preview & Hot Reloading

When running `vidra dev`, the CLI spins up an HTTP server. It dynamically recompiles the AST to IR on file save, diffs the new IR against the running state, uploads new textures to the GPU, and streams back single frames over WebSockets. The compilation takes ~10-20ms, meaning "Hot Module Replacement" for video is functionally real-time.

## Cloud & Web Workers

The wgpu `vidra-render` pipeline is specifically configured to target WebGL2 / WebGPU. The engine compiles into WASM (Phase 3). 
When passing `--cloud`, the `vidra-cli` zips the IR + `.vidra` file and queues a job in Vidra Cloud, which provisions an ephemeral worker (running a containerized headless `vidra-render` instance on NVIDIA T4s), renders it, provisions the shareable URL, and returns the result to the user.

## Web Scenes Architecture

Web Scenes extend Vidra's rendering pipeline to support HTML/CSS/JS content as composited layers. The architecture adds a parallel rendering path:

```
                        ┌─── Native Layers ──→ GPU Compositor ──┐
 Vidra IR (JSON) ──────┤                                        ├──→ Frame Buffer
                        └─── Web Layers ───→ Capture Engine ────┘
```

**Key components:**

- **`LayerContent::Web`** (vidra-ir): Extends the IR with `source`, `viewport`, `mode`, `wait_for`, and `variables` fields.
- **Capture Engine** (vidra-render): Uses Chrome DevTools Protocol (CDP) to load web pages, advance frames via `__vidra_advance_frame`, and rasterize content.
- **Browser Player** (vidra-player): Renders web layers as sandboxed `<iframe>` overlays positioned via CSS transforms from WASM-computed bounding boxes.
- **`@sansavision/vidra-web-capture`** (npm): Framework-agnostic SDK for web content to communicate with the capture harness. Provides `VidraCapture` (vanilla) and `useVidraScene` (React) APIs.

**Rendering modes:**

| Mode | Context | Mechanism | Deterministic? |
|---|---|---|---|
| Capture | `vidra render` | Headless browser frame-by-frame | ✓ |
| Realtime | `vidra dev` / `vidra editor` | iframe overlay | ✗ |

See [docs/web-scenes.md](web-scenes.md) for full integration details and examples.

## Visual Editor Architecture

The `vidra editor` command launches a local server + web-based editing environment:

```
┌─────────────────────────────────────────────────────────┐
│                   vidra editor CLI                       │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  File Watcher │  │  Compiler    │  │  GPU Renderer│  │
│  │  (notify)     │→│  (vidra-lang) │→│  (vidra-render)│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         │                                      │        │
│         ▼                                      ▼        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                   HTTP + WS Server               │  │
│  │  GET  /              → Embedded frontend         │  │
│  │  GET  /api/project   → IR JSON                   │  │
│  │  PUT  /api/project/source → Write + recompile    │  │
│  │  POST /api/project/patch  → Edit layer props     │  │
│  │  POST /api/render/frame   → Single frame JPEG    │  │
│  │  POST /api/mcp/invoke     → MCP tool relay       │  │
│  │  POST /api/ai/chat        → LLM proxy            │  │
│  │  WS   /ws                 → Real-time updates    │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│               Editor Frontend (React + Vite)            │
│                                                         │
│  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌──────────────┐ │
│  │ Scene   │ │ Canvas   │ │Timeline│ │  Property    │ │
│  │ Graph   │ │ Preview  │ │        │ │  Inspector   │ │
│  └─────────┘ └──────────┘ └────────┘ └──────────────┘ │
│                           │                             │
│  ┌─────────┐ ┌──────────┐                              │
│  │ Code    │ │ Toolbar  │                              │
│  │ Editor  │ │ Undo/Redo│                              │
│  └─────────┘ └──────────┘                              │
└─────────────────────────────────────────────────────────┘
```

**Key design decisions:**

1. **Server-side rendering**: The GPU pipeline runs in the Rust backend, not in the browser. The frontend only receives JPEG frames over WebSocket.
2. **Source-of-truth on disk**: `PUT /api/project/source` writes directly to the `.vidra` file. The file watcher triggers recompilation automatically.
3. **MCP relay**: The editor can invoke any MCP tool (`vidra-add_scene`, `vidra-add_web_scene`, etc.) through `POST /api/mcp/invoke`, enabling AI-assisted editing.
4. **No Node.js runtime**: The editor frontend is embedded in the Rust binary — users need only `vidra editor` to start editing.

