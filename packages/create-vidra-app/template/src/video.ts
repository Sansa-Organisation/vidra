// ─── {{PROJECT_NAME}} — Video Project ───────────────────────────────
// Edit this file to describe your video programmatically.
// Run `npm run build:video` to compile → public/project.json
// Then the Player tab renders it live in the browser.

import { Project, Scene, Layer, Easing } from "@sansavision/vidra-sdk";

const project = new Project({ width: 1920, height: 1080, fps: 30 });

// ── Scene 1: Vibrant Intro ──────────────────────────────────────────
project.addScene(
  new Scene("intro", 4)
    .addLayer(new Layer("bg").solid("#1e3a5f"))

    // Big blue accent circle
    .addLayer(
      new Layer("circle")
        .shape("circle", { radius: 200, fill: "#58a6ff" })
        .position(960, 540)
        .animate("opacity", 0, 0.8, 1.5, Easing.EaseOut)
        .animate("ScaleX", 0.2, 1, 1.5, Easing.CubicOut)
        .animate("ScaleY", 0.2, 1, 1.5, Easing.CubicOut)
    )

    // Title text
    .addLayer(
      new Layer("title")
        .text("Welcome to Vidra", "Inter", 72, "#ffffff")
        .position(960, 480)
        .animate("opacity", 0, 1, 0.8, Easing.EaseOut, 0.5)
    )

    // Subtitle
    .addLayer(
      new Layer("subtitle")
        .text("Programmatic Video, Made Simple", "Inter", 32, "#b0d4f1")
        .position(960, 560)
        .animate("opacity", 0, 1, 0.8, Easing.EaseOut, 0.8)
    )

    // Accent bar
    .addLayer(
      new Layer("bar")
        .shape("rect", { width: 300, height: 6, radius: 3, fill: "#f0883e" })
        .position(960, 620)
        .animate("opacity", 0, 1, 0.6, Easing.EaseOut, 1.0)
    )
);

// ── Scene 2: Feature Showcase ───────────────────────────────────────
project.addScene(
  new Scene("features", 4)
    .addLayer(new Layer("bg2").solid("#0f2942"))

    .addLayer(
      new Layer("heading")
        .text("Build Videos with Code", "Inter", 56, "#ffffff")
        .position(960, 260)
        .animate("opacity", 0, 1, 0.6, Easing.EaseOut)
    )

    // Colored feature boxes
    .addLayer(
      new Layer("box1")
        .shape("rect", { width: 340, height: 160, radius: 16, fill: "#238636" })
        .position(380, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.2)
    )
    .addLayer(
      new Layer("box2")
        .shape("rect", { width: 340, height: 160, radius: 16, fill: "#1f6feb" })
        .position(960, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.4)
    )
    .addLayer(
      new Layer("box3")
        .shape("rect", { width: 340, height: 160, radius: 16, fill: "#8b5cf6" })
        .position(1540, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.6)
    )

    .addLayer(
      new Layer("label1")
        .text("TypeScript SDK", "Inter", 28, "#ffffff")
        .position(380, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.3)
    )
    .addLayer(
      new Layer("label2")
        .text("WASM Rendering", "Inter", 28, "#ffffff")
        .position(960, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.5)
    )
    .addLayer(
      new Layer("label3")
        .text("Web Scenes", "Inter", 28, "#ffffff")
        .position(1540, 500)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.7)
    )
);

// ── Scene 3: Call to Action ─────────────────────────────────────────
project.addScene(
  new Scene("outro", 3)
    .addLayer(new Layer("bg3").solid("#0d1b2a"))

    .addLayer(
      new Layer("cta_circle")
        .shape("circle", { radius: 120, fill: "#f0883e" })
        .position(960, 440)
        .animate("opacity", 0, 1, 1, Easing.EaseOut)
    )

    .addLayer(
      new Layer("cta")
        .text("Start Building", "Inter", 52, "#ffffff")
        .position(960, 580)
        .animate("opacity", 0, 1, 0.8, Easing.EaseOut, 0.3)
    )

    .addLayer(
      new Layer("url")
        .text("github.com/Sansa-Organisation/vidra", "Inter", 22, "#8b949e")
        .position(960, 650)
        .animate("opacity", 0, 1, 0.6, Easing.EaseOut, 0.6)
    )
);

// ── Output ──────────────────────────────────────────────────────────
const ir = project.toJSON();
console.log(JSON.stringify(ir, null, 2));
