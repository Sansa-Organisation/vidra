import { ProjectBuilder, SceneBuilder, LayerBuilder, AnimationBuilder, ColorUtils } from "./src/index.js";
import fs from "fs";

// Build a simple project programmatically
const p = new ProjectBuilder(1920, 1080, 30)
  .background(ColorUtils.hex("#1a1a1a"));

const s = new SceneBuilder("intro", 5.0);

const textLayer = new LayerBuilder("title", {
  Text: {
    text: "Hello from Vidra SDK!",
    font_family: "Inter",
    font_size: 150,
    color: ColorUtils.hex("#ffffff")
  }
})
  .position(960, 540)
  .addAnimation(
    new AnimationBuilder("Opacity")
      .addKeyframe(0, 0, "Linear")
      .addKeyframe(2.0, 1.0, "EaseOut")
      .build()
  );

s.addLayer(textLayer.build());
p.addScene(s.build());

const json = JSON.stringify(p.build(), null, 2);
fs.writeFileSync("sdksample.json", json);
console.log("Successfully wrote sdksample.json");
