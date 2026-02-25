# Vidra Development Tasklist

This document tracks the ongoing implementation of the S-Tier feature roadmap for the Vidra engine.

## Phase 1 — Quick Wins (✅ Completed)
- [x] Native Effects Library (vidra-fx, WGSL compute chaining)
- [x] Animation Presets & Easings (fadeInUp, EaseOutBack)
- [x] Scene Transitions (morph, crossfade, wipe, slide)
- [x] Timeline Variables (`@var`, basic expressions)

---

## Phase 2 — Power Users (✅ Completed)
These features unlock creative depth using the existing architecture.

- [x] **Animation Groups & Stagger**
  - [x] Implement `animate.group` for parallel animations.
  - [x] Implement `animate.sequence` for sequential animations.
  - [x] Implement `animate.stagger` for applying offset animations across multiple layers.
- [x] **Advanced Animation Types**
  - [x] Spring Physics animations (stiffness, damping instead of fixed duration).
  - [x] Expression-driven animations evaluated per frame.
  - [x] Path animations (animating position along bezier/SVG paths).
- [x] **Extended Animatable Properties**
  - [x] Animate layer text/content: `color`, `fontSize`.
  - [x] Animate shape properties: `cornerRadius`, `strokeWidth`.
  - [x] Animate video properties: `crop.top/left/right/bottom`, audio `volume`.
  - [x] Animate effect properties (e.g. blur radius, brightness level).
- [x] **Custom Shader Ingestion**
  - [x] Support `shader("path/to.wgsl")` to load and compile custom WGSL files.
  - [x] Inject uniforms (`@t`, `@resolution`) into custom shaders automatically.
- [x] **Additional Exporters**
  - [x] WebM (VP9) multi-format export for web.
  - [x] GIF / APNG exporters for shorts and stickers.

---

## Phase 3 — Platform (✅ Completed)
Transforming Vidra from a tool into a platform.

- [x] **Responsive Constraint-Based Layout Engine**
  - [x] Implement constraints parser (`center(horizontal)`, `pin(top)`, `below("title")`).
  - [x] Implement layout solver engine to handle multi-aspect ratio rendering dynamically.
- [x] **Data-Driven Templates**
  - [x] Import data files (`@data "contacts.csv"` or JSON).
  - [x] Bind row data to template values (`{{first_name}}`, `{{avatar_url}}`).
  - [x] Implement CLI batch rendering over bound data.
- [x] **Vidra Plugin Ecosystem (Rust)**
  - [x] Define core `VidraPlugin`, `EffectPlugin`, `LayerPlugin`, and `TransitionPlugin` traits.
  - [x] Implement standard dynamic library loading or WASM module binding for plugins (`@plugin "name"`).
- [x] **VidraFX DSL Stabilization**
  - [x] Finalize custom `vidra-fx` typescript-like language to write effects natively in `.vfx` files.

---

## Phase 4 — Frontier 
Next-generation differentiators for the video engine.

- [x] **AI-Native Video Capabilities (Native + WASM)**
  - [x] **AI Provider Layer (shared)**
    - [x] Deterministic cache keys + local cache layout for AI outputs (audio, captions, masks).
    - [x] Provider config in `vidra.config.toml` + env var support (keys never stored in repo).
    - [x] OpenAI-compatible HTTP clients (base_url override for OpenAI/Groq/other compatibles).
    - [x] Gemini provider (HTTP) for captioning / image tasks.
    - [x] ElevenLabs provider (HTTP) for high-quality TTS.
    - [x] WASM/JS bridge plan: API calls in JS (React Native/Expo/web) with the same cache keys.
  - [x] **TTS (Text-to-Speech)**
    - [x] Materialize `LayerContent::TTS` → muxed audio track in exported video.
    - [x] Support OpenAI TTS + ElevenLabs as providers.
    - [x] Cache & reuse identical TTS across batch renders (`--data`).
  - [x] **AutoCaptions (Whisper / provider-backed)**
    - [x] Materialize `LayerContent::AutoCaption` into timed text layers (word/segment timestamps).
    - [x] OpenAI Whisper API (or compatible) transcription with `verbose_json` timestamps.
    - [x] VTT/JSON caption asset caching + deterministic IDs.
    - [x] WASM/JS runtime: captions generation in web/RN and feed IR patches to renderer.
  - [x] **AI Background Removal**
    - [x] Add `removeBackground` effect (image-first) and a deterministic cached alpha-mask output.
    - [x] Provider-backed implementation (e.g. remove.bg / Clipdrop / Gemini) + local fallbacks.
    - [x] WASM path: remote API call returns PNG-with-alpha (or mask) for compositing.
  - [x] **Plugin migration**
    - [x] Migrate 2–3 hardcoded effects into VidraFX DSL to validate the new plugin pipeline.
- [x] **Advanced Audio Features**
  - [x] Waveform visualization layers.
  - [x] Audio-Reactive animation driver (`1 + audio.amplitude`).
  - [x] Audio ducking and mixer timeline controls.
- [x] **Live & Interactive Mode**
  - [x] WASM API: set/get mouse position.
  - [x] Evaluate state like `@mouse.x` and `@mouse.y`.
  - [x] Reactive elements (`@on click { set count = count + 1 }`).
- [x] **2.5D Layer Transforms**
  - [x] Add perspective GPU projection (translateZ, rotateY, rotateX) for planar layer warping.
  - [x] CPU homography fallback for 2.5D (native + WASM) to validate semantics end-to-end.
- [x] **Asset Management Enhancements**
  - [x] Remote asset fetching & local caching (http/https asset paths cached under `resources.cache_dir`).
  - [x] LUT pipeline for cinematic color grading.
  - [x] Spritesheet decoding logic for UI / VFX animations.
- [x] **Collaborative Architecture**
  - [x] Flesh out existing CRDT module for real-time multiplayer editing or branched undo workflows.
