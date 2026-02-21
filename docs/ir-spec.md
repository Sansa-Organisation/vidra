# Vidra IR — Open Intermediate Representation Specification

**Version:** 1.0.0-draft  
**Status:** Public Draft  
**Authors:** Vidra Team  

---

## 1. Overview

The Vidra IR (Intermediate Representation) is a **queryable, composable, deterministic scene graph** that serves as the universal language for video production. Every input surface — VidraScript, the TypeScript/Python SDKs, MCP tools, and the visual editor — compiles down to this IR before rendering.

### Design Goals

1. **Deterministic:** The same IR always produces the same output, byte-for-byte.
2. **Composable:** Scenes, layers, and components are nestable and reusable.
3. **Queryable:** Any node can be addressed by path for editing, animation, or AI mutation.
4. **Serializable:** The full IR is representable as JSON for storage, transport, and diffing.

---

## 2. Core Types

### 2.1 Project

The root node of any Vidra production.

```json
{
  "settings": {
    "width": 1920,
    "height": 1080,
    "fps": 60.0,
    "duration": { "seconds": 10.0 }
  },
  "scenes": [ ... ],
  "assets": { ... }
}
```

| Field      | Type             | Description                          |
|------------|------------------|--------------------------------------|
| `settings` | `ProjectSettings`| Resolution, framerate, total duration |
| `scenes`   | `Scene[]`        | Ordered list of scenes                |
| `assets`   | `AssetRegistry`  | Content-addressed asset store         |

### 2.2 Scene

A time-bounded segment of the production.

```json
{
  "id": "intro",
  "start_time": { "seconds": 0.0 },
  "end_time": { "seconds": 5.0 },
  "layers": [ ... ]
}
```

| Field        | Type       | Description                      |
|--------------|------------|----------------------------------|
| `id`         | `SceneId`  | Unique string identifier         |
| `start_time` | `Duration` | Scene start in the timeline      |
| `end_time`   | `Duration` | Scene end in the timeline        |
| `layers`     | `Layer[]`  | Layers rendered bottom-to-top    |

### 2.3 Layer

The fundamental renderable unit.

```json
{
  "id": "title",
  "content": {
    "type": "Text",
    "text": "Hello World",
    "font_family": "Inter",
    "font_size": 64.0,
    "color": { "r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0 }
  },
  "transform": {
    "position": { "x": 960.0, "y": 540.0 },
    "scale": { "x": 1.0, "y": 1.0 },
    "rotation": 0.0,
    "anchor": { "x": 0.5, "y": 0.5 },
    "opacity": 1.0
  },
  "blend_mode": "Normal",
  "animations": [ ... ],
  "effects": [ ... ],
  "children": [ ... ]
}
```

### 2.4 LayerContent (Union Type)

| Variant        | Fields                                                  |
|----------------|---------------------------------------------------------|
| `Text`         | `text`, `font_family`, `font_size`, `color`             |
| `Image`        | `asset_id`                                              |
| `Video`        | `asset_id`, `trim_start`, `trim_end`                    |
| `Audio`        | `asset_id`, `trim_start`, `trim_end`, `volume`          |
| `TTS`          | `text`, `voice`, `volume`                               |
| `AutoCaption`  | `asset_id`, `font_family`, `font_size`, `color`         |
| `Shape`        | `shape`, `fill`, `stroke`, `stroke_width`               |
| `Solid`        | `color`                                                 |
| `Empty`        | *(used for grouping / component instances)*              |

### 2.5 Animation

Keyframe-based property animations.

```json
{
  "property": "opacity",
  "keyframes": [
    { "time": { "seconds": 0.0 }, "value": 0.0 },
    { "time": { "seconds": 1.0 }, "value": 1.0 }
  ],
  "easing": "ease-in-out"
}
```

### 2.6 Asset

Content-addressed media reference.

```json
{
  "id": "hero-video",
  "asset_type": "Video",
  "path": "assets/hero.mp4"
}
```

---

## 3. Semantic Addressing

Every node in the IR is addressable by a **semantic path**:

```
project.scenes[intro].layers[title].content.text
project.scenes[intro].layers[bg].transform.opacity
```

This addressing scheme is used by:
- MCP tools (`vidra.edit_layer`, `vidra.set_style`)
- CRDT operations (collaborative editing)
- Animation targets
- AI Copilot mutations

---

## 4. Validation Rules

1. All `asset_id` references must resolve in the `AssetRegistry`.
2. Scene time ranges must not overlap.
3. Layer IDs must be unique within a scene.
4. Animation property names must match valid transform/content fields.
5. Color values must be in range `[0.0, 1.0]`.

---

## 5. Serialization

The canonical IR format is **JSON**. The Rust reference implementation uses `serde` for serialization.

```rust
let json = serde_json::to_string_pretty(&project)?;
let loaded: Project = serde_json::from_str(&json)?;
assert_eq!(project, loaded); // round-trip guarantee
```

---

## 6. Extension Points

Plugins can extend the IR through:

1. **Custom LayerContent variants** — registered via the Plugin API.
2. **Custom effects** — new `LayerEffect` types.
3. **Custom animation easings** — pluggable easing functions.

All extensions must pass through the WASM sandbox for safety.

---

## 7. CRDT Protocol

For real-time collaboration, the IR supports a CRDT-based synchronization protocol:

| Operation         | Description                                  |
|-------------------|----------------------------------------------|
| `InsertNode`      | Add a new layer/scene at a specified path    |
| `DeleteNode`      | Remove a node by ID                          |
| `UpdateProperty`  | Mutate a property at a semantic path         |
| `MoveNode`        | Reorder/reparent a node                      |
| `PresenceUpdate`  | Share cursor position and client metadata    |

See `vidra-ir/src/crdt.rs` for the reference implementation.

---

*This specification is part of Vidra's commitment to open infrastructure for programmable video.*
