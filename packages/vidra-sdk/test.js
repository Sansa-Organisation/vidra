import { Project, Scene, Layer, Easing, hex } from "./dist/index.js";
import fs from "node:fs";

// ─── Build a video programmatically ─────────────────────────────────

const project = new Project(1920, 1080, 60)
    .background("#1a1a2e");

// Scene 1: Animated intro
const intro = new Scene("intro", 3.0);

intro.addLayers(
    new Layer("bg").solid("#1a1a2e"),

    new Layer("title")
        .text("Built with TypeScript SDK!", "Inter", 100, "#ffffff")
        .position(960, 540)
        .animate("opacity", 0, 1, 1.5, Easing.EaseOut),

    new Layer("subtitle")
        .text("@sansavision/vidra-sdk", "Inter", 40, "#00aaff")
        .position(960, 640)
        .animate("opacity", 0, 1, 1.0, Easing.EaseOut, 0.5)
        .animate("positionY", 680, 640, 1.0, Easing.CubicOut, 0.5),
);

project.addScene(intro);

// Scene 2: Feature showcase
const showcase = new Scene("features", 3.0);

showcase.addLayers(
    new Layer("bg2").solid("#0d1117"),

    new Layer("heading")
        .text("Programmatic Video", "Inter", 80, "#58a6ff")
        .position(960, 300),

    new Layer("bullet1")
        .text("✓ Fluent Builder API", "Inter", 48, "#ffffff")
        .position(960, 480)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.3),

    new Layer("bullet2")
        .text("✓ VidraScript Export", "Inter", 48, "#ffffff")
        .position(960, 560)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.6),

    new Layer("bullet3")
        .text("✓ CLI Rendering", "Inter", 48, "#ffffff")
        .position(960, 640)
        .animate("opacity", 0, 1, 0.5, Easing.EaseOut, 0.9),
);

project.addScene(showcase);

// ─── Output ─────────────────────────────────────────────────────────

// 1. Write the IR JSON
const json = project.toJSONString();
fs.writeFileSync("output/sdk_test.json", json);
console.log("✅ Wrote IR JSON → output/sdk_test.json");

// 2. Write the VidraScript DSL
const dsl = project.toVidraScript();
fs.writeFileSync("output/sdk_test.vidra", dsl);
console.log("✅ Wrote VidraScript → output/sdk_test.vidra");
console.log("\n── Generated VidraScript ──────────────────────");
console.log(dsl);

// 3. Render hint
console.log("── To render ──────────────────────────────────");
console.log("vidra render output/sdk_test.vidra --output output/sdk_test.mp4");
