// ─── {{PROJECT_NAME}} — Video Project (SDK) ────────────────────────
//
// This file uses the Vidra SDK to build a video project programmatically.
// Run it with: npm run build:video
//
// It outputs the Vidra IR as JSON to stdout. To render to video,
// use the VidraScript file instead: npx @sansavision/vidra render video.vidra -o output.mp4

import {
  Project,
  Scene,
  Layer,
  Easing,
  hex,
} from "@sansavision/vidra-sdk";

// ── Build the project ───────────────────────────────────────────────

const project = new Project({ width: 1920, height: 1080, fps: 30 })
  // Scene 1: Intro
  .addScene(
    new Scene("intro", 4)
      // Dark gradient background
      .addLayer(new Layer("bg").solid("#0d1117"))

      // Main title — fades in and slides up
      .addLayer(
        new Layer("title")
          .text("Welcome to {{PROJECT_NAME}}", "Inter", 72, "#e6edf3")
          .position(960, 480)
          .animate("opacity", 0, 1, 0.8, Easing.EaseOut)
      )

      // Subtitle
      .addLayer(
        new Layer("subtitle")
          .text("Built with Vidra SDK", "Inter", 32, "#8b949e")
          .position(960, 560)
          .animate("opacity", 0, 1, 0.8, Easing.EaseOut, 0.3)
      )

      // Decorative shape
      .addLayer(
        new Layer("accent")
          .shape("rect", { width: 200, height: 4, radius: 2, fill: "#58a6ff" })
          .position(960, 620)
          .animate("opacity", 0, 1, 0.6, Easing.EaseOut, 0.6)
      )
  )

  // Scene 2: Content
  .addScene(
    new Scene("content", 4)
      .addLayer(new Layer("bg2").solid("#161b22"))

      .addLayer(
        new Layer("heading")
          .text("Programmatic Video", "Inter", 56, "#e6edf3")
          .position(960, 300)
          .animate("opacity", 0, 1, 0.6, Easing.EaseOut)
      )

      .addLayer(
        new Layer("bullet1")
          .text("✦ TypeScript SDK", "Inter", 36, "#58a6ff")
          .position(960, 450)
          .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.2)
      )

      .addLayer(
        new Layer("bullet2")
          .text("✦ GPU-accelerated rendering", "Inter", 36, "#58a6ff")
          .position(960, 520)
          .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.4)
      )

      .addLayer(
        new Layer("bullet3")
          .text("✦ Web scenes (React, D3, Three.js)", "Inter", 36, "#58a6ff")
          .position(960, 590)
          .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.6)
      )
  )

  // Scene 3: Outro
  .addScene(
    new Scene("outro", 3)
      .addLayer(new Layer("bg3").solid("#0d1117"))

      .addLayer(
        new Layer("cta")
          .text("Start building with Vidra", "Inter", 48, "#ffffff")
          .position(960, 500)
          .animate("opacity", 0, 1, 0.8, Easing.EaseOut)
      )

      .addLayer(
        new Layer("url")
          .text("github.com/Sansa-Organisation/vidra", "Inter", 24, "#8b949e")
          .position(960, 580)
          .animate("opacity", 0, 1, 0.6, Easing.EaseOut, 0.4)
      )
  );

// ── Output ──────────────────────────────────────────────────────────

const ir = project.build();
const json = JSON.stringify(ir, null, 2);

// Write to stdout (pipe to file: node src/video.js > project.json)
console.log(json);
