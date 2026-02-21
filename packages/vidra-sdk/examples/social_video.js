import { 
  ProjectBuilder, 
  SceneBuilder, 
  LayerBuilder, 
  AnimationBuilder, 
  ColorUtils 
} from "../src/index.js";
import fs from "fs";

/**
 * Procedurally generates a Video and Audio combined clip using Vidra SDK layer management.
 */

// 1. Build a landscape Project
const project = new ProjectBuilder(1920, 1080, 24)
  .background(ColorUtils.hex("#0d0d0d"))
  .addAsset("Image", "logo", "assets/logo.png")
  .addAsset("Audio", "bg_music", "assets/lofi_beat.mp3");

// 2. Build 6 seconds total
const mainScene = new SceneBuilder("main_loop", 6.0);

// Add an atmospheric Solid Red Background 
const bgLayer = new LayerBuilder("solid_red", {
  Solid: { color: ColorUtils.hex("#7f1d1d") }
});
mainScene.addLayer(bgLayer.build());

// Inject Animated Text
const textLayer = new LayerBuilder("title", {
  Text: {
    text: "Dynamic Templates with Vidra",
    font_family: "Inter",
    font_size: 100,
    color: ColorUtils.hex("#ffffff") 
  }
})
  .position(960, 500)
  .addAnimation(
    new AnimationBuilder("ScaleX")
      .addKeyframe(0.0, 0.5, "EaseOut")
      .addKeyframe(1.5, 1.0, "Linear")
      .addKeyframe(4.5, 1.0, "EaseIn")
      .addKeyframe(6.0, 0.5, "EaseOut")
      .build()
  )
  .addAnimation(
    new AnimationBuilder("ScaleY")
      .addKeyframe(0.0, 0.5, "EaseOut")
      .addKeyframe(1.5, 1.0, "Linear")
      .addKeyframe(4.5, 1.0, "EaseIn")
      .addKeyframe(6.0, 0.5, "EaseOut")
      .build()
  );

mainScene.addLayer(textLayer.build());

const audioLayer = new LayerBuilder("bg_music_track", {
  Audio: {
    asset_id: "bg_music",
    trim_start: { seconds: 0.0 },
    trim_end: { seconds: 6.0 },
    volume: 0.5
  }
});
mainScene.addLayer(audioLayer.build());

project.addScene(mainScene.build());

const data = JSON.stringify(project.build(), null, 2);
fs.writeFileSync("./social_video.json", data);

console.log("Successfully generated dynamic social_video.json output block.");
console.log("To render locally, run:");
console.log("  vidra render ./social_video.json --output render.mp4");
