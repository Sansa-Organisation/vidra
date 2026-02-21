# ADR 001: Phase 0 Architecture Decisions

**Status:** Accepted
**Date:** 2026-02-21

## Context

We are building the Vidra Engine (v2.0) from scratch to replace the Remotion-based architecture. Phase 0 requires us to build a working prototype that parses VidraScript, compiles it to an Intermediate Representation (IR), renders it to a sequence of frame buffers, and encodes the output into an H.264 MP4. 

The primary goals for Phase 0 were to establish the workspace structure, define the core data models, and prove the end-to-end viability of a Rust-first rendering approach.

## Decisions

### 1. Multi-Crate Workspace
**Decision:** We split the engine into multiple specialized crates (`vidra-core`, `vidra-ir`, `vidra-lang`, `vidra-render`, `vidra-encode`, `vidra-cli`).
**Rationale:** This enforces a strict directed acyclic graph (DAG) of dependencies (e.g., `vidra-render` relies on `vidra-ir`, but `vidra-ir` knows nothing about rendering) and allows independent compilation and testing.

### 2. Intermediate Representation (IR)
**Decision:** We designed a strongly-typed, serializable Intermediate Representation (IR) tree to represent the parsed state of a Vidra project.
**Rationale:** The IR serves as the system's lingua franca. It decouples the VidraScript parser from the renderer, enabling future SDKs (TypeScript/Python) to bypass the parser and generate IR directly.

### 3. CPU-Based Rendering for Phase 0
**Decision:** We implemented the initial renderer as a single-threaded, CPU-based compositor.
**Rationale:** This allowed us to iterate quickly on the core semantic logic (transforms, opacity, compositing logic, z-indexing) without the overhead and complexity of standing up a `wgpu` graphics pipeline out of the gate. GPU acceleration is deferred to Phase 1.

### 4. FFmpeg Subprocess for Video Operations
**Decision:** We use `ffmpeg` via external subprocesses for both decoding video assets (extracting frames) and encoding the final render (H.264 MP4).
**Rationale:** Writing native encoders/decoders is extremely complex. By shelling out to FFmpeg, we get a reliable, battle-tested media pipeline for Phase 0. Future phases will integrate FFmpeg native bindings or native Rust transcoders (`rav1e` etc.).

### 5. TrueType Font Rendering via `fontdue`
**Decision:** We selected `fontdue` for rendering text.
**Rationale:** `fontdue` provides fast, pure-Rust CPU rasterization and works well without needing native dependencies, making it ideal for the Phase 0 CPU renderer. Multi-line text was implemented manually by calculating spacing and offsets.

### 6. Deterministic Content Hashing
**Decision:** We integrated deterministic SHA-256 content hashing at the frame and project level.
**Rationale:** This creates a strict conformance test suite. A specific IR project configuration will always produce the exact same byte hash, allowing us to confidently refactor or migrate to GPU rendering later knowing we haven't introduced visual regressions.

## Consequences

- **Positives:** We have completely eliminated the Node.js/Chromium overhead. The architecture is modular and highly testable. The conformance suite locks down rendering behavior.
- **Negatives/To Address in Phase 1:** CPU rendering is currently slow and does not match our sub-50ms latency targets for real-time preview. Subprocess FFmpeg calls have an execution ceiling. Future iterations will replace the CPU pipeline with `wgpu`.
