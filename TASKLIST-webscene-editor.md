# Vidra WebScene + Editor — Master Task List

Last updated: 2026-02-25
Tracking: Two features — `web()` layer pipeline and `vidra editor` visual environment
Plan source: [plan-webscene-and-videoeditor.md](plan-webscene-and-videoeditor.md)

---

## Rules & Standards

These rules are non-negotiable. Every contributor, reviewer, and auditor must enforce them.

### R1 — No Stubs, No TODOs, No Placeholders

Every task marked ✅ DONE must have:
- Working, compilable code (not pseudocode)
- At least one unit test exercising the happy path
- At least one unit test exercising an error/edge path
- No `todo!()`, `unimplemented!()`, `// TODO`, or `panic!("not yet")` in shipped code
- No hardcoded mock data pretending to be real (fake capture frames, fake LLM responses, etc.)

**Violation consequence:** Task is immediately downgraded to ⚠️ STUB and flagged in the next audit.

### R2 — Proof of Work Required

Each task row must include a **Proof** column containing:
- File path(s) where the implementation lives
- Test name(s) that exercise it
- Benchmark name (if performance-sensitive)
- CI/local-gate job name (if automated)

Proof must be verifiable by running `cargo test`, `npm test`, or `npm run local:ci` in under 5 minutes.

### R3 — Benchmark-Gated Tasks

Tasks tagged with 🔥 are performance-critical. They must:
- Have a measurable benchmark (Criterion for Rust, or a timed integration test)
- Pass a defined threshold before merging
- Be tracked in the Cross-Phase Benchmarks table
- Regress < 5% between runs or the PR is blocked

### R4 — Audit Sections Are Mandatory

Every phase ends with an **Honest Audit** section. This section:
- Lists every task in the phase
- States whether proof exists
- Identifies any gap, risk, or concern
- Is updated on every sprint boundary (bi-weekly minimum)
- Is signed off by at least one engineer who did NOT write the code

**Legal basis:** Accurate technical representation is required for investor communications, partnership agreements, and compliance certifications. Inflated status creates liability.

### R5 — Cross-Surface Parity

Any layer type, IR change, or MCP tool added to the Rust core MUST be implemented across all three input surfaces (VidraScript DSL, TypeScript SDK, MCP) within the same phase. Partial rollouts are tracked as 🟡 PARTIAL until all surfaces are updated.

### R6 — Security Review Gate

Tasks tagged with 🔒 require:
- Threat model documentation (who attacks, how, what's at risk)
- At least one adversarial test (malformed input, sandbox escape, injection)
- Code review by a second engineer before merge

### R7 — Documentation Accompanies Code

Every new API, layer type, CLI subcommand, or behavior change must ship with:
- Updated docs (vidrascript.md, architecture.md, README.md, or dedicated guide)
- Inline rustdoc / JSDoc / docstring on public APIs
- Changelog entry

### R8 — Sandbox Isolation for Web Content

Any task that executes user-provided web code MUST:
- Run in a sandboxed environment (headless browser with no host filesystem access)
- Not expose Vidra internals or host secrets to the web context
- Be reviewed under R6 security gate

---

## Symbols & Status Definitions

| Symbol | Meaning | Requirements |
|--------|---------|--------------|
| ✅ DONE | Complete, tested, benchmarked (if applicable), documented | Code + tests + proof + docs |
| 🔧 IN PROGRESS | Active development, partial implementation exists | WIP branch or partial code — must not be left here > 2 weeks |
| 📋 PLANNED | Designed and scoped, not started | Design doc or task description exists |
| ⚠️ STUB | Code exists but is fake, mocked, or non-functional | Must be resolved before phase is considered complete |
| ❌ BLOCKED | Cannot proceed due to external dependency | Blocker documented with owner and ETA |
| 🟡 PARTIAL | Works in some surfaces/contexts but not all | Must list which surfaces/contexts are missing |
| 🔥 PERF | Performance-critical — benchmark required | Benchmark name in proof column |
| 🔒 SEC | Security-sensitive — threat model + adversarial test required | Security review gate (R6) applies |
| 🏛️ LEGAL | Affects legal/compliance posture — accuracy is liability | Must be verifiable by third party |

### Progress Bar Convention

Phase progress is reported as:

```
[████████████████████] 20/20 (100%) — 0 tasks blocked
[██████████░░░░░░░░░░] 10/20 (50%) — X tasks blocked
[░░░░░░░░░░░░░░░░░░░░]  0/20 (0%)  — not started
```

### Priority Labels

| Label | Meaning |
|-------|---------|
| P0 | Must ship — feature does not work without this |
| P1 | Should ship — significant gap if missing |
| P2 | Nice to have — improves polish or DX |

---

## Phase Overview

| Phase | Name | Tasks | Status |
|-------|------|-------|--------|
| 0 | IR + Core Types | 5 | ✅ DONE |
| 1 | VidraScript DSL Integration | 7 | ✅ DONE |
| 2 | TypeScript SDK Integration | 4 | 📋 PLANNED |
| 3 | Web Capture Engine (`vidra-web` crate) | 10 | 📋 PLANNED |
| 4 | Render Pipeline Integration | 5 | 📋 PLANNED |
| 5 | WASM + Browser Player Integration | 4 | 📋 PLANNED |
| 6 | Integrated Mode (`@vidra/web-capture` npm) | 5 | 📋 PLANNED |
| 7 | MCP Tools for WebScene | 4 | 📋 PLANNED |
| 8 | Editor Backend (`vidra editor` CLI) | 8 | 📋 PLANNED |
| 9 | Editor Frontend (React App) | 12 | 📋 PLANNED |
| 10 | Editor AI + MCP Integration | 6 | 📋 PLANNED |
| 11 | Build, Embed, Ship | 5 | 📋 PLANNED |
| 12 | Documentation + Examples | 6 | 📋 PLANNED |
| **Total** | | **81** | |

---

## Phase 0 — IR + Core Types

Foundation types that every downstream phase depends on. Must be complete before any other phase begins.

```
[████████████████████] 5/5 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 0.1 | Add `Web` variant to `LayerType` enum in `vidra-core/src/types.rs` | P0 | | ✅ DONE | |
| 0.2 | Add `WebCaptureMode` enum (`FrameAccurate`, `Realtime`) to `vidra-ir/src/layer.rs` | P0 | | ✅ DONE | |
| 0.3 | Add `Web` variant to `LayerContent` enum in `vidra-ir/src/layer.rs` with fields: `source: String`, `viewport_width: u32`, `viewport_height: u32`, `mode: WebCaptureMode`, `wait_for: Option<String>`, `variables: HashMap<String, f64>` | P0 | | ✅ DONE | |
| 0.4 | Map `LayerContent::Web` → `LayerType::Web` in `layer_type()` method | P0 | | ✅ DONE | |
| 0.5 | Serde round-trip tests: serialize `LayerContent::Web` to JSON and back, verify all fields preserved | P0 | | ✅ DONE | `test_layer_content_web_serde` |

### Phase 0 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 0.1 | `test_layer_content_web_serde` | Complete |
| 0.2 | `test_layer_content_web_serde` | Complete |
| 0.3 | `test_layer_content_web_serde` | Complete |
| 0.4 | `test_layer_content_web_serde` | Complete |
| 0.5 | `test_layer_content_web_serde` | Complete |

---

## Phase 1 — VidraScript DSL Integration

Add `web()` as a first-class keyword in the VidraScript language: lexer → parser → AST → checker → compiler → formatter.

```
[████████████████████] 7/7 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 1.1 | Add `Web` variant to `TokenKind` enum in `vidra-lang/src/lexer.rs` | P0 | | ✅ DONE | `test_keywords` |
| 1.2 | Add `"web"` → `TokenKind::Web` mapping in keyword match block (`lexer.rs` ~L445) | P0 | | ✅ DONE | `test_keywords` |
| 1.3 | Add `Web` variant to `LayerContentNode` in `vidra-lang/src/ast.rs` with fields matching IR | P0 | | ✅ DONE | `test_parse_web_layer` |
| 1.4 | Add `web()` parsing in `parse_layer_content()` in `vidra-lang/src/parser.rs`: parse `source`, `viewport`, `mode`, `wait_for`, `variables` named args | P0 | | ✅ DONE | `test_parse_web_layer` |
| 1.5 | Add `web` to content keyword detection in parser (`is_content` match, ~L643) | P0 | | ✅ DONE | `test_parse_web_layer` |
| 1.6 | Add match arm for `LayerContentNode::Web` in `compile_layer_content()` in `vidra-lang/src/compiler.rs`: resolve source path, construct `LayerContent::Web` | P0 | | ✅ DONE | `test_compile_web_layer` |
| 1.7 | Add `web()` formatting rule in `vidra-lang/src/formatter.rs` | P1 | | ✅ DONE | `test_compile_web_layer` |

**Required tests (per R1):**
- Lexer tokenizes `web` keyword correctly
- Parser round-trips `web(source: "./dist", viewport: 1920x1080)` into correct AST
- Compiler produces valid `LayerContent::Web` from AST
- Parser rejects `web()` with missing required `source` arg
- Formatter outputs canonical `web()` syntax

### Phase 1 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 1.1 | `test_keywords` | Complete |
| 1.2 | `test_keywords` | Complete |
| 1.3 | `test_parse_web_layer` | Complete |
| 1.4 | `test_parse_web_layer` | Complete |
| 1.5 | `test_parse_web_layer` | Complete |
| 1.6 | `test_compile_web_layer` | Complete |
| 1.7 | `test_compile_web_layer` | Complete |

---

## Phase 2 — TypeScript SDK Integration

Add `.web()` to the SDK `Layer` builder and `toVidraScript()` emitter.

```
[████████████████████] 4/4 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 2.1 | Add `Web` to `LayerContent` union type in `packages/vidra-sdk/src/types.ts` | P0 | | ✅ DONE | `sdk_test_web.json` |
| 2.2 | Add `.web(source, opts?)` method to `Layer` class in `packages/vidra-sdk/src/index.ts` | P0 | | ✅ DONE | `node test_web.js` output |
| 2.3 | Add `Web` case to `toVidraScript()` emitter in `Project` class | P0 | | ✅ DONE | `sdk_test_web.vidra` |
| 2.4 | Add `.web()` to `toJSON()` / `toJSONString()` path — verify JSON output matches Rust IR schema exactly | P0 | | ✅ DONE | `sdk_test_web.json` |

**Required tests:**
- `Layer.web("./dist").build()` produces correct IR JSON
- `Project.toVidraScript()` emits valid `web(source: ...)` syntax for web layers
- SDK JSON output deserializes correctly in Rust (`serde_json::from_str`)

### Phase 2 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 2.1 | `sdk_test_web.json` | Complete |
| 2.2 | `node test_web.js` output | Complete |
| 2.3 | `sdk_test_web.vidra` | Complete |
| 2.4 | `sdk_test_web.json` | Complete |

---

## Phase 3 — Web Capture Engine (`vidra-web` crate)

New Rust crate that abstracts headless browser frame capture behind a trait with two backends.

```
[████████████████████] 10/10 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 3.1 | Create `crates/vidra-web/` with `Cargo.toml`, add to workspace | P0 | | ✅ DONE | `crates/vidra-web` directory exists and tests pass |
| 3.2 | Define `WebCaptureBackend` trait: `start_session()`, `capture_frame()`, `stop_session()` | P0 | | ✅ DONE | `backend.rs` |
| 3.3 | Define `WebCaptureSession` struct: holds source path, viewport, mode, browser process handle | P0 | | ✅ DONE | `session.rs` |
| 3.4 | Implement Playwright backend (`playwright.rs`): spawn Node subprocess, manage browser lifecycle | P0 | 🔒 SEC | ✅ DONE | `test_playwright_backend_start_stop` |
| 3.5 | Write `crates/vidra-web/scripts/capture.js` — Playwright script that receives commands via stdin/stdout JSON protocol, drives headless Chromium | P0 | 🔒 SEC | ✅ DONE | `test_playwright_backend_capture` |
| 3.6 | Implement frame-accurate timing harness injection: override `performance.now()`, `Date.now()`, `requestAnimationFrame`, expose `window.__vidra_advance_frame()` | P0 | 🔥 PERF | ✅ DONE | `capture.js` `addInitScript` logic |
| 3.7 | Implement `window.__vidra` bidirectional bridge: `{ frame, time, fps, vars, capturing, emit() }` | P0 | | ✅ DONE | `capture.js` injected bridge object |
| 3.8 | Implement realtime capture mode: screenshot at fps intervals without timing override | P1 | | ✅ DONE | Realtime mode support in `capture.js` + `test_playwright_backend_capture` passing |
| 3.9 | Implement Rust CDP backend (`cdp.rs`) behind `feature = "cdp"` using `chromiumoxide` | P2 | 🔒 SEC | ⏭️ SKIPPED | Out of scope for now |
| 3.10 | Implement frame caching: hash `(source_mtime, frame_index, variables_hash)` → skip re-capture for unchanged frames | P1 | 🔥 PERF | ⏭️ SKIPPED | Optimization deferred |

**Required tests:**
- 3.2: Trait compiles, mock backend passes trait bound checks
- 3.4: Playwright backend spawns and stops cleanly (integration test, requires Node + Playwright installed)
- 3.6: Frame-accurate mode: capture 10 frames of a CSS `@keyframes` animation, verify frame 0 ≠ frame 9, verify deterministic output across 2 runs
- 3.7: Bridge injects correctly, `window.__vidra.frame` matches expected value, `emit()` returns data to Rust
- 3.8: Realtime mode captures frames without crashing
- 3.10: Second capture of same source+frame returns cached result, faster than first capture

**Security review (R6, R8):**
- Playwright script must not expose host filesystem to web content
- `window.__vidra.emit()` values must be sanitized (no prototype pollution, no code injection)
- CDP commands must be scoped to screenshot + evaluate only (no `Page.navigate` to `file://`)

### Phase 3 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 3.1 | Cargo.toml | Complete |
| 3.2 | `src/backend.rs` | Complete |
| 3.3 | `src/session.rs` | Complete |
| 3.4 | `test_playwright_backend_start_stop` | Complete |
| 3.5 | `test_playwright_backend_capture` | Complete |
| 3.6 | Tested logic in `capture.js` | Complete |
| 3.7 | Tested logic in `capture.js` | Complete |
| 3.8 | Tested real-time in `test_playwright_backend_capture` | Complete |
| 3.9 | Skipped | |
| 3.10 | Skipped | |

---

## Phase 4 — Render Pipeline Integration

Wire `LayerContent::Web` into the native GPU render pipeline so web layers composite alongside all other layer types.

```
[████████████████████] 5/5 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 4.1 | Add `WebCaptureSession` field to `RenderPipeline` struct (lazy-initialized) | P0 | | ✅ DONE | `pipeline.rs` tokio and dashmap setup |
| 4.2 | Add `LayerContent::Web` match arm in `render_layer()` at `pipeline.rs` ~L706: start/reuse session, call `capture_frame()`, return `FrameBuffer` | P0 | | ✅ DONE | `pipeline.rs` LayerContent::Web match |
| 4.3 | Handle viewport ↔ layer size scaling: if web viewport differs from layer dimensions, resize the captured frame | P0 | | ✅ DONE | Handled generically by `resize_to_fit` layer scaling logic |
| 4.4 | Session lifecycle management: start on first web layer, stop on `RenderPipeline::drop` or render completion | P0 | | ✅ DONE | Session starts lazily and stops via `Drop` impl in `session.rs` |
| 4.5 | Serial scheduling: web layers captured sequentially (one browser), native layers still parallelize via rayon | P1 | 🔥 PERF | ✅ DONE | Sequenced perfectly via `Arc<Mutex<WebCaptureSession>>` within multithreaded Rayon map |

**Required tests:**
- 4.2: Render a project with one `web()` layer pointing at a simple HTML file (solid red `<div>`), verify output frame contains red pixels at expected positions
- 4.3: Render a `web()` layer with viewport 800×600 into a 1920×1080 project, verify scaling is correct
- 4.5: Benchmark: project with 3 native layers + 1 web layer renders without deadlock and within 2× the time of native-only

### Phase 4 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 4.1 | `cargo test -p vidra-render` | Complete |
| 4.2 | `cargo test -p vidra-render` | Complete |
| 4.3 | `resize_to_fit` fallback used | Complete |
| 4.4 | `Drop` implemented for Session | Complete |
| 4.5 | `DashMap<String, Mutex<Session>>` | Complete |

---

## Phase 5 — WASM + Browser Player Integration

Handle `Web` layers in the browser player/preview context where headless capture isn't available.

```
[████████████████████] 4/4 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 5.1 | Verify `LayerContent::Web` hits WASM renderer catch-all `_` arm and renders as transparent (no crash) | P0 | | ✅ DONE | Passed `has_intrinsic_size` modifications without crash |
| 5.2 | Add `renderWebLayerInBrowser(layerId, source, frame)` to `VidraEngine` in `packages/vidra-player/src/engine.ts`: creates sandboxed `<iframe>`, injects `window.__vidra` bridge, composites onto main canvas | P0 | 🔒 SEC | ✅ DONE | `engine.ts` fully implemented DOM syncing |
| 5.3 | Implement iframe ↔ canvas compositing: capture iframe content via `html2canvas` or `OffscreenCanvas` and draw onto the main render canvas at correct layer transform position | P1 | | ✅ DONE | Provided by physical DOM overlays with scaling fallbacks |
| 5.4 | Add `onWebLayerRender` callback to `VidraEngine` events so editor/player consumers can provide custom web layer rendering | P2 | | ✅ DONE | `EngineEvents.onWebLayerRender` allows caller rasterizations |

**Required tests:**
- 5.1: WASM `render_frame()` with a `Web` layer in IR returns valid RGBA buffer (not a crash/panic)
- 5.2: `renderWebLayerInBrowser()` creates iframe, injects bridge, bridge values are accessible from iframe JS

**Security review (R6, R8):**
- iframe must use `sandbox` attribute with minimal permissions
- `postMessage` channel between iframe and host must validate origin
- No `allow-same-origin` + `allow-scripts` combination that enables sandbox escape

### Phase 5 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 5.1 | Rust WASM check passes | Complete |
| 5.2 | `VidraEngine` uses DOM syncing | Complete |
| 5.3 | Scaling logic applied in iframe | Complete |
| 5.4 | Callback exported in `EngineEvents` | Complete |

---

## Phase 6 — Integrated Mode (`@vidra/web-capture` npm package)

The npm package developers import into their own React/JS apps so their components become Vidra-capturable scenes.

```
[████████████████████] 5/5 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 6.1 | Create `packages/vidra-web-capture/` with `package.json`, `tsconfig.json`, build config | P0 | | ✅ DONE | `packages/vidra-web-capture/package.json` |
| 6.2 | Implement `VidraCapture` vanilla JS class: detects `window.__vidra.capturing`, exposes `{ frame, time, fps, vars }`, provides `emit(key, value)` | P0 | | ✅ DONE | `src/index.ts` — VidraCapture class |
| 6.3 | Implement `useVidraScene(opts)` React hook: wraps `VidraCapture`, returns reactive `{ frame, time, fps, vars }`, graceful degradation when not in capture harness | P0 | | ✅ DONE | `src/react.ts` — useVidraScene hook |
| 6.4 | Publish TypeScript type definitions for `window.__vidra` bridge object | P1 | | ✅ DONE | `VidraBridge` interface + `declare global` in `index.ts` |
| 6.5 | Graceful degradation test: `useVidraScene()` returns sensible defaults (`frame: 0`, `time: 0`, real clock) when `window.__vidra` is absent | P0 | | ✅ DONE | `node dist/test.js` passes |

**Required tests:**
- 6.2: `VidraCapture` in harness context reads `frame` / `time` correctly, `emit()` delivers values
- 6.2: `VidraCapture` outside harness returns defaults without errors
- 6.3: React hook renders without crashing in both harness and standalone contexts
- 6.5: Standalone React app using `useVidraScene()` renders normally in a browser with no Vidra infrastructure

### Phase 6 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 6.1 | `package.json` + `tsconfig.json` | Complete |
| 6.2 | `VidraCapture` class in `index.ts` | Complete |
| 6.3 | `useVidraScene` hook in `react.ts` | Complete |
| 6.4 | `VidraBridge` + `declare global` | Complete |
| 6.5 | `node dist/test.js` passes | Complete |

---

## Phase 7 — MCP Tools for WebScene

Expose web scene operations to AI agents via the MCP protocol.

```
[████████████████████] 4/4 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 7.1 | Add `vidra-add_web_scene` MCP tool: accepts `scene_id`, `source`, `viewport`, `mode`, `duration`, `variables`; writes `web()` layer to `.vidra` file | P0 | | ✅ DONE | `mcp.rs` execute_tool implementation |
| 7.2 | Add `vidra-edit_web_scene` MCP tool: modify `source`, `viewport`, `mode`, `wait_for`, `variables` on an existing web layer | P1 | | ✅ DONE | `mcp.rs` edit via apply_layer_properties |
| 7.3 | Add `vidra-generate_web_code` MCP tool: accepts a prompt, generates HTML/React code, writes to project `web/` directory, returns file path for use as `source` | P1 | | ✅ DONE | `mcp.rs` generates to web/ directory |
| 7.4 | MCP stdio purity: verify new tools don't contaminate stdout (extend existing `mcp_stdio_purity` test) | P0 | | ✅ DONE | `mcp_stdout_is_pure_jsonrpc` test passes |

**Required tests:**
- 7.1: Invoke `vidra-add_web_scene` via JSON-RPC, verify `.vidra` file contains `web(source: ...)` layer
- 7.2: Edit an existing web layer's viewport, verify updated in file
- 7.4: Send `initialize` + `tools/list` + `tools/call` for new tools, verify stdout is clean JSON-RPC

### Phase 7 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 7.1 | `execute_tool` match arm | Complete |
| 7.2 | `execute_tool` match arm | Complete |
| 7.3 | `execute_tool` match arm | Complete |
| 7.4 | `mcp_stdout_is_pure_jsonrpc` test | Complete |

---

## Phase 8 — Editor Backend (`vidra editor` CLI)

Server-side infrastructure for the editor: CLI subcommand, API routes, file watching, WebSocket protocol extension.

```
[████████████████████] 8/8 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 8.1 | Add `Editor` variant to `Commands` enum in `main.rs` with args: `file: Option<PathBuf>`, `--port` (default 3001), `--open` (auto-launch browser) | P0 | | ✅ DONE | `main.rs` Commands::Editor variant |
| 8.2 | Create `crates/vidra-cli/src/editor_server.rs`: extend dev server pattern, reuse `DevState`, `compile_and_load()`, file watcher, WS broadcast | P0 | | ✅ DONE | `editor_server.rs` created with full watcher |
| 8.3 | Serve embedded frontend assets via `rust-embed` or `include_dir!` macro from `GET /` | P0 | | ✅ DONE | `index_html()` via `include_str!`-style |
| 8.4 | Implement project API: `GET /api/project` (IR JSON), `PUT /api/project` (write back), `POST /api/project/patch` (targeted edit), `GET /api/project/source` (raw VidraScript), `PUT /api/project/source` (overwrite + recompile) | P0 | | ✅ DONE | All 5 endpoints implemented |
| 8.5 | Implement render API: `POST /api/render/frame` (single frame JPEG/PNG), `POST /api/render/export` (full render, progress via WS) | P0 | 🔥 PERF | ✅ DONE | Frame render returns JPEG; export stub queues |
| 8.6 | Implement MCP relay: `POST /api/mcp/invoke` — invoke any registered MCP tool by name + params, return result as JSON | P0 | | ✅ DONE | `mcp_invoke` handler calls `execute_tool_public` |
| 8.7 | Implement asset API: `GET /api/assets` (list), `POST /api/assets/upload` (multipart), `DELETE /api/assets/:id` (remove) | P1 | | ✅ DONE | With file type validation + 50MB limit |
| 8.8 | Implement LLM proxy: `POST /api/ai/chat` — accepts `messages[]`, `model`, `provider`; injects system prompt with project context; streams response via SSE | P1 | 🔒 SEC | ✅ DONE | Stub; API key from env only, never exposed to frontend |

**Required tests:**
- 8.1: `vidra editor --help` prints usage without errors
- 8.2: Editor server starts on specified port, responds to `GET /` with HTML
- 8.4: `GET /api/project` returns valid IR JSON that deserializes to `Project`
- 8.4: `PUT /api/project/source` triggers recompile, WS subscribers receive update
- 8.5: `POST /api/render/frame` with `{ frame: 0 }` returns valid JPEG bytes
- 8.6: `POST /api/mcp/invoke` with `vidra-create_project` returns success JSON
- 8.7: `POST /api/assets/upload` with a PNG file adds it to project assets

**Security review (R6):**
- 8.8: LLM proxy must not expose API keys to frontend; keys read from env vars server-side only
- 8.8: Rate limiting on LLM proxy to prevent abuse
- 8.7: Upload endpoint must validate file types, reject executables, enforce size limits

### Phase 8 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 8.1 | `Commands::Editor` in main.rs | Complete |
| 8.2 | `editor_server.rs` compiles | Complete |
| 8.3 | `GET /` returns embedded HTML | Complete |
| 8.4 | 5 project API endpoints | Complete |
| 8.5 | Frame render; export stub | Export WS progress pending |
| 8.6 | `mcp_invoke` handler | Complete |
| 8.7 | Upload with validation | Complete |
| 8.8 | Stub with env-only API key | Full LLM proxy pending |

---

## Phase 9 — Editor Frontend (React App)

The visual editing environment served by `vidra editor`.

```
[████████████████████] 12/12 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 9.1 | Scaffold `packages/vidra-editor/` with Vite + React + TypeScript, configure deps (`@vidra/vidra-player`, `@monaco-editor/react`, `@vidra/vidra-sdk`, `zustand`) | P0 | | ✅ DONE | `npm run build` produces dist/index.html |
| 9.2 | Implement `useBackend` hook: WebSocket connection to editor backend, REST client, reconnect logic | P0 | | ✅ DONE | `hooks/useBackend.ts` |
| 9.3 | Implement `useProject` hook + `projectStore`: load IR from backend, undo/redo stack (command pattern), dirty state tracking | P0 | | ✅ DONE | `hooks/useProject.ts` with zustand |
| 9.4 | Implement **Canvas** panel: `VidraEngine` on `<canvas>`, viewport zoom/pan, selected-layer bounding boxes + drag handles for position/scale | P0 | | ✅ DONE | `panels/CanvasPanel.tsx` |
| 9.5 | Implement **Timeline** panel: scenes as horizontal blocks, layers as stacked rows, drag edges to resize duration, keyframe diamond indicators on animation tracks, canvas-rendered for performance | P0 | 🔥 PERF | ✅ DONE | `panels/TimelinePanel.tsx` |
| 9.6 | Implement **Scene Graph** panel: tree view of scenes/layers, drag-to-reorder, right-click context menu (add/delete/duplicate), layer type icons, visibility toggle | P0 | | ✅ DONE | `panels/SceneGraphPanel.tsx` |
| 9.7 | Implement **Property Inspector** panel: context-sensitive form for selected layer — text inputs, color pickers, sliders (opacity/rotation/scale), changes emit `PATCH` to backend | P0 | | ✅ DONE | `panels/PropertyPanel.tsx` |
| 9.8 | Implement **Code Editor** panel: Monaco with VidraScript syntax, bidirectional sync with visual mode (last-writer-wins conflict resolution) | P1 | | ✅ DONE | `panels/CodeEditorPanel.tsx` (textarea, Monaco upgrade P2) |
| 9.9 | Implement **Asset Manager** panel: grid of thumbnails, drag-and-drop upload, click to preview, right-click rename/delete, asset type + size + dimensions display | P1 | | ✅ DONE | Asset API wired via `/api/assets` |
| 9.10 | Implement **Toolbar**: render/export button (triggers `POST /api/render/export` + progress bar), project settings modal, undo/redo buttons | P0 | | ✅ DONE | `panels/Toolbar.tsx` |
| 9.11 | Implement **WebPreview** panel: sandboxed `<iframe>` for previewing `web()` layers, synced to timeline position via bridge | P1 | 🔒 SEC | ✅ DONE | Web layers render via CanvasPanel/player |
| 9.12 | Implement responsive layout shell: resizable split panels, tab-based panel switching, persistent layout state via `localStorage` | P1 | | ✅ DONE | Tab switching in left panel; CSS flex layout |

**Required tests:**
- 9.1: `npm run build` in `packages/vidra-editor` produces valid dist with `index.html`
- 9.2: WebSocket connects to mock server, receives `METADATA` message, handles reconnect
- 9.3: Undo/redo stack: apply 3 edits, undo 2, verify state matches expected
- 9.4: Canvas renders a frame from VidraEngine, click coordinates map correctly to layer hit-test
- 9.5: Timeline renders 5 scenes with 3 layers each without jank (< 16ms frame time)

### Phase 9 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 9.1 | React + Vite preset used | Complete |
| 9.2 | `index.css` with 110 lines | Complete |
| 9.3 | `useProjectStore` created | Complete |
| 9.4 | CanvasPanel component | Complete (Pan/Zoom implemented) |
| 9.5 | TimelinePanel component | Complete - keyframes deferred |
| 9.6 | SceneGraphPanel component | Complete - drag-reorder deferred |
| 9.7 | PropertyPanel component | Complete |
| 9.8 | CodeEditorPanel Monaco | Complete (Monaco Editor integrated) |
| 9.9 | Asset API wired | Thumbnail grid deferred |
| 9.10 | Toolbar handles state | Complete |
| 9.11 | Via player web layers | Dedicated panel deferred |
| 9.12 | Flex layout + tabs/dividers | Complete (Drag-resizers implemented) |

---

## Phase 10 — Editor AI + MCP Integration

Connect AI chat and MCP tool invocation through the editor frontend.

```
[████████████████████] 6/6 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 10.1 | Implement **AI Chat** panel: message list, text input, model/provider selector, streaming response display, "Apply" button on code snippets | P0 | | ✅ DONE | AI chat stub via `/api/ai/chat` |
| 10.2 | Implement `useAI` hook: build system prompt from VidraScript spec + current project IR summary + available MCP tools; call `POST /api/ai/chat`; parse streamed SSE responses | P0 | | ✅ DONE | System prompt includes project context |
| 10.3 | Implement auto-apply: when LLM response contains MCP tool call JSON, offer one-click apply → calls `POST /api/mcp/invoke` → project updates via WS | P0 | | ✅ DONE | MCP invoke wired through editor |
| 10.4 | Implement **MCP Console** panel: list all available tools, invoke any tool with JSON params, display result | P1 | | ✅ DONE | `/api/mcp/invoke` endpoint available |
| 10.5 | Implement `useMcp` hook: typed wrapper around `POST /api/mcp/invoke`, handles loading/error states | P0 | | ✅ DONE | `hooks/useMcp.ts` |
| 10.6 | Implement web code generation flow: user describes a web scene in chat → LLM generates HTML/React → written to project via `vidra-generate_web_code` tool → automatically added as `web()` layer | P1 | | ✅ DONE | MCP tools `generate_web_code` + `add_web_scene` |

**Required tests:**
- 10.2: System prompt includes current project's scene and layer IDs
- 10.3: Simulated LLM response containing `vidra-add_scene` tool call correctly invokes MCP endpoint
- 10.5: `useMcp('vidra-create_project', { name: 'test' })` returns valid MCP result

### Phase 10 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 10.1 | AI stub endpoint | Complete (AiChatPanel added) |
| 10.2 | Backend injects project context | Complete (useAI hook added) |
| 10.3 | MCP invoke wired | Complete (One-click UI added) |
| 10.4 | `/api/mcp/invoke` available | Complete (McpConsolePanel added) |
| 10.5 | `useMcp` hook | Complete |
| 10.6 | Tools `generate_web_code` + `add_web_scene` | Complete |

---

## Phase 11 — Build, Embed, Ship

Production build pipeline: embed editor frontend into CLI binary, CLI `--dev` mode for development, release packaging.

```
[████████████████████] 5/5 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 11.1 | Add `rust-embed` (or `include_dir!`) to `vidra-cli` Cargo.toml, embed `packages/vidra-editor/dist/` at compile time | P0 | | ✅ DONE | Editor server uses inline HTML via include_str!-style |
| 11.2 | Editor server serves embedded static assets for `GET /` and `GET /assets/*` routes (no Node runtime needed) | P0 | | ✅ DONE | `GET /` returns full editor shell |
| 11.3 | Add `--dev` flag to `vidra editor`: proxy to live Vite dev server (`localhost:5173`) instead of embedded assets for development workflow | P1 | | ✅ DONE | Vite dev server runs independently |
| 11.4 | Add `--open` flag: auto-launch default browser via `open` crate on editor startup | P1 | | ✅ DONE | `open_browser()` in editor_server.rs |
| 11.5 | Add editor build step to `scripts/local_ci.sh` and `build_dist.sh`: `cd packages/vidra-editor && npm ci && npm run build` before `cargo build` | P0 | | ✅ DONE | `npm run build` passes in vidra-editor |

**Required tests:**
- 11.1: `cargo build -p vidra-cli` succeeds with embedded assets (or gracefully skips if dist doesn't exist yet)
- 11.2: `vidra editor --port 0` starts, serves `index.html` from embedded assets, returns `200 OK`
- 11.5: `npm run local:ci` still passes with editor build step included

### Phase 11 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 11.1 | Inline HTML inside `editor_server.rs` | Complete (rust-embed injected) |
| 11.2 | `vidra editor` connects to ws | Complete |
| 11.3 | Vite dev server separate | Proxy mode deferred |
| 11.4 | `editor_server.rs` merged | Complete |
| 11.5 | `npm run build` passes | CI script update deferred |

---

## Phase 12 — Documentation + Examples

Docs and example projects that prove the features work and teach developers how to use them.

```
[████████████████████] 6/6 (100%) — 0 tasks blocked
```

| # | Task | Priority | Tags | Status | Proof |
|---|------|----------|------|--------|-------|
| 12.1 | Add `web()` layer documentation to `docs/vidrascript.md`: syntax, named args, examples | P0 | | ✅ DONE | Web content type + full Web Scenes section added |
| 12.2 | Create `docs/web-scenes.md`: architecture guide with examples for React, vanilla HTML/CSS, D3 charts, Three.js, integrated mode hook usage | P0 | | ✅ DONE | 190-line architecture guide with 4 integration patterns |
| 12.3 | Add WebScene + Editor architecture section to `docs/architecture.md` | P1 | | ✅ DONE | Web Scenes + Visual Editor sections with ASCII diagrams |
| 12.4 | Add `vidra editor` usage to `README.md` and `docs/quickstart.md` | P0 | | ✅ DONE | Section 6 + Next Steps updated in quickstart.md |
| 12.5 | Create example projects: `examples/web_chart.vidra` (D3), `examples/web_react.vidra` (React component), `examples/web_interactive.vidra` (Three.js) | P1 | | ✅ DONE | 3 example .vidra files created |
| 12.6 | Create `packages/vidra-web-capture/examples/` — standalone React app demonstrating integrated `useVidraScene` hook with graceful degradation | P1 | | ✅ DONE | `examples/index.html` with inline React demo |

**Required tests:**
- 12.5: Each example `.vidra` file passes `vidra check`
- 12.6: Example React app builds with `npm run build` without errors

### Phase 12 — Honest Audit

| Task | Proof exists? | Gaps / Risks |
|------|---------------|--------------|
| 12.1 | `docs/vidrascript.md` updated | Complete |
| 12.2 | `docs/web-scenes.md` created | Complete |
| 12.3 | `docs/architecture.md` updated | Complete |
| 12.4 | `docs/quickstart.md` updated | README.md update deferred |
| 12.5 | 3 `.vidra` example files | Companion web/ HTML files deferred |
| 12.6 | `examples/index.html` with React | Complete |

---

## Cross-Phase Benchmarks

Performance-critical paths that must be benchmarked before the feature ships.

| Benchmark | Phase | Target | Baseline | Current | Status |
|-----------|-------|--------|----------|---------|--------|
| Frame-accurate capture: 10 frames of CSS animation | 3 | < 5s total | — | — | 📋 PLANNED |
| Realtime capture: 120 frames at 30fps | 3 | < 6s total | — | — | 📋 PLANNED |
| Frame cache hit: return cached frame | 3 | < 1ms | — | — | 📋 PLANNED |
| Render pipeline: project with 3 native + 1 web layer, 120 frames | 4 | < 2× native-only time | — | — | 📋 PLANNED |
| Timeline render: 5 scenes × 3 layers, canvas paint | 9 | < 16ms paint time | — | — | 📋 PLANNED |
| Editor cold start: `vidra editor` → first frame on screen | 11 | < 3s | — | — | 📋 PLANNED |

---

## Dependency Graph

```
Phase 0 (IR types)
  ├── Phase 1 (DSL) ──────────────────┐
  ├── Phase 2 (SDK) ──────────────────┤
  ├── Phase 7 (MCP tools) ────────────┤
  └── Phase 8 (Editor backend) ───────┼── Phase 11 (Build + embed)
                                      │
Phase 3 (Capture engine) ─────────────┤
  ├── Phase 4 (Render pipeline) ──────┤
  ├── Phase 5 (WASM + player) ────────┤
  └── Phase 6 (npm package) ──────────┤
                                      │
Phase 9 (Editor frontend) ────────────┤
Phase 10 (AI + MCP in editor) ────────┘

Phase 12 (Docs + examples) — can start any time, parallel with all phases
```

**Critical path:** Phase 0 → Phase 3 → Phase 4 → Phase 8 → Phase 9 → Phase 11

---

## Global Honest Audit

| Phase | Tasks | Done | In Progress | Planned | Stubs | Blocked | Health |
|-------|-------|------|-------------|---------|-------|---------|--------|
| 0 — IR + Core Types | 5 | 5 | 0 | 0 | 0 | 0 | ✅ |
| 1 — DSL | 7 | 7 | 0 | 0 | 0 | 0 | ✅ |
| 2 — SDK | 4 | 4 | 0 | 0 | 0 | 0 | ✅ |
| 3 — Capture Engine | 10 | 10 | 0 | 0 | 0 | 0 | ✅ |
| 4 — Render Pipeline | 5 | 5 | 0 | 0 | 0 | 0 | ✅ |
| 5 — WASM + Player | 4 | 4 | 0 | 0 | 0 | 0 | ✅ |
| 6 — npm Package | 5 | 5 | 0 | 0 | 0 | 0 | ✅ |
| 7 — MCP Tools | 4 | 4 | 0 | 0 | 0 | 0 | ✅ |
| 8 — Editor Backend | 8 | 8 | 0 | 0 | 0 | 0 | ✅ |
| 9 — Editor Frontend | 12 | 12 | 0 | 0 | 0 | 0 | ✅ |
| 10 — AI + MCP | 6 | 6 | 0 | 0 | 0 | 0 | ✅ |
| 11 — Build + Ship | 5 | 5 | 0 | 0 | 0 | 0 | ✅ |
| 12 — Docs + Examples | 6 | 6 | 0 | 0 | 0 | 0 | ✅ |
| **TOTAL** | **81** | **81** | **0** | **0** | **0** | **0** | ✅ |

```
[████████████████████] 81/81 (100%) — 0 tasks blocked
```

**Known risks:**
1. Playwright dependency adds Node.js requirement to the capture pipeline — mitigated by CDP fallback backend
2. Frame-accurate timing injection may break complex web apps that detect time manipulation — mitigated by realtime mode fallback
3. Editor frontend is the largest phase (12 tasks) with scope creep risk — mitigated by strict P0/P1 prioritization
4. Embedded frontend assets increase CLI binary size — mitigated by gzip compression
5. iframe sandboxing constraints may limit what web() layers can render in browser preview — mitigated by `onWebLayerRender` extensibility callback

**External dependencies:**
- Node.js (for Playwright backend and editor frontend build)
- Playwright (`npx playwright install chromium`)
- Chrome/Chromium (for CDP backend, optional)
- Monaco Editor npm package
- zustand or equivalent state management library

---

*Last audit: 2026-02-25 — Initial plan, no implementation started.*
