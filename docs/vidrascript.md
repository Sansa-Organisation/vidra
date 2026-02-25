# VidraScript Language Reference

VidraScript is a purpose-built domain-specific language (DSL) for declarative video composition. It offers structured syntax combining the declarative readability of CSS with the composability of modern UI frameworks (like React or SwiftUI).

## Project Structure

A `project` is the root node. It takes three parameters: `width`, `height`, and `fps` (Frames Per Second).

```javascript
project(1920, 1080, 60) {
    // scenes and components go here
}
```

## Scenes

A `scene` is a time-bounded segment of the timeline. Scenes execute sequentially. It takes a name and a duration.

```javascript
scene("intro", 5s) {
    // layers go here
}
```

## Layers

A `layer` is the fundamental renderable unit. Layers are stacked bottom-to-top (the last layer in a block renders on top).

```javascript
layer("background") {
    solid(#1a1a2e)
}
```

Every layer contains exactly one **Layer Content**, followed by optional properties (like position, animation, and children).

### Content Types

| Content Type | Syntax | Description |
|---|---|---|
| Solid | `solid(#hex)` | A solid color background. |
| Text | `text("String", font: "Inter", size: 48, color: #ffffff)` | Render text with automatic font management. |
| Image | `image("path/to.png")` | Load a static image (.png, .jpeg). |
| Spritesheet | `spritesheet("path/to.png", frameWidth: 64, frameHeight: 64, fps: 12)` | Animate tiles from a sheet image. |
| Video | `video("path/to.mp4", trim_start: 0s, trim_end: 5s)` | Load and play a video clip. |
| Audio | `audio("path/to.mp3", volume: 1.0)` | Play audio. Cannot be transformed visually. |
| TTS | `tts("Text to speak", "en-US-Standard-A")` | AI text-to-speech. Uses cloud orchestration. |
| AutoCaption| `autocaption("path/to.mp3", font: "Inter", size: 32)` | Automatically transcribe and layout animated words. |
| Shape | `shape(rect, fill: #ff0000, width: 100, height: 100)` | Primitive shapes (`rect`, `circle`, etc). |
| Component | `use("Name", prop: "value")` | Place an instantiated component block. |
| **Web** | `web("source", viewport: 800x600)` | **Render a web page (HTML/React/D3) as a layer.** |

### Web Scenes

The `web()` content type lets you embed live HTML content as a composited layer. This is ideal for data visualizations, interactive overlays, React components, D3 charts, Three.js scenes, and any web-based content.

**Syntax:**

```javascript
layer("dashboard") {
    web("web/chart.html", viewport: 1280x720)
    position(960, 540)
}
```

**Named arguments:**

| Argument | Type | Default | Description |
|---|---|---|---|
| `source` | `String` | (required) | Path to HTML file or URL |
| `viewport` | `WxH` | `1920x1080` | Viewport dimensions for the embedded browser |
| `mode` | `capture` \| `realtime` | `capture` | `capture` renders frame-by-frame; `realtime` uses live DOM |
| `wait_for` | `String` | — | CSS selector to wait for before capturing (e.g., `".chart-ready"`) |
| `variables` | `{ key: value }` | `{}` | Variables injected into the web page via `window.__vidra.vars` |

**Modes:**

- **`capture` mode** (default): The capture engine loads the page in a headless browser, advances the timeline frame-by-frame via the `__vidra_advance_frame` protocol, and rasterizes each frame into the compositing pipeline. This is pixel-perfect and deterministic.
- **`realtime` mode**: The browser player renders the web layer as an `<iframe>` overlay, synced to the timeline position. Ideal for interactive previews.

**Examples:**

```javascript
// D3 chart with data passed via variables
layer("revenue_chart") {
    web("web/revenue.html", viewport: 800x400, variables: { year: 2025, currency: "USD" })
    position(960, 400)
    animation(opacity, from: 0, to: 1, duration: 0.5s)
}

// React component in capture mode
layer("react_overlay") {
    web("web/ticker.html", viewport: 400x100, mode: capture, wait_for: ".loaded")
    position(960, 980)
}

// Interactive Three.js scene (realtime in browser player)
layer("3d_scene") {
    web("web/globe.html", viewport: 1920x1080, mode: realtime)
}
```

**Integrated mode (`@vidra/web-capture`):**

When building web content specifically for Vidra, use the `@sansavision/vidra-web-capture` npm package to access the capture bridge:

```tsx
import { useVidraScene } from '@sansavision/vidra-web-capture/react';

function AnimatedChart() {
    const { frame, time, fps, vars, emit } = useVidraScene();
    // Use frame/time to drive animations deterministically
    return <div>Frame: {frame}, Value: {vars.revenue}</div>;
}
```

The hook gracefully degrades when running outside the capture harness — `frame` defaults to 0, `time` uses `Date.now()`, and `emit()` is a no-op.

### Properties

Properties apply transformations and animations to layers.

*   `position(x, y)` sets the anchor position.
*   `scale(x, y)` adjusts size.
*   `animation(property, from: val, to: val, duration: time, easing: type)` animates the layer.
*   `effect(type, intensity)` applies a visual post-process effect (e.g., `blur`, `grayscale`, `invert`).

Color grading (LUT):

```javascript
layer("grade") {
    image("assets/shot.png")
    effect(lut, "assets/film.cube", 1.0)
}
```

Example:

```javascript
layer("logo") {
    image("assets/logo.png")
    position(960, 540)
    scale(1.5, 1.5)
    animation(opacity, from: 0, to: 1, duration: 1s, easing: ease-out)
    animation(position_y, from: 600, to: 540, duration: 2s, easing: ease-out)
    effect(blur, 10.0)
}
```

Spritesheet example:

```javascript
layer("spark") {
    spritesheet("assets/sparks.png", frameWidth: 64, frameHeight: 64, fps: 24)
    position(960, 540)
}
```

Notes:

- Asset paths can also be remote URLs (e.g. `image("https://...")`). The Rust CLI/dev server will download and cache them under `resources.cache_dir` automatically.

### Expression Animations

You can also drive an animation with an expression:

```javascript
layer("follow_mouse") {
    text("@mouse.x", size: 48)
    animation(x, expr: "@mouse.x", duration: 10.0)
    animation(y, expr: "@mouse.y", duration: 10.0)
}
```

Available expression variables:

- `t`: seconds since the animation started
- `p`: progress from 0 → 1 over the animation duration
- `T`: the animation duration in seconds
- `@mouse.x`, `@mouse.y`: mouse position in pixels (runtime; in non-interactive renders these default to `0`)

### 2.5D Transforms

Vidra supports simple planar 2.5D transforms to “tilt” a layer in 3D:

- `translateZ(z)`
- `rotateX(deg)`
- `rotateY(deg)`
- `perspective(distance)`

Example:

```javascript
layer("card") {
    image("assets/card.png")
    position(960, 540)
    anchor(0.5, 0.5)

    perspective(900)
    rotateY(25)
    translateZ(80)
}
```

You can also animate these via `animation(...)` by using properties:

- `translateZ`
- `rotateX`
- `rotateY`
- `perspective`

### Reactive Events

You can attach interactive handlers to a layer:

```javascript
layer("button") {
    solid(#ffffff)
    position(100, 100)

    @on click {
        set count = count + 1
    }
}
```

Notes:

- Reactive events are primarily for WASM previews right now.
- Hosts should call the exported WASM API `dispatch_click(irJson, frameIndex, x, y)`.
- Runtime state vars can be seeded/read via `set_state_var(name, value)` and `get_state_var(name)`.

## Components

Components are reusable blocks that encapsulate one or more layers, accepting props.

```javascript
component(TitleCard, text: String, color: Color) {
    layer("bg") {
        solid(color)
    }
    layer("text") {
        text(text, font: "Inter", size: 100, color: #ffffff)
        position(960, 540)
    }
}
```

To use a component:

```javascript
layer("scene1_title") {
    use("TitleCard", text: "Welcome!", color: #e94560)
}
```

### Slots

Components can accept children by declaring `slot()` in a layer.

```javascript
component(Container) {
    layer("wrapper") {
        solid(#000000)
        slot()
    }
}
```

Usage:

```javascript
use("Container") {
    layer("child1") { text("I am inside!") }
}
```

### Variants

Components can define variants to encapsulate specific styles or overrides.

```javascript
component(Button, label: String) {
    layer("base") {
        solid(#000000)
    }
    
    variant("primary") {
        layer("base") {
            solid(#e94560)
        }
    }
}

use("Button", label: "Click", variant: "primary")
```

## Brand Kits

References to brand assets can be passed directly into layout attributes using the `@brand` prefix. These are resolved at compile time from the workspace `vidra.config.toml` (or cloud metadata).

```javascript
layer("bg") {
    solid(@brand.colors.primary)
}

layer("heading") {
    text("Hello", font: @brand.fonts.heading, color: @brand.colors.text)
}
```

## Conditionals and Loops (Logic)

You can use basic logic nodes to toggle visibility or instance elements multiple times.

```javascript
if (show_disclaimer) {
    layer("disclaimer") {
        text("Terms and conditions apply.", size: 12)
        position(1920, 1060)
    }
} else {
    // ...
}
```

## Responsive Layouts (When / Aspect)

Vidra automatically handles multi-format rendering using `layout rules` and `when aspect()`.

```javascript
layout rules {
    when aspect(9:16) {
        layer("title") {
            position(540, 1500)
            scale(1.2, 1.2)
        }
    }
}
```

When rendering with `vidra render --targets 16:9,9:16`, the engine will compile the IR graph specifically overriding properties dynamically based on the target aspect ratio, ensuring perfect composition across formats inside the exact same project tree.
