# @sansavision/vidra-player

Real-time video rendering in the browser — powered by WebAssembly.

Compile VidraScript **or** use the fluent TypeScript/JavaScript SDK to build videos programmatically, then render them at 60fps on a `<canvas>`.

## Quick Start

```bash
# From the repo root
cd packages/vidra-player
npm install
npm run demo
# → opens http://localhost:3456/examples/demo.html
```

Then open **http://localhost:3456/examples/demo.html** in your browser.

## How It Works

```
┌─────────────────────────────────────────────────────┐
│                    Browser                          │
│                                                     │
│  ┌──────────────┐     ┌──────────────────────────┐  │
│  │ VidraScript   │────▶│ WASM Module (vidra-wasm) │  │
│  │ or JS SDK     │     │                          │  │
│  └──────────────┘     │  ┌────────────────────┐  │  │
│                        │  │ Parser + Compiler  │  │  │
│                        │  │ (vidra-lang)       │  │  │
│                        │  └────────┬───────────┘  │  │
│                        │           ▼              │  │
│                        │  ┌────────────────────┐  │  │
│                        │  │ CPU Renderer       │  │  │
│                        │  │ (single-threaded)  │  │  │
│                        │  └────────┬───────────┘  │  │
│                        └───────────┼──────────────┘  │
│                                    ▼                 │
│                        ┌──────────────────────┐      │
│                        │ RGBA pixel data      │      │
│                        │ → ImageData          │      │
│                        │ → <canvas>           │      │
│                        └──────────────────────┘      │
│                                    ▲                 │
│                        requestAnimationFrame         │
│                        (60fps render loop)            │
└─────────────────────────────────────────────────────┘
```

### Two Input Modes

#### 1. VidraScript DSL
The WASM module includes the full `vidra-lang` parser and compiler. You write `.vidra` syntax, and the WASM module compiles it to IR internally:

```
project(1920, 1080, 60) {
    scene("intro", 3s) {
        layer("bg") { solid(#1a1a2e) }
        layer("title") {
            text("Hello!", font: "Inter", size: 100, color: #ffffff)
            position(960, 540)
            animation(opacity, from: 0, to: 1, duration: 1s, easing: easeOut)
        }
    }
}
```

#### 2. JavaScript/TypeScript SDK
Build videos using the fluent builder API. The SDK generates IR JSON directly — no VidraScript compilation step needed:

```javascript
import { Project, Scene, Layer, Easing } from "@sansavision/vidra-player";

const project = new Project(1920, 1080, 60);
const scene = new Scene("intro", 3.0);
scene.addLayers(
    new Layer("bg").solid("#1a1a2e"),
    new Layer("title")
        .text("Hello!", "Inter", 100, "#ffffff")
        .position(960, 540)
        .animate("opacity", 0, 1, 1.0, Easing.EaseOut)
);
project.addScene(scene);
```

### Engine API

```typescript
import { VidraEngine } from "@sansavision/vidra-player";

const canvas = document.getElementById("canvas");
const engine = new VidraEngine(canvas, {
    onReady: () => console.log("WASM loaded"),
    onFrame: (frame) => console.log(`Frame ${frame}`),
    onStateChange: (state) => console.log(`State: ${state}`),
    onError: (err) => console.error(err),
});

// Initialize WASM (call once)
await engine.init();

// Load content (pick one)
engine.loadSource(vidraScript);       // VidraScript string
engine.loadProject(sdkProject);       // SDK Project object
engine.loadIR(irJsonString);          // Raw IR JSON

// Playback controls
engine.play();
engine.pause();
engine.stop();
engine.seekToFrame(120);
engine.seekToTime(2.0);

// State
engine.getCurrentFrame();   // → number
engine.getCurrentTime();    // → number (seconds)
engine.getState();          // → "idle" | "loading" | "playing" | "paused" | "stopped"
engine.getProjectInfo();    // → { width, height, fps, totalFrames, totalDuration, sceneCount }
```

## Build Commands

| Command | Description |
|---------|-------------|
| `npm run build` | Compile TypeScript → `dist/` |
| `npm run build:wasm` | Compile Rust → WASM → `wasm/` |
| `npm run build:all` | Build WASM + TypeScript |
| `npm run demo` | Build TS and start local server on port 3456 |
| `npm run dev` | Alias for `demo` |

## Architecture

The WASM module (`vidra-wasm`) is a standalone Rust crate that bundles:
- **vidra-lang**: VidraScript lexer, parser, and compiler
- **vidra-core**: Frame buffers, color types, duration types
- **vidra-ir**: Intermediate representation for the scene graph
- A **CPU-only renderer** (simplified from `vidra-render`, no GPU dependencies)

It compiles to ~2MB of WASM via `wasm-pack` and exposes these functions:
- `parse_and_compile(source)` → IR JSON string
- `render_frame(irJson, frameIndex)` → RGBA `Uint8Array`
- `get_project_info(irJson)` → metadata JSON
- `load_image_asset(id, bytes)` → cache image for rendering

The JavaScript `VidraEngine` class wraps these calls in a `requestAnimationFrame` loop, pushing RGBA pixel data to a 2D canvas via `ImageData`.

## Requirements

- Browser with WebAssembly support (all modern browsers)
- For rebuilding WASM: Rust + `wasm-pack`
