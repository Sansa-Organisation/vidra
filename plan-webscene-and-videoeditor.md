# Plan: WebScene Pipeline + Vidra Editor

Two features shipping together: a `web()` layer type that turns any browser-renderable content into a compositable video layer, and a `vidra editor` command that launches a full visual editing environment. Both build on top of the existing IR/render/MCP infrastructure.

## Architecture Summary

The `web()` layer adds a 14th variant to `LayerContent`. On the CLI/native side, a new `vidra-web` crate abstracts headless browser frame capture behind a `WebCaptureBackend` trait with Playwright (default) and Rust CDP backends. The web layer participates in the normal composite pipeline — its frames are `FrameBuffer` pixels like any other layer. A bidirectional bridge (`window.__vidra`) lets web code read timeline position, frame index, fps, and project variables, and emit values back via a console protocol. The editor is a new React app (`packages/vidra-editor`) served by an `editor` subcommand that extends the existing `axum` dev server with project CRUD, MCP relay, LLM proxy, and asset management endpoints.

---

## Feature 1: `web()` Layer — Web-to-Video Pipeline

### 1. IR: Add `Web` variant to `LayerContent`

- In `crates/vidra-ir/src/layer.rs` (L49), add a new variant:
  - `Web { source: String, viewport_width: u32, viewport_height: u32, mode: WebCaptureMode, wait_for: Option<String>, variables: HashMap<String, f64> }`
  - `WebCaptureMode` enum: `FrameAccurate`, `Realtime`
- Map to new `LayerType::Web` in `layer_type()` at `layer.rs` L176
- Add `Web` variant to `vidra_core::LayerType` in `crates/vidra-core/src/types.rs`

### 2. DSL: Add `web()` keyword to VidraScript

- Lexer: add `Web` to `TokenKind` enum and keyword match at `crates/vidra-lang/src/lexer.rs` L24 and L445
- AST: add `Web` variant to `LayerContentNode` at `crates/vidra-lang/src/ast.rs` L151
- Parser: add `web()` parsing in `parse_layer_content()` at `crates/vidra-lang/src/parser.rs` L680. Syntax: `web(source: "./dist", viewport: 1920x1080, mode: "frame-accurate", wait_for: ".loaded")`
- Compiler: add match arm in `compile_layer_content()` at `crates/vidra-lang/src/compiler.rs` L667. Resolve source path relative to project, register asset, construct `LayerContent::Web`
- Checker: add `web` to allowed content types in `crates/vidra-lang/src/checker.rs`
- Formatter: add `web()` formatting rule in `crates/vidra-lang/src/formatter.rs`

### 3. SDK: Add `.web()` method to Layer class

- In `packages/vidra-sdk/src/index.ts` (L98), add:
  - `web(source: string, opts?: { viewport?: [number, number], mode?: 'frame-accurate' | 'realtime', waitFor?: string, variables?: Record<string, number> }): Layer`
- In `types.ts`, add `Web` to `LayerContent` union type
- In `toVidraScript()` at L349, add VidraScript emit for `Web` variant

### 4. New crate: `vidra-web` — Web Capture Engine

- Create `crates/vidra-web/` with `Cargo.toml` depending on `vidra-core`, `serde`, `image`, `anyhow`
- Define `WebCaptureBackend` trait:
  ```rust
  trait WebCaptureBackend {
      fn start_session(source, viewport, mode) -> Session;
      fn capture_frame(session, frame_index, fps, variables) -> FrameBuffer;
      fn stop_session(session);
  }
  ```

#### 4a. Playwright backend (default)

- Shell out to `npx playwright` or a bundled Playwright script
- Create `crates/vidra-web/src/playwright.rs`
- Launch headless Chromium via Playwright's CDP connection
- For **frame-accurate mode**: inject a timing harness script via `page.addInitScript()` that:
  - Overrides `performance.now()`, `Date.now()`, `requestAnimationFrame` callback timing
  - Exposes `window.__vidra_advance_frame()` that steps the virtual clock forward by `1000/fps` ms
  - Exposes `window.__vidra = { frame, time, fps, vars, emit() }` bridge
- For **realtime mode**: just screenshot at `1000/fps` intervals, inject the bridge but don't override timing APIs
- `wait_for`: optional CSS selector — wait for element to appear before starting capture (useful for async-loaded apps)
- Communication: use Playwright's `page.evaluate()` to inject bridge and `page.screenshot({ type: 'png' })` per frame, return as RGBA pixels
- Playwright script lives in `crates/vidra-web/scripts/capture.js` (or `.ts`) — invoked via `node` subprocess

#### 4b. Rust CDP backend (optional, feature-gated)

- Add `chromiumoxide` as optional dependency behind `feature = "cdp"`
- Create `crates/vidra-web/src/cdp.rs` implementing same trait
- Uses `chromiumoxide::Browser::launch()` + CDP `Page.captureScreenshot`
- Same timing injection via `Page.addScriptToEvaluateOnNewDocument`

#### 4c. Frame capture pipeline integration

- Backend selection: check `VIDRA_WEB_BACKEND` env var, default to Playwright, fall back to CDP if playwright unavailable
- Frame caching: hash `(source_path_mtime, frame_index, variables_hash)` → cache RGBA in temp dir to avoid re-capturing unchanged frames during iterative editing

### 5. Render pipeline integration

- In `crates/vidra-render/src/pipeline.rs` (L706), add match arm for `LayerContent::Web`:
  - Create/reuse a `WebCaptureSession` (held on `RenderPipeline` alongside `VideoDecoder`)
  - Call `backend.capture_frame(session, frame_index, fps, variables)` → `FrameBuffer`
  - Scale to layer dimensions if viewport differs from layer size
  - Return as compositable `FrameBuffer` like any other layer
- Session lifecycle: start session on first web layer encountered, stop on render completion
- Parallel rendering: web capture is inherently sequential per session (one browser), so web layers get their own serial pass while native layers can still parallelize via rayon

### 6. WASM renderer handling

- In `crates/vidra-wasm/src/renderer.rs` (L483), the `Web` variant hits the existing catch-all `_` and renders as transparent
- In the player/editor (JS side), web layers get special treatment: the engine renders them in a separate `<iframe>` or `<div>` and composites via CSS `z-index` / `mix-blend-mode`, or captures via `html2canvas` for in-browser preview
- Add `renderWebLayerInBrowser(layerId, source, frame)` support to `packages/vidra-player/src/engine.ts` — creates a sandboxed iframe, injects the bridge, and composites its content onto the main canvas

### 7. Integrated mode: `@vidra/web-capture` npm package

- Create `packages/vidra-web-capture/` as a lightweight npm package
- Exports:
  - `useVidraScene(opts)` — React hook that registers the component tree as a capturable scene
  - `VidraCapture` — vanilla JS class for non-React usage
  - `window.__vidra` type definitions
- When running inside Vidra's capture harness (`window.__vidra.capturing === true`):
  - Components yield to frame-by-frame timing
  - `useVidraScene` returns `{ frame, time, fps, vars }` from the bridge
  - `useVidraScene` calls `window.__vidra.emit(key, value)` to send data back
- When running standalone (no harness): components run normally in real-time (graceful degradation)
- TypeScript types for the `window.__vidra` bridge object

### 8. MCP tool: `vidra-add_web_scene`

- Add new MCP tool in `crates/vidra-cli/src/mcp.rs` and `crates/vidra-cli/src/mcp_tools.rs`
- Accepts: `scene_id`, `source` (URL or path), `viewport`, `mode`, `duration`, `variables`
- Generates a `web()` layer in the `.vidra` file
- LLMs can also generate the HTML/React code and write it to the project's `web/` directory, then reference it as a web scene

---

## Feature 2: Vidra Editor

### 9. New package: `packages/vidra-editor`

- Create a fresh Vite + React + TypeScript app
- Dependencies: `@vidra/vidra-player` (WASM renderer), `@monaco-editor/react`, `@vidra/vidra-sdk`
- Project structure:
  ```
  packages/vidra-editor/
    package.json
    vite.config.ts
    tsconfig.json
    index.html
    src/
      main.tsx
      App.tsx
      components/
        Timeline/          — visual scene/layer timeline with drag handles and keyframes
        Canvas/            — live preview canvas (VidraEngine) with overlay controls
        SceneGraph/        — tree view of scenes/layers with drag-and-drop reorder
        PropertyInspector/ — context-sensitive property editor for selected layer
        CodeEditor/        — Monaco with VidraScript syntax + SDK code mode
        AIChat/            — conversational LLM panel with streaming responses
        AssetManager/      — browse, upload, preview assets (images, video, audio, fonts)
        WebPreview/        — sandboxed iframe for previewing web() layers
        McpConsole/        — direct MCP tool invocation panel for power users
        Toolbar/           — top bar with render/export, project settings, undo/redo
      hooks/
        useProject.ts      — project state management (load, save, undo/redo stack)
        useBackend.ts      — WebSocket connection to backend + REST calls
        useTimeline.ts     — playback state, current frame, selection
        useMcp.ts          — MCP tool invocation from frontend
        useAI.ts           — LLM integration (chat, code generation)
      stores/
        projectStore.ts    — zustand or similar state store for the open project
      api/
        backend.ts         — typed REST/WS client for the editor backend
      types/
        editor.ts          — editor-specific types
  ```

### 10. Editor backend: `vidra editor` CLI subcommand

- Add `Editor` variant to `Commands` enum in `crates/vidra-cli/src/main.rs` (L42):
  - Args: `file: Option<PathBuf>`, `--port` (default 3001), `--open` (auto-open browser)
- Create `crates/vidra-cli/src/editor_server.rs` extending the existing dev server pattern from `crates/vidra-cli/src/dev_server.rs`:
  - Reuse `DevState`, `compile_and_load()`, file watcher, WS fan-out
  - Serve the built editor frontend from embedded assets (via `include_dir` or `rust-embed`)
  - Add `open` crate dependency to auto-launch browser on startup

### 11. Editor backend API routes

- `GET /` — serve editor SPA (index.html + assets)
- `GET /ws` — WebSocket (extends existing dev server protocol):
  - All existing messages: `METADATA`, `ERROR`, `INSPECT_BOUNDS`, binary frames, `REQUEST_FRAME`, `REQUEST_BOUNDS`
  - New messages: `PROJECT_STATE` (full IR JSON), `PATCH_APPLIED` (diff), `UNDO`, `REDO`, `MCP_RESULT`
- `GET /api/project` — return current project IR as JSON
- `PUT /api/project` — write updated project back to `.vidra` file
- `POST /api/project/patch` — apply a targeted edit (uses existing `mcp_tools::apply_layer_properties_to_vidra_file`)
- `GET /api/project/source` — return raw VidraScript source
- `PUT /api/project/source` — overwrite VidraScript source (triggers recompile)
- `POST /api/render/frame` — render a single frame as JPEG/PNG (for timeline thumbnails)
- `POST /api/render/export` — trigger full render to file, stream progress via WS
- `POST /api/mcp/invoke` — invoke any MCP tool by name + params, return result
- `GET /api/assets` — list project assets
- `POST /api/assets/upload` — multipart upload, write to project `assets/` dir, register in IR
- `DELETE /api/assets/:id` — remove asset
- `POST /api/ai/chat` — LLM proxy endpoint:
  - Accepts: `messages[]`, `model`, `provider` (openai/gemini/anthropic)
  - System prompt includes VidraScript spec + SDK spec + current project context
  - Streams response via SSE or chunked response
  - Optionally auto-applies generated code to project

### 12. Editor frontend: core panels

- **Timeline**: renders scenes as colored blocks on a horizontal track, layers as stacked rows within each scene. Click to select, drag edges to resize duration, keyframe diamonds on animation tracks. Uses `<canvas>` for smooth rendering at scale.
- **Canvas**: `VidraEngine` rendering on a `<canvas>` with overlay controls: bounding boxes for selected layers, drag handles for position/scale, viewport zoom/pan. For `web()` layers, show a sandboxed `<iframe>` overlay positioned to match the layer's transform.
- **Scene Graph**: tree view with scene headers and layer children. Drag to reorder. Right-click context menu for add/delete/duplicate. Icons per layer type. Visibility toggle.
- **Property Inspector**: when a layer is selected, shows a form for its properties. Text inputs, color pickers, sliders for opacity/rotation/scale, animation curve editor. Changes emit `PATCH` messages to backend.
- **Code Editor**: Monaco with VidraScript syntax. Toggle between visual and code mode. Code changes compile and update the visual view; visual changes update the code. Conflicts are resolved in favor of the last editor used.
- **AI Chat**: message list, text input, model selector. Streaming responses. "Apply" button on generated code snippets. Context-aware: includes current project state in system prompt. Can generate VidraScript, SDK code, or web() scene HTML/React code.
- **Asset Manager**: grid of asset thumbnails. Drag-and-drop upload zone. Click to preview. Right-click to rename/delete. Shows asset type, file size, dimensions.

### 13. MCP integration in editor

- The editor's AI Chat uses MCP tools internally:
  - When the LLM suggests an edit, it can generate an MCP tool call (`vidra-edit_layer`, `vidra-add_scene`, etc.)
  - The editor frontend sends the tool call to `POST /api/mcp/invoke`
  - The backend executes it, updates the project file, broadcasts the change via WS
  - The frontend receives the update and refreshes timeline + preview
- The MCP Console panel exposes all 12+ tools directly for power users and debugging
- The `vidra-add_web_scene` tool (from Feature 1) is available in the editor for AI-generated web scenes

### 14. LLM integration

- System prompt generation in `packages/vidra-editor/src/hooks/useAI.ts`:
  - Include full VidraScript language spec (from `packages/vidra-examples/src/vidra-spec.ts`)
  - Include current project IR summary (scene names, layer IDs, asset list)
  - Include `web()` layer documentation and examples
  - Include available MCP tools as function definitions
- The AI can:
  - Generate new scenes from natural language descriptions
  - Modify existing layers by emitting MCP tool calls
  - Generate web() scene code (React/HTML/CSS) and save it to the project
  - Suggest animations, transitions, effects
  - Answer questions about VidraScript syntax

### 15. Build + embed pipeline

- `packages/vidra-editor` builds to `dist/` via Vite
- At CLI compile time, embed the dist folder using `rust-embed` or `include_dir!` macro into the `vidra-cli` binary
- The `editor` subcommand serves these embedded static files — zero runtime dependency on Node for the editor itself
- For development: `--dev` flag serves from the live Vite dev server instead of embedded assets

---

## Cross-Cutting Concerns

### 16. Documentation

- Add `web()` layer documentation to `docs/vidrascript.md`
- Add WebScene architecture section to `docs/architecture.md`
- Add `vidra editor` docs to `README.md`
- Update `docs/llm.md` and `docs/small-llm.md` with new capabilities
- Add a `docs/web-scenes.md` guide with examples for React, vanilla HTML, D3, Three.js

### 17. New MCP tools

- `vidra-add_web_scene`: create a web() layer from a source path/URL
- `vidra-edit_web_scene`: modify web scene properties (viewport, mode, variables)
- `vidra-generate_web_code`: LLM generates HTML/React code for a web scene from a prompt, saves to project

### 18. Example projects

- `examples/web_chart.vidra` — D3 chart as a web() layer composited with text overlays
- `examples/web_react.vidra` — React component with CSS animations
- `examples/web_interactive.vidra` — interactive Three.js scene captured frame-by-frame
- `packages/vidra-web-capture/examples/` — React app demonstrating integrated mode

---

## Verification

- Unit tests for `LayerContent::Web` serialization/deserialization roundtrip
- Unit tests for DSL parsing: `web(source: "./dist")` parses correctly
- Integration test: capture a simple HTML page (solid color div) and verify the output frame matches expected pixels
- Integration test: frame-accurate mode produces deterministic frames from a CSS animation
- Integration test: bidirectional variables — VidraScript `@var` changes web content, web `emit()` changes overlay text
- Editor: `vidra editor --port 0` starts successfully and serves the SPA
- Editor: WebSocket protocol responds to `REQUEST_FRAME`
- Editor: `POST /api/mcp/invoke` returns valid MCP results
- MCP purity: existing `mcp_stdio_purity` test still passes
- `npm run local:ci` passes with all new code

---

## Decisions

| Decision | Choice |
|----------|--------|
| Layer name | `web()` in DSL, `Web` in IR, `.web()` in SDK |
| Capture backend | Trait-abstracted, Playwright default, Rust CDP feature-gated |
| Timing | Both frame-accurate and realtime modes, switchable via `mode` param |
| Editor base | Fresh `packages/vidra-editor` package |
| Editor hosting | Served from embedded static assets via `vidra editor` CLI subcommand |
| WASM fallback | `Web` layers render as transparent in WASM; browser preview uses iframe compositing |
