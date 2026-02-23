# Vidra Development Tasklist

This document tracks the ongoing implementation of the S-Tier feature roadmap for the Vidra engine.

## Phase 1 — Quick Wins (✅ Completed)
- [x] Native Effects Library (vidra-fx, WGSL compute chaining)
- [x] Animation Presets & Easings (fadeInUp, EaseOutBack)
- [x] Scene Transitions (morph, crossfade, wipe, slide)
- [x] Timeline Variables (`@var`, basic expressions)

---

## Phase 2 — Power Users (🚧 Next Up)
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
- [ ] **Custom Shader Ingestion**
  - [ ] Support `shader("path/to.wgsl")` to load and compile custom WGSL files.
  - [ ] Inject uniforms (`@t`, `@resolution`) into custom shaders automatically.
- [ ] **Additional Exporters**
  - [ ] WebM (VP9) multi-format export for web.
  - [ ] GIF / APNG exporters for shorts and stickers.

---

## Phase 3 — Platform 
Transforming Vidra from a tool into a platform.

- [ ] **Responsive Constraint-Based Layout Engine**
  - [ ] Implement constraints parser (`center(horizontal)`, `pin(top)`, `below("title")`).
  - [ ] Implement layout solver engine to handle multi-aspect ratio rendering dynamically.
- [ ] **Data-Driven Templates**
  - [ ] Import data files (`@data "contacts.csv"` or JSON).
  - [ ] Bind row data to template values (`{{first_name}}`, `{{avatar_url}}`).
  - [ ] Implement CLI batch rendering over bound data.
- [ ] **Vidra Plugin Ecosystem (Rust)**
  - [ ] Define core `VidraPlugin`, `EffectPlugin`, `LayerPlugin`, and `TransitionPlugin` traits.
  - [ ] Implement standard dynamic library loading or WASM module binding for plugins (`@plugin "name"`).
- [ ] **VidraFX DSL Stabilization**
  - [ ] Finalize custom `vidra-fx` typescript-like language to write effects natively in `.vfx` files.

---

## Phase 4 — Frontier 
Next-generation differentiators for the video engine.

- [ ] **AI-Native Video Capabilities**
  - [ ] AI Background Removal (`effect(removeBackground)`).
  - [ ] AutoCaptions (via Whisper inference model).
  - [ ] TTS (Text-to-Speech) generation layer.
  - [ ] AI Style Transfer and AI Object Tracking.
  - [ ] Text-to-Audio / AI Music generation integration.
- [ ] **Advanced Audio Features**
  - [ ] Waveform visualization layers.
  - [ ] Audio-Reactive animation driver (`1 + audio.amplitude`).
  - [ ] Audio ducking and mixer timeline controls.
- [ ] **Live & Interactive Mode**
  - [ ] Evaluate state like `@mouse.x` and `@mouse.y`.
  - [ ] Reactive elements (`@on click { set count = count + 1 }`).
- [ ] **2.5D Layer Transforms**
  - [ ] Add perspective GPU projection (translateZ, rotateY, rotateX) for planar layer warping.
- [ ] **Asset Management Enhancements**
  - [ ] Remote asset fetching & local caching (`@assets { fetch(...) }`).
  - [ ] LUT pipeline for cinematic color grading.
  - [ ] Spritesheet decoding logic for UI / VFX animations.
- [ ] **Collaborative Architecture**
  - [ ] Flesh out existing CRDT module for real-time multiplayer editing or branched undo workflows.
