# @sansavision/vidra-sdk

The official TypeScript SDK for the **Vidra** video engine.

Build video compositions programmatically using a fluent, chainable API. The SDK generates Vidra IR JSON and VidraScript DSL, which the engine compiles and renders via GPU acceleration.

## Installation

```bash
npm install @sansavision/vidra-sdk
```

## Quick Start

```typescript
import { Project, Scene, Layer, Easing } from "@sansavision/vidra-sdk";

// 1920x1080 @ 60fps
const project = new Project(1920, 1080, 60);

const intro = new Scene("intro", 3.0);
intro.addLayers(
    new Layer("bg").solid("#1a1a2e"),
    new Layer("title")
        .text("Hello, Vidra!", "Inter", 100, "#ffffff")
        .position(960, 540)
        .animate("opacity", 0, 1, 1.5, Easing.EaseOut),
    new Layer("subtitle")
        .text("Built with TypeScript", "Inter", 40, "#58a6ff")
        .position(960, 640)
        .animate("opacity", 0, 1, 1.0, Easing.EaseOut, 0.5)
        .animate("positionY", 680, 640, 1.0, Easing.CubicOut, 0.5),
);
project.addScene(intro);

// Output as VidraScript
console.log(project.toVidraScript());

// Output as IR JSON
console.log(project.toJSONString());

// Render directly to MP4 (requires vidra CLI installed)
await project.render("output.mp4");
```

## Output Modes

| Method | Description |
|--------|-------------|
| `project.toJSON()` | Returns the IR as a JavaScript object |
| `project.toJSONString()` | Returns the IR as a JSON string |
| `project.toVidraScript()` | Returns a valid `.vidra` DSL string |
| `project.render("out.mp4")` | Writes a temp `.vidra` file and shells to `vidra render` |

## Layer Content Types

```typescript
// Solid color
new Layer("bg").solid("#1a1a2e")

// Text
new Layer("title").text("Hello!", "Inter", 100, "#ffffff")

// Image (requires asset registration)
new Layer("photo").image("photo-01")

// Video
new Layer("clip").video("clip-01", 0, 10) // trimStart=0, trimEnd=10

// Audio
new Layer("music").audio("bgm-01", 0.8) // volume=0.8

// Shape
new Layer("box").shape("rect", { width: 400, height: 200, radius: 12, fill: "#ff0000" })
new Layer("dot").shape("circle", { radius: 50, fill: "#00ff00" })

// Text-to-Speech
new Layer("narration").tts("Welcome to Vidra", "en-US")
```

## Transforms & Animations

```typescript
new Layer("hero")
    .text("Animate me!", "Inter", 80, "#ffffff")
    .position(960, 540)      // x, y
    .scale(1.5)              // uniform scale
    .scale(2, 1)             // x, y scale
    .rotation(45)            // degrees
    .opacity(0.8)            // 0..1
    .anchor(0.5, 0.5)        // normalized anchor point
    .animate("opacity", 0, 1, 1.0, Easing.EaseOut)
    .animate("positionY", 600, 540, 0.8, Easing.CubicOut, 0.2) // with delay
```

### Available Easings

`Easing.Linear` · `Easing.EaseIn` · `Easing.EaseOut` · `Easing.EaseInOut` · `Easing.CubicIn` · `Easing.CubicOut` · `Easing.CubicInOut` · `Easing.Step`

## Effects

```typescript
new Layer("blurred").solid("#000").blur(8)
new Layer("shadow").text("Drop", "Inter", 60, "#fff")
    .dropShadow(4, 4, 10, "#000000")
```

## Assets

```typescript
project.addAsset("Image", "bg-img", "/assets/background.png", "Background");
project.addAsset("Video", "clip-01", "/assets/intro.mp4", "Intro Clip");
project.addAsset("Audio", "bgm", "/assets/music.mp3", "Background Music");
```

## Features

- **Fluent API**: Chainable methods — `new Layer("x").text(...).position(...).animate(...)`
- **Type-safe**: Full TypeScript types for all IR structs
- **Zero dependencies**: Pure TypeScript, no runtime dependencies
- **Dual output**: Generate both VidraScript DSL and IR JSON
- **Direct rendering**: Call `.render()` to produce MP4 via the Vidra CLI
- **Browser compatible**: Works in Node.js *and* the browser (via `@sansavision/vidra-player`)
