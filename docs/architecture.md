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
