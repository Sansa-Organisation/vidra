# @vidra/sdk

The official TypeScript SDK for the **Vidra** Video Engine.

Vidra is an AI-native, high-performance video infrastructure platform that allows you to define video compositions programmatically. This SDK lets you directly generate, configure, and serialize the Vidra Intermediate Representation (IR), which the `vidra` engine then natively compiles and renders via GPU hardware acceleration.

## Installation

```bash
npm install @vidra/sdk
```

## Setup & Quickstart

Import the core Builders and configure your project timeline dynamically using standard ES modules. 
Then, serialize your project state directly to the filesystem for Vidra to ingest natively.

### 1. Build a simple Title

```javascript
import { 
  ProjectBuilder, 
  SceneBuilder, 
  LayerBuilder, 
  AnimationBuilder, 
  ColorUtils 
} from "@vidra/sdk";
import fs from "fs";

// Initialize Project (1080p @ 30fps)
const project = new ProjectBuilder(1920, 1080, 30)
  .background(ColorUtils.hex("#1a1a1a"));

// Initialize a 5-second scene named "main"
const scene = new SceneBuilder("main", 5.0);

// Add a Text Layer
const titleText = new LayerBuilder("title", {
  Text: {
    text: "Vidra Generated Programmatically",
    font_family: "Inter",
    font_size: 110,
    color: ColorUtils.rgba(255, 255, 255, 255)
  }
})
  .position(960, 540)
  // Animate the opacity fading in across 2 seconds
  .addAnimation(
    new AnimationBuilder("Opacity")
      .addKeyframe(0.0, 0.0, "Linear")
      .addKeyframe(2.0, 1.0, "EaseOut")
      .build()
  );

// Compile the Scene
scene.addLayer(titleText.build());
project.addScene(scene.build());

// Dump the generated Vidra IR to disk
fs.writeFileSync("output.json", JSON.stringify(project.build(), null, 2));
```

### 2. Rendering Output

Pass the generated `output.json` directly into the fast `vidra` compiler native CLI:

```bash
vidra render output.json --output ./final_video.mp4
```

## Features

- **Object-Oriented API:** Exposes clean Builder patterns for Projects, Scenes, Layers, and Animations. 
- **Type-safe Typescript:** Full TS intellisense auto-completion mirroring mapping to the native `vidra-core` Rust structs.
- **Dependency-Free JSON Rendering:** The SDK outputs raw, standardized JSON payload configs so it is `Platform Agnostic` (NodeJS, V8 Isolates, Browser edge-workers such as Cloudflare Workers natively compatible).
- **Fully Supports Vidra IR:** Includes primitives to bind media tracks such as `Image`, `Text`, `Video`, `Audio`, `Shape`, and internal `Assets`.
