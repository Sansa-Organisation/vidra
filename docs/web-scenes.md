# Web Scenes — Architecture Guide

Web Scenes let you embed any web technology — React, D3, Three.js, vanilla HTML/CSS — as composited layers in a Vidra video. This document explains the architecture, rendering modes, and integration patterns.

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                    .vidra source                      │
│                                                      │
│  layer("chart") {                                     │
│      web("web/chart.html", viewport: 800x400)        │
│  }                                                    │
└──────────┬───────────────────────────────────────────┘
           │ compile
           ▼
┌──────────────────────────────────────────────────────┐
│                  Vidra IR (JSON)                       │
│                                                       │
│  LayerContent::Web {                                  │
│      source: "web/chart.html",                        │
│      viewport_width: 800,                             │
│      viewport_height: 400,                            │
│      mode: "capture",                                 │
│      variables: {}                                    │
│  }                                                    │
└──────────┬──────────────────────┬────────────────────┘
           │                      │
    ┌──────▼──────┐      ┌───────▼────────┐
    │  Capture    │      │  Browser       │
    │  Engine     │      │  Player        │
    │  (CDP)      │      │  (iframe)      │
    └──────┬──────┘      └───────┬────────┘
           │                      │
    Frame-by-frame          Real-time DOM
    rasterization           overlay
```

## Rendering Modes

### Capture Mode (Default)

In capture mode, the engine uses a headless browser (via Chrome DevTools Protocol) to render web content frame-by-frame:

1. **Page Load**: The capture engine opens the HTML file in a headless browser with the specified viewport dimensions.
2. **Frame Advance**: For each video frame, the engine sends a `__vidra_advance_frame` message containing `{ frame, time, fps, vars }`.
3. **Screenshot**: The engine captures the rendered frame as raw RGBA pixels.
4. **Compositing**: The captured pixels are fed into the GPU compositing pipeline alongside native layers.

This mode is **deterministic** — the same input always produces the same output, regardless of system clock or rendering speed.

### Realtime Mode

In realtime mode (used in the browser player and `vidra dev`), web layers render as `<iframe>` overlays positioned above the `<canvas>`:

1. **Iframe Creation**: A sandboxed `<iframe>` is created with `allow-scripts allow-same-origin`.
2. **Position Sync**: The iframe is positioned using CSS transforms matching the layer's computed bounding box from WASM.
3. **Frame Sync**: On each frame, a `postMessage` sends `{ type: "vidra_frame", frame, time, fps }` to the iframe.

This mode supports **interactivity** — mouse events, clicks, and DOM updates work naturally.

## The Communication Protocol

Web content communicates with the Vidra engine via the `window.__vidra` bridge:

```typescript
interface VidraBridge {
  capturing: boolean;           // true when running in the capture harness
  frame: number;                // current frame index
  time: number;                 // current time in seconds
  fps: number;                  // project framerate
  vars: Record<string, any>;   // variables passed from VidraScript
  emit: (key: string, value: any) => void;  // send data back to engine
}
```

When the page loads inside the capture harness, `window.__vidra.capturing` is `true` and values update on each frame. Outside the harness (standalone browser, development), the bridge degrades gracefully to clock-based defaults.

## Integration Patterns

### 1. Vanilla HTML/CSS

The simplest approach — write self-contained HTML with CSS animations:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin: 0; background: transparent; font-family: 'Inter', sans-serif; }
    .counter {
      font-size: 120px;
      color: #58a6ff;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
    }
  </style>
</head>
<body>
  <div class="counter" id="value">0</div>
  <script>
    window.addEventListener('message', (e) => {
      if (e.data?.type === 'vidra_frame') {
        document.getElementById('value').textContent = Math.floor(e.data.frame / 30);
      }
    });
  </script>
</body>
</html>
```

### 2. React Component

Build a React app that uses the `@sansavision/vidra-web-capture` SDK:

```tsx
import { useVidraScene } from '@sansavision/vidra-web-capture/react';

function StockTicker() {
  const { time, vars } = useVidraScene();
  const price = vars.basePrice + Math.sin(time * 2) * vars.volatility;

  return (
    <div style={{ padding: '20px', background: 'rgba(0,0,0,0.8)', borderRadius: '12px' }}>
      <div style={{ color: '#8b949e', fontSize: '14px' }}>{vars.symbol}</div>
      <div style={{ color: price > vars.basePrice ? '#3fb950' : '#f85149', fontSize: '48px', fontWeight: 700 }}>
        ${price.toFixed(2)}
      </div>
    </div>
  );
}
```

### 3. D3.js Chart

Create animated data visualizations that respond to the Vidra timeline:

```javascript
import * as d3 from 'd3';
import { VidraCapture } from '@sansavision/vidra-web-capture';

const capture = new VidraCapture();
const data = [30, 80, 45, 60, 20, 90, 55];

function render() {
  const progress = capture.time / 3; // animate over 3 seconds
  const visibleCount = Math.ceil(progress * data.length);

  d3.select('#chart')
    .selectAll('rect')
    .data(data.slice(0, visibleCount))
    .join('rect')
    .attr('x', (_, i) => i * 60)
    .attr('y', d => 200 - d * 2)
    .attr('width', 50)
    .attr('height', d => d * 2)
    .attr('fill', '#58a6ff')
    .attr('rx', 4);
}

setInterval(render, 16);
```

### 4. Three.js Scene

Embed 3D content that synchronizes with the video timeline:

```javascript
import * as THREE from 'three';
import { VidraCapture } from '@sansavision/vidra-web-capture';

const capture = new VidraCapture();
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, 16/9, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ alpha: true });

// Rotate based on Vidra timeline
function animate() {
  requestAnimationFrame(animate);
  mesh.rotation.y = capture.time * 0.5;
  mesh.rotation.x = Math.sin(capture.time) * 0.3;
  renderer.render(scene, camera);
}
animate();
```

## MCP Tools for Web Scenes

AI agents can create and manage web scenes programmatically:

| Tool | Description |
|---|---|
| `vidra-add_web_scene` | Add a `web()` layer to an existing scene |
| `vidra-edit_web_scene` | Modify source, viewport, mode, or variables of a web layer |
| `vidra-generate_web_code` | Generate and save HTML/React code to `web/` directory |

Example AI workflow:
1. Agent calls `vidra-generate_web_code` to create `web/chart.html`
2. Agent calls `vidra-add_web_scene` to add the layer to a scene
3. User previews in `vidra editor` or `vidra dev`
4. Agent calls `vidra-edit_web_scene` to adjust viewport or variables

## Editor Integration

The `vidra editor` command provides a visual editing environment for web scenes:

```bash
vidra editor main.vidra --port 3001 --open
```

The editor shows:
- **Canvas panel**: Renders native layers via WASM, web layers via iframe overlays
- **Timeline**: Scrub through frames, web layers update in real-time
- **Property inspector**: Edit web layer properties (source, viewport, variables)
- **Code editor**: Edit VidraScript source with live recompilation

## Performance Considerations

- **Capture mode** is slower but deterministic — each frame requires a full page render + screenshot
- **Realtime mode** is fast but depends on the browser's rendering performance
- Use `wait_for` with a CSS selector to ensure the page is fully rendered before capture
- Keep web content lightweight — minimize DOM nodes, avoid heavy JS computation per frame
- The frame cache prevents re-rendering identical frames
