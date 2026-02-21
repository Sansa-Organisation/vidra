# Vidra — Implementation Tasklist

**Version:** 2.1  
**Last Updated:** 2026-02-21  
**Derived From:** [vidra_prd_v2.1.md](file:///Users/mohamedahmed/Downloads/Projects/dev/vidra/plan/vidra_prd_v2.1.md)

---

## Legend & Symbols

| Symbol | Meaning |
|--------|---------|
| `[ ]`  | Not started |
| `[/]`  | In progress |
| `[x]`  | Completed |
| `[-]`  | Blocked / On hold |
| `[!]`  | Needs review / Decision required |
| `[~]`  | Partially done / Needs rework |

### Priority Tags

| Tag | Meaning |
|-----|---------|
| `P0` | **Critical path** — blocks other work, must ship |
| `P1` | **High priority** — core feature, needed for milestone |
| `P2` | **Medium priority** — important but not blocking |
| `P3` | **Low priority** — nice to have, can defer |

### Ownership Tags

| Tag | Owner |
|-----|-------|
| `@engine` | Rust/GPU rendering team |
| `@lang` | VidraScript / DSL / compiler team |
| `@sdk` | SDK & API team |
| `@cli` | CLI & developer tooling team |
| `@infra` | Cloud / infrastructure team |
| `@ai` | AI / MCP team |
| `@platform` | Platform & marketplace team |
| `@docs` | Documentation & DX team |
| `@gamedev` | Game developer pipeline team |
| `@design` | Design / brand team |

### Dependency Notation

- `→ depends on [X.Y]` means this task requires item X.Y to be complete first
- `⚡ perf-critical` marks performance-sensitive tasks with SLA targets
- `🔒 conformance` marks tasks gated by the conformance test suite
- `🧪 testable` marks tasks that require automated test coverage before merge

---

## Rules

1. **No task moves to `[x]` without automated tests** (where applicable, marked with `🧪`)
2. **Conformance-gated tasks (`🔒`) cannot ship until they pass** the full conformance suite on all target hardware
3. **Performance-critical tasks (`⚡`) must include benchmarks** before and after, with results committed to the repo
4. **Blocked tasks (`[-]`) must have a linked reason** — add a comment below the item explaining the blocker
5. **`[!]` items require a design decision** before implementation — create an RFC or discuss in the relevant channel
6. **Phase ordering is strict** — Phase N+1 items should not begin until their Phase N dependencies are marked `[x]`
7. **Every `[x]` item must have a corresponding PR** linked in a comment below the checklist item
8. **Mark items `[/]` as soon as you start** — do not leave stale `[ ]` markers on active work
9. **Review `[~]` items weekly** — partial completions that linger become tech debt

---

## Progress Overview

| Phase | Total | Done | In Progress | Blocked | Not Started |
|-------|-------|------|-------------|---------|-------------|
| **Phase 0 — Prototype** | 30 | 30 | 0 | 0 | 0 |
| **Phase 1 — Developer Release** | 82 | 82 | 0 | 0 | 0 |
| **Phase 1.5 — Platform Soft Launch** | 43 | 43 | 0 | 0 | 0 |
| **Phase 2 — AI & Cloud** | 42 | 42 | 0 | 0 | 0 |
| **Phase 3 — Ecosystem & Edge** | 20 | 20 | 0 | 0 | 0 |
| **Totals** | **217** | **217** | **0** | **0** | **0** |

---

## Phase 0 — Prototype (Months 1–3)

**Milestone:** A VidraScript file renders to an MP4 that is correct and fast.

### 0.1 — Rust Render Pipeline `P0` `@engine`

- [x] 0.1.1 — Set up Rust project structure and workspace (crates: `vidra-core`, `vidra-cli`, `vidra-ir`, `vidra-render`, `vidra-encode`) `P0`
- [x] 0.1.2 — Define core `Frame` and `FrameBuffer` types `P0` `🧪`
- [x] 0.1.3 — Implement basic render loop: IR → render graph → frame output `P0` `🧪`
- [x] 0.1.4 — Implement single-threaded frame compositor `P0` `🧪`
- [x] 0.1.5 — Wire end-to-end pipeline: parse → compile → render → encode → file `P0` `🧪`
- [x] 0.1.6 — Establish error handling and Result types across crates `P1`

### 0.2 — VidraScript Parser (Subset) `P0` `@lang`

- [x] 0.2.1 — Define VidraScript grammar (subset: `project`, `scene`, `layer`, `text`, `image`, `video`, `shape`, `solid`) `P0`
- [x] 0.2.2 — Implement lexer / tokenizer `P0` `🧪`
- [x] 0.2.3 — Implement parser → AST `P0` `🧪`
- [x] 0.2.4 — Implement AST → IR compiler `P0` `🧪`
- [x] 0.2.5 — Error reporting with source locations (file, line, column) `P1` `🧪`
- [x] 0.2.6 — Write parser test suite (valid and invalid inputs) `P0` `🧪`

### 0.3 — Vidra IR (Core) `P0` `@engine`

- [x] 0.3.1 — Define IR node types: `Project`, `Settings`, `Scene`, `Layer`, `Asset` `P0` `🧪`
- [x] 0.3.2 — Implement IR tree structure with semantic addressing (`video.scenes[0].layers["name"]`) `P0` `🧪`
- [x] 0.3.3 — Implement IR serialization to JSON `P1` `🧪`
- [x] 0.3.4 — Implement IR deserialization from JSON `P1` `🧪`
- [x] 0.3.5 — Implement content hashing for deterministic output verification `P1` `🔒` `🧪`

### 0.4 — Layer Types `P0` `@engine`

- [x] 0.4.1 — Text layer (font loading, basic text rendering) `P0` `🧪` <!-- Phase 0 text layout complete -->
- [x] 0.4.2 — Image layer (decode PNG/JPEG, compositing) `P0` `🧪`
- [x] 0.4.3 — Video layer (decode via FFmpeg bindings, frame extraction) `P0` `🧪`
- [x] 0.4.4 — Shape layer (rect, circle, path) `P1` `🧪`
- [x] 0.4.5 — Solid color layer `P0` `🧪`

### 0.5 — Basic Animation System `P0` `@engine`

- [x] 0.5.1 — Define `Keyframe` and `Animation` types in IR `P0` `🧪`
- [x] 0.5.2 — Implement keyframe interpolation (linear) `P0` `🧪`
- [x] 0.5.3 — Implement easing functions (easeIn, easeOut, easeInOut) `P1` `🧪`
- [x] 0.5.4 — Animate core properties: position, scale, rotation, opacity `P0` `🧪`

### 0.6 — Encoding & Output `P0` `@engine`

- [x] 0.6.1 — Integrate FFmpeg bindings for H.264 encoding `P0` `🧪`
- [x] 0.6.2 — Implement frame-to-encoded-stream pipeline `P0` `🧪`
- [x] 0.6.3 — MP4 container output with correct metadata `P0` `🧪`

### 0.7 — CLI: `vidra render` `P0` `@cli`

> → depends on [0.1, 0.2, 0.6]

- [x] 0.7.1 — Implement `vidra render <file>` command (argument parsing, config) `P0` `🧪`
- [x] 0.7.2 — Structured output (progress bar, render stats, timing) `P1`
- [x] 0.7.3 — Error reporting with actionable messages `P1`

### 0.8 — Conformance Test Suite (v0) `P0` `@engine` `🔒`

- [x] 0.8.1 — Design conformance test framework (reference renders, pixel-diff comparison) `P0`
- [x] 0.8.2 — Create 10 reference test cases covering all 5 layer types `P0` `🧪`
- [x] 0.8.3 — Implement CI pipeline to run conformance tests on every commit `P0`

### 0.9 — Internal Demo `P1` `@docs`

> → depends on [0.1–0.8]

- [x] 0.9.1 — Create demo VidraScript: 30-second branded intro `P1`
- [x] 0.9.2 — Document Phase 0 architecture decisions (ADR) `P2`

---

## Phase 1 — Developer Release (Months 4–9)

**Milestone:** 1,000 developers using Vidra weekly. 10x+ perf over Remotion (public benchmarks).

### 1.1 — GPU Acceleration `P0` `@engine` `⚡`

> → depends on [0.1]

- [x] 1.1.1 — Integrate wgpu for cross-platform GPU access (Vulkan, Metal, DX12) `P0`
- [x] 1.1.2 — Implement GPU compositor (layer blending on GPU) `P0` `🧪`
- [x] 1.1.3 — Implement WGSL shader pipeline for built-in effects `P0` `🧪`
- [x] 1.1.4 — GPU-accelerated text rendering `P1` `🧪`
- [x] 1.1.5 — SIMD intrinsics for CPU-side pixel operations `P1` `⚡` `🧪`
- [x] 1.1.6 — GPU memory management (streaming asset decode, < 2 GB VRAM at 1080p) `P0` `⚡` `🧪`

### 1.2 — Multi-Threaded Rendering `P0` `@engine` `⚡`

> → depends on [0.1]

- [x] 1.2.1 — Parallelize frame rendering across threads `P0` `⚡` `🧪`
- [x] 1.2.2 — Thread-safe asset loading and caching `P0` `🧪`
- [x] 1.2.3 — Render graph partitioning for parallel GPU dispatch `P1` `⚡` `🧪`

### 1.3 — VidraScript Full Type System `P0` `@lang`

> → depends on [0.2]

- [x] 1.3.1 — Implement full type system (String, Number, Duration, Color, Image, etc.) `P0` `🧪`
- [x] 1.3.2 — Type inference for property assignments `P0` `🧪`
- [x] 1.3.3 — Static type checking pass `P0` `🧪`
- [x] 1.3.4 — Import system (local files, marketplace packages) `P1` `🧪`
- [x] 1.3.5 — Component definition syntax with typed props and defaults `P0` `🧪`
- [x] 1.3.6 — Conditional rendering (if/else in composition) `P1` `🧪`
- [x] 1.3.7 — Layout rules syntax for multi-target output `P1` `🧪`

### 1.4 — LSP Server `P1` `@lang`

> → depends on [1.3]

- [x] 1.4.1 — Implement Language Server Protocol server `P1`
- [x] 1.4.2 — Autocomplete for keywords, types, properties, and component props `P1`
- [x] 1.4.3 — Hover docs (type info, property descriptions) `P2`
- [x] 1.4.4 — Go-to-definition (components, imports) `P2`
- [x] 1.4.5 — Diagnostic errors and warnings with source locations `P1`
- [x] 1.4.6 — VS Code extension packaging `P1`

### 1.5 — `vidra fmt` — Formatter `P2` `@lang`

> → depends on [0.2]

- [x] 1.5.1 — Implement opinionated auto-formatter for VidraScript `P2` `🧪`
- [x] 1.5.2 — `vidra fmt --check` for CI enforcement `P2` `🧪`

### 1.6 — `vidra check` — Linter `P1` `@lang`

> → depends on [1.3]

- [x] 1.6.1 — Static analysis rules (unused layers, unreachable scenes, duplicate IDs) `P1` `🧪`
- [x] 1.6.2 — Type checking integration `P1` `🧪`
- [x] 1.6.3 — Configurable rule severity (error, warning, info) `P2`

### 1.7 — TypeScript Server SDK (`@vidra/sdk`) `P1` `@web`
- [x] 1.7.1 — TypeScript definitions for Vidra IR (components, layers, timeline)
- [x] 1.7.2 — Fluent Builder API for generating `.vidra` JSON natively
- [x] 1.7.3 — Auto-validation and lint checks before serialization `P2`
- [x] 1.7.6 — SDK documentation and examples `P1`

### 1.8 — Preview Server & Hot-Reload `P0` `@cli` `⚡`

> → depends on [1.1, 0.2]

- [x] 1.8.1 — Implement `vidra dev` local dev server `P0`
- [x] 1.8.2 — File watcher for VidraScript changes `P0`
- [x] 1.8.3 — IR diff engine (compute minimal changed frame set) `P0` `⚡` `🧪`
- [x] 1.8.4 — Incremental re-render of only affected frames `P0` `⚡` `🧪`
- [x] 1.8.5 — Browser-based preview player (WebSocket frame push) `P0`
- [x] 1.8.6 — Achieve < 500ms hot-reload latency target `P0` `⚡` `🔒`

### 1.9 — `vidra init` — Project Scaffolding `P1` `@cli`

- [x] 1.9.1 — Implement `vidra init <name>` with project template `P1`
- [x] 1.9.2 — Generate `vidra.config` with sensible defaults `P1`
- [x] 1.9.3 — Create starter `main.vidra` file `P1`
- [x] 1.9.4 — Assets directory scaffolding `P2`

### 1.10 — Component System `P0` `@engine` `@lang`

> → depends on [1.3]

- [x] 1.10.1 — Component definition and instantiation in IR `P0` `🧪`
- [x] 1.10.2 — Typed props with validation and defaults `P0` `🧪`
- [x] 1.10.3 — Component nesting (components containing components) `P0` `🧪`
- [x] 1.10.4 — Slots (components accept child content) `P1` `🧪`
- [x] 1.10.5 — Variants (`component.variant("dark")`) `P2` `🧪`
- [x] 1.10.6 — Component versioning `P2`

### 1.11 — Template System `P1` `@platform`

> → depends on [1.10]

- [x] 1.11.1 — Template package format specification `P1`
- [x] 1.11.2 — Pre-built templates (branded intro, lower-third, social post, product showcase) `P1`
- [x] 1.11.3 — `vidra add <template>` command `P1`

### 1.12 — Asset Pipeline `P1` `@engine`

- [x] 1.12.1 — Asset registry in IR (fonts, images, audio, video clips) `P1` `🧪`
- [x] 1.12.2 — Font loading and management (TTF/OTF, Google Fonts) `P1` `🧪`
- [x] 1.12.3 — Image asset pipeline (decode, cache, resize) `P1` `🧪`
- [x] 1.12.4 — Audio asset pipeline (decode, timeline placement, basic mixing) `P1` `🧪`
- [x] 1.12.5 — Video clip import (FFmpeg-based decode, seek, trim) `P1` `🧪`

### 1.13 — `vidra test` — Visual Regression Testing `P0` `@cli` `🧪`

> → depends on [1.1, 0.8]

- [x] 1.13.1 — Snapshot capture at key frames or time ranges `P0` `🧪`
- [x] 1.13.2 — Pixel-by-pixel diff with configurable tolerance `P0` `🧪`
- [x] 1.13.3 — `vidra test --update` to update baselines `P0`
- [x] 1.13.4 — CI-friendly output (exit codes, structured reports) `P1`
- [x] 1.13.5 — HTML diff report generation `P2`

### 1.14 — `vidra bench` — Performance Profiling `P1` `@cli` `⚡`

> → depends on [1.1, 1.2]

- [x] 1.14.1 — Benchmark runner across resolutions and durations `P1` `🧪`
- [x] 1.14.2 — Structured report (render time per scene, GPU mem, asset decode) `P1`
- [x] 1.14.3 — Regression detection against committed baseline `P1` `🧪`
- [x] 1.14.4 — CI integration (block PR if perf regresses beyond threshold) `P2`

### 1.15 — `vidra inspect` — Visual Debugger `P1` `@cli`

> → depends on [1.1]

- [x] 1.15.1 — Render tree visualization (layer hierarchy, shader info, GPU stats) `P1`
- [x] 1.15.2 — Frame-level inspection (hover-to-inspect any visual element) `P1`
- [x] 1.15.3 — Source mapping (click element → VidraScript source) `P2`
- [x] 1.15.4 — `vidra inspect --frame <N>` jump-to-frame `P2`

### 1.16 — Time-Travel Debugging `P2` `@engine`

> → depends on [1.15]

- [x] 1.16.1 — Emit replayable render traces per render job `P2`
- [x] 1.16.2 — Frame-level scrubbing through render graph execution `P2`
- [x] 1.16.3 — Inspect intermediate buffer states and shader outputs `P2`

### 1.17 — Export Formats `P0` `@engine`

> → depends on [0.6]

- [x] 1.17.1 — H.265 / HEVC encoding `P1` `🧪`
- [x] 1.17.2 — ProRes encoding (.mov) `P1` `🧪`
- [x] 1.17.3 — VP9 encoding (.webm) `P2` `🧪`
- [x] 1.17.4 — AV1 encoding (native encoder) `P2` `🧪`
- [x] 1.17.5 — PNG image sequence export `P1` `🧪`
- [x] 1.17.6 — GIF export `P2` `🧪`

### 1.18 — Import & Interop `P1` `@engine`

- [x] 1.18.1 — Lottie/Rive animation import → IR conversion `P1` `🧪`
- [x] 1.18.2 — Image sequence import `P1` `🧪`
- [x] 1.18.3 — FFmpeg filter graph import (subset) `P2` `🧪`

### 1.19 — Multi-Target Responsive Output `P1` `@engine` `@lang`

> → depends on [1.3.7]

- [x] 1.19.1 — Layout rule evaluation in IR → render graph `P1` `🧪`
- [x] 1.19.2 — `vidra render --targets 16:9,9:16,1:1,4:5` multi-output `P1` `🧪`
- [x] 1.19.3 — Per-target layout overrides and preview `P2`

### 1.20 — Deterministic Rendering `P0` `@engine` `🔒`

> → depends on [0.8]

- [x] 1.20.1 — Content-addressable output (same IR → same bytes) `P0` `🔒` `🧪`
- [x] 1.20.2 — Cross-platform conformance (NVIDIA, AMD, Apple Silicon) `P0` `🔒` `🧪`
- [x] 1.20.3 — Expand conformance suite to 100+ test cases `P0` `🔒`
- [x] 1.20.4 — CI matrix across all supported GPU vendors `P1`

### 1.21 — Game Dev Support `P1` `@gamedev`

> → depends on [1.1, 1.10]

- [x] 1.21.1 — Sprite sheet export (packed atlas, configurable padding) `P1` `🧪`
- [x] 1.21.2 — Unity sprite atlas format `P1` `🧪`
- [x] 1.21.3 — Unreal flipbook texture format `P2` `🧪`
- [x] 1.21.4 — Godot AnimatedSprite2D format `P2` `🧪`
- [x] 1.21.5 — `vidra export --spritesheet` CLI command `P1`
- [x] 1.21.6 — `vidra export --sequence` CLI command `P1`
- [x] 1.21.7 — Parameterized batch rendering (variant matrix) `P1` `🧪`
- [x] 1.21.8 — Procedural animation nodes (particles, noise, glow, dissolve) `P2` `🧪`
- [x] 1.21.9 — Engine-aware preview mode (color space, compression simulation) `P3`

### 1.22 — Video Storybook (Component Playground) `P2` `@cli` `@docs`

> → depends on [1.8, 1.10]

- [x] 1.22.1 — Local dev server rendering components in isolation `P2`
- [x] 1.22.2 — Adjustable props UI (sliders, dropdowns, text inputs) `P2`
- [x] 1.22.3 — Live preview per component `P2`

### 1.23 — Documentation `P0` `@docs`

- [x] 1.23.1 — Documentation site scaffolding (architecture, deploy pipeline) `P0`
- [x] 1.23.2 — Getting Started guide (install → first render < 60s) `P0`
- [x] 1.23.3 — VidraScript language reference `P0`
- [x] 1.23.4 — CLI reference (all commands, flags, examples) `P0`
- [x] 1.23.5 — TypeScript SDK API reference `P1`
- [x] 1.23.6 — Component authoring guide `P1`
- [x] 1.23.7 — Animation & easing reference `P2`
- [x] 1.23.8 — Game dev pipeline guide `P2`
- [x] 1.23.9 — Example projects (3–5 real-world examples) `P1`
- [x] 1.23.10 — Public benchmark results page `P1`

### 1.24 — Audio Engine `P1` `@engine`

> → depends on [0.1]

- [x] 1.24.1 — Rust-native audio mixer `P1` `🧪`
- [x] 1.24.2 — Sample-accurate audio/video sync `P1` `⚡` `🧪`
- [x] 1.24.3 — Audio effects: fade in/out, volume, ducking `P2` `🧪`
- [x] 1.24.4 — Multi-track audio mixing `P2` `🧪`

### 1.25 — Vidra License Token (VLT) `P0` `@platform` `@cli`

> → depends on [1.9]

- [x] 1.25.1 — VLT data model (JWT-like signed token with claims: plan, features, limits, expiry) `P0` `🧪`
- [x] 1.25.2 — `vidra auth login` — browser-based auth flow, VLT issuance and local storage `P0`
- [x] 1.25.3 — Offline VLT validation (local signature check, expiry + 7-day grace) `P0` `🧪`
- [x] 1.25.4 — Plan enforcement from VLT claims (feature gating, rate limits) `P1` `🧪`
- [x] 1.25.5 — `vidra auth create-key` / `list-keys` / `revoke-key` — API key management `P1`

### 1.26 — Telemetry System `P1` `@cli` `@infra`

> → depends on [1.25]

- [x] 1.26.1 — Telemetry data collection framework (render counts, duration, resolution, errors) `P1` `🧪`
- [x] 1.26.2 — Tiered telemetry levels (anonymous / identified / diagnostics) `P1` `🧪`
- [x] 1.26.3 — `vidra telemetry show` / `set` / `export` / `delete` CLI commands `P1`
- [x] 1.26.4 — Telemetry specification document (public transparency doc) `P2`

### 1.27 — `vidra doctor` — Environment Health Check `P1` `@cli`

> → depends on [1.1, 1.25]

- [x] 1.27.1 — GPU / driver / VRAM detection and validation `P1`
- [x] 1.27.2 — VLT validity check, asset cache integrity check `P1`
- [x] 1.27.3 — Conformance suite pass/fail summary, CLI/SDK version check `P1`
- [x] 1.27.4 — Structured output for bug reports `P2`
- [x] 1.27.5 — Cloud connectivity and sync status check `P2`

---

## Phase 1.5 — Platform Soft Launch (Months 8–12)

**Milestone:** 500 paying Pro users. $15K MRR. 5,000 resources in Vidra Commons.

### 1.5.1 — `vidra share` `P0` `@platform`

> → depends on [1.1]

- [x] 1.5.1.1 — Shareable preview link generation `P0`
- [x] 1.5.1.2 — Hosted preview player (web) `P0`
- [x] 1.5.1.3 — Timestamped commenting / feedback layer `P1`
- [x] 1.5.1.4 — Feedback loop (comments → MCP or manual edit) `P2`

### 1.5.2 — Brand Kit System `P1` `@platform` `@lang`

- [x] 1.5.2.1 — Brand kit data model (colors, fonts, logos, motion style) `P1` `🧪`
- [x] 1.5.2.2 — `@brand.*` reference syntax in VidraScript `P1` `🧪`
- [x] 1.5.2.3 — Brand kit management CLI / web UI `P2`
- [x] 1.5.2.4 — Auto-apply brand kit to projects `P2`

### 1.5.3 — Cloud Preview Rendering `P1` `@infra`

- [x] 1.5.3.1 — Cloud render worker (containerized Vidra engine) `P1`
- [x] 1.5.3.2 — Low-res cloud preview pipeline `P1`
- [x] 1.5.3.3 — Job queue and status API `P1`

### 1.5.4 — Team Workspaces `P2` `@platform`

- [x] 1.5.4.1 — Workspace creation and member management `P2`
- [x] 1.5.4.2 — Shared asset libraries `P2`
- [x] 1.5.4.3 — Team-scoped brand kits `P2`

### 1.5.5 — Version History `P2` `@platform`

- [x] 1.5.5.1 — Project version snapshots `P2`
- [x] 1.5.5.2 — Visual diffs between versions `P2`

### 1.5.6 — Marketplace (Curated, First-Party) `P1` `@platform`

- [x] 1.5.6.1 — Component publishing pipeline `P1`
- [x] 1.5.6.2 — Marketplace web UI (browse, install, preview) `P1`
- [x] 1.5.6.3 — `vidra add <package>` install from marketplace `P1`

### 1.5.7 — Pro Tier Launch `P0` `@platform`

- [x] 1.5.7.1 — Billing and subscription system ($29/month) `P0`
- [x] 1.5.7.2 — Feature gating (free vs. Pro limits) `P0`
- [x] 1.5.7.3 — Account management and dashboard `P1`

### 1.5.8 — Hybrid Sync Architecture `P0` `@infra` `@cli`

> → depends on [1.25]

- [x] 1.5.8.1 — `vidra sync` — bidirectional project metadata sync (push/pull/status) `P0`
- [x] 1.5.8.2 — Smart asset hydration (manifest-first sync, on-demand asset fetch, LRU cache) `P0` `⚡`
- [x] 1.5.8.3 — Offline reconciliation (last-write-wins metadata, content-addressed dedup) `P1`
- [x] 1.5.8.4 — `vidra.config.toml` sync settings section (`sync_source`, `sync_assets`, `auto_sync`) `P1`

### 1.5.9 — Render Receipts `P1` `@engine` `@infra`

> → depends on [1.25, 1.5.8]

- [x] 1.5.9.1 — Render receipt generation (IR hash, output hash, hardware info, duration, VLT ID) `P1` `🧪`
- [x] 1.5.9.2 — Ed25519 receipt signing and verification `P1` `🧪`
- [x] 1.5.9.3 — Auto-upload receipts on `vidra sync` `P1`
- [x] 1.5.9.4 — Cloud receipt dashboard (render history, analytics) `P2`

### 1.5.10 — Cloud Job Queue (Local Execution) `P1` `@infra` `@cli`

> → depends on [1.5.8]

- [x] 1.5.10.1 — `vidra jobs` — list pending render jobs from cloud `P1`
- [x] 1.5.10.2 — `vidra jobs --run` / `--run-all` — pull, render locally, upload result `P1`
- [x] 1.5.10.3 — `vidra jobs --watch` — daemon mode (continuous poll and execute) `P2`

### 1.5.11 — `vidra preview --share` `P1` `@cli` `@infra`

> → depends on [1.5.8]

- [x] 1.5.11.1 — Local low-res render + upload to cloud storage `P1`
- [x] 1.5.11.2 — Shareable link generation and clipboard copy `P1`

### 1.5.12 — Cloud Asset Management `P1` `@infra` `@cli`

> → depends on [1.5.8]

- [x] 1.5.12.1 — `vidra upload` — upload files/directories to cloud project storage `P1`
- [x] 1.5.12.2 — `vidra assets --list` / `--pull` — manage cloud-stored assets `P1`

### 1.5.13 — Vidra Commons (Initial) `P1` `@platform`

> → depends on [1.11]

- [x] 1.5.13.1 — Commons data model (resource types, metadata, content-addressed hashing) `P1` `🧪`
- [x] 1.5.13.2 — `vidra add` for resources (components + raw assets from Commons) `P1`
- [x] 1.5.13.3 — `vidra search <query>` — search Commons by type, tags, keyword `P1`
- [x] 1.5.13.4 — `vidra explore` — browse trending resources and featured work `P2`
- [x] 1.5.13.5 — License-aware asset management (`vidra licenses` output) `P1`

### 1.5.14 — Starter Kits `P1` `@platform` `@docs`

> → depends on [1.5.13, 1.11]

- [x] 1.5.14.1 — Starter kit package format (templates + components + sounds + fonts + examples) `P1`
- [x] 1.5.14.2 — `vidra init --kit <name>` — scaffold project with starter kit `P1`
- [x] 1.5.14.3 — YouTube Intro Kit (first-party) `P1`
- [x] 1.5.14.4 — Product Launch Kit (first-party) `P1`
- [x] 1.5.14.5 — Game UI Kit (first-party) `P2`

---

## Phase 2 — AI & Cloud (Months 10–18)

**Milestone:** 10K weekly active devs. 5K MCP renders/day. 50,000 resources in Vidra Commons. $100K MRR.

### 2.1 — Vidra MCP Server `P0` `@ai`

> → depends on [0.3, 1.7]

- [x] 2.1.1 — MCP server scaffolding (protocol implementation) `P0`
- [x] 2.1.2 — Tool: `vidra.create_project` `P0` `🧪`
- [x] 2.1.3 — Tool: `vidra.add_scene` `P0` `🧪`
- [x] 2.1.4 — Tool: `vidra.edit_layer` (semantic path editing) `P0` `🧪`
- [x] 2.1.5 — Tool: `vidra.set_style` `P1` `🧪`
- [x] 2.1.6 — Tool: `vidra.apply_brand_kit` `P1` `🧪`
- [x] 2.1.7 — Tool: `vidra.render_preview` / `vidra.render_final` `P0` `🧪`
- [x] 2.1.8 — Tool: `vidra.add_asset` `P1` `🧪`
- [x] 2.1.9 — Tool: `vidra.list_templates` `P2`
- [x] 2.1.10 — Tool: `vidra.share` `P2`
- [x] 2.1.11 — Tool: `vidra.add_resource` (pull from Vidra Commons) `P1` `🧪`
- [x] 2.1.12 — Tool: `vidra.list_resources` (search the resource library) `P2`
- [x] 2.1.13 — Tool: `vidra.storyboard` (visual storyboard from text) `P1` `🧪`

### 2.2 — Conversational Storyboarding `P1` `@ai`

> → depends on [2.1]

- [x] 2.2.1 — Tool: `vidra.storyboard` — text-to-storyboard generation `P1`
- [x] 2.2.2 — Storyboard key frame grid rendering `P1`
- [x] 2.2.3 — Storyboard iteration workflow (accept/reject/modify frames) `P2`

### 2.3 — Managed Cloud Rendering `P0` `@infra` `⚡`

> → depends on [1.5.3]

- [x] 2.3.1 — Auto-scaling render cluster (GPU instances) `P0`
- [x] 2.3.2 — `vidra render --cloud` CLI integration `P0`
- [x] 2.3.3 — Usage-based pricing engine (per render-second) `P0`
- [x] 2.3.4 — Render job API (REST + webhook) `P1`
- [x] 2.3.5 — CDN delivery for rendered output `P1`

### 2.4 — CRDT-Based Collaboration `P1` `@platform`

- [x] 2.4.1 — CRDT protocol for IR-level multiplayer editing `P1`
- [x] 2.4.2 — Real-time sync between code editor and visual editor `P1`
- [x] 2.4.3 — Presence indicators and cursor sharing `P2`

### 2.5 — Community Marketplace `P1` `@platform`

> → depends on [1.5.6]

- [x] 2.5.1 — Third-party component submission pipeline `P1`
- [x] 2.5.2 — Automated review (render test, lint, docs check) `P1` `🧪`
- [x] 2.5.3 — Revenue share system (80/20 creator/platform) `P1`
- [x] 2.5.4 — Marketplace search and discovery `P2`

### 2.6 — AI Copilot in Visual Editor `P2` `@ai`

- [x] 2.6.1 — Inline AI assistance in visual editor `P2`
- [x] 2.6.2 — Semantic editing via natural language ("make the intro faster") `P2`
- [x] 2.6.3 — AI-powered asset intelligence (auto-tagging, smart cropping) `P3`

### 2.7 — Native AI Pipeline Hooks `P1` `@ai` `@engine`

- [x] 2.7.1 — AI model nodes as first-class render graph elements `P1`
- [x] 2.7.2 — Shared GPU memory between AI models and renderer `P1` `⚡`
- [x] 2.7.3 — Built-in hooks: style transfer, object detection, generative fill `P2`

### 2.8 — Render Streaming (Progressive Output) `P2` `@engine` `⚡`

- [x] 2.8.1 — Chunked encoding for progressive playback `P2` `⚡`
- [x] 2.8.2 — Out-of-order frame assembly `P2`

### 2.9 — GitHub Integration `P2` `@platform`

- [x] 2.9.1 — GitHub Actions: render on PR `P2`
- [x] 2.9.2 — Visual diffs in PR review `P2`
- [x] 2.9.3 — Deploy-on-merge to CDN `P3`

### 2.10 — Python SDK `P1` `@sdk`

> → depends on [0.3]

- [x] 2.10.1 — Python SDK compiling to Vidra IR `P1` `🧪`
- [x] 2.10.2 — PyPI package publishing pipeline `P1`
- [x] 2.10.3 — Python SDK documentation `P1`

### 2.11 — Plugin System `P1` `@engine`

- [x] 2.11.1 — Plugin API specification (IR extension points) `P1`
- [x] 2.11.2 — Plugin loader and lifecycle management `P1` `🧪`
- [x] 2.11.3 — Sandboxed plugin execution (WASM-based) `P2`

### 2.12 — Team Tier & Render Dashboard `P1` `@platform`

- [x] 2.12.1 — Team tier launch ($79/seat/month) `P1`
- [x] 2.12.2 — Render observability dashboard (traces, GPU metrics) `P1`

### 2.13 — Community Publishing & Challenges `P2` `@platform`

> → depends on [1.5.13]

- [x] 2.13.1 — `vidra publish` — publish resources to Vidra Commons `P2`
- [x] 2.13.2 — Automated submission review (render test, metadata, license, content policy) `P2` `🧪`
- [x] 2.13.3 — Inspiration boards (curated collections, browsable via `vidra explore --boards`) `P2`
- [x] 2.13.4 — Community challenges system (weekly/monthly with featured showcases) `P3`

### 2.14 — Remaining Starter Kits `P2` `@platform` `@docs`

> → depends on [1.5.14]

- [x] 2.14.1 — Social Media Kit (first-party) `P2`
- [x] 2.14.2 — Corporate Kit (first-party) `P2`
- [x] 2.14.3 — Cinematic Kit (first-party) `P2`

---

## Phase 3 — Ecosystem & Edge (Months 18–30)

**Milestone:** 50K weekly active devs. 1M+ daily render jobs. $1M MRR.

### 3.1 — Edge Runtime `P0` `@engine` `@infra`

- [x] 3.1.1 — WASM-compiled lightweight renderer `P0`
- [x] 3.1.2 — Deploy to Cloudflare Workers / Fastly Compute `P0`
- [x] 3.1.3 — Personalized video at CDN edge (< 100ms) `P0` `⚡`

### 3.2 — Public IR Specification `P1` `@engine` `@docs`

- [x] 3.2.1 — Open IR spec document `P1`
- [x] 3.2.2 — Reference implementation and validation tools `P1`

### 3.3 — Open Collaboration Protocol `P2` `@platform`

- [x] 3.3.1 — Open CRDT protocol spec for multi-client editing `P2`

### 3.4 — Enterprise Features `P1` `@platform`

- [x] 3.4.1 — SSO (SAML, OIDC) `P1`
- [x] 3.4.2 — Audit logs `P1`
- [x] 3.4.3 — Role-based access control (RBAC) `P1`
- [x] 3.4.4 — Enterprise SLA guarantees `P2`

### 3.5 — After Effects Import `P2` `@engine`

- [x] 3.5.1 — .aep file parsing `P2`
- [x] 3.5.2 — AE project → Vidra IR conversion `P2` `🧪`

### 3.6 — Broadcast Integration `P2` `@engine`

- [x] 3.6.1 — RTMP live output `P2`
- [x] 3.6.2 — SRT live output `P2`

### 3.7 — Community & Third-Party `P2` `@platform`

- [x] 3.7.1 — Community runtime ports (specialized hardware) `P2`
- [x] 3.7.2 — Third-party plugin sandbox (public API) `P2`

### 3.8 — Render Formats Expansion `P2` `@engine`

- [x] 3.8.1 — MPEG-DASH adaptive bitrate export `P2` `🧪`
- [x] 3.8.2 — HLS adaptive bitrate export `P2` `🧪`

### 3.9 — Machine Seat Licensing `P2` `@platform`

> → depends on [1.25]

- [x] 3.9.1 — Hardware fingerprint in render receipts `P2`
- [x] 3.9.2 — Machine seat enforcement per plan tier (Pro: 3, Team: 5) `P2`

### 3.10 — Team Resource Registries `P2` `@platform`

> → depends on [1.5.13, 3.4]

- [x] 3.10.1 — Private resource registries for enterprise teams `P2`
- [x] 3.10.2 — Advanced analytics and render cost optimization recommendations `P3`

---

## Appendix: Cross-Cutting Concerns

These items apply continuously across all phases.

### A — CI/CD & Infrastructure

- [x] A.1 — Set up CI pipeline (Rust build, tests, linting) `P0` `@infra`
- [x] A.2 — Conformance test CI matrix (NVIDIA, AMD, Apple Silicon) `P0` `@infra`
- [x] A.3 — Automated benchmark regression detection in CI `P1` `@infra`
- [x] A.4 — Release automation and versioning (semver) `P1` `@infra`
- [x] A.5 — Install script (`curl -fsSL https://vidra.dev/install.sh | sh`) `P1` `@infra`

### B — Quality & Testing

- [x] B.1 — Unit test coverage target: ≥ 80% for core crates `P0` `@engine`
- [x] B.2 — Integration test suite for CLI commands `P1` `@cli`
- [x] B.3 — SDK test suites (TypeScript, Python) `P1` `@sdk`
- [x] B.4 — Fuzz testing for parser and IR compiler `P2` `@lang`

### C — Security

- [x] C.1 — Dependency auditing pipeline (`cargo audit`) `P1` `@infra`
- [x] C.2 — Sandboxed execution for marketplace components `P1` `@platform`
- [x] C.3 — API authentication and rate limiting (cloud layer) `P1` `@infra`

### D — Performance Monitoring

- [x] D.1 — Establish performance baselines for all SLA targets `P0` `@engine`
- [x] D.2 — Automated performance tracking per release `P1` `@engine`
- [x] D.3 — Public performance dashboard `P2` `@docs`
