import { Project, Scene, Layer, Easing, hex } from "./dist/index.js";
import fs from "node:fs";

const project = new Project(1920, 1080, 60)
    .background("#1a1a2e");

const s1 = new Scene("intro", 3.0);

s1.addLayers(
    new Layer("bg").solid("#1a1a2e"),
    new Layer("w").web("http://localhost:3000", { viewport_width: 1280, viewport_height: 720, mode: "Realtime", wait_for: ".loaded" })
);

project.addScene(s1);

fs.mkdirSync("output", { recursive: true });

const json = project.toJSONString();
fs.writeFileSync("output/sdk_test_web.json", json);
console.log("✅ Wrote IR JSON");

const dsl = project.toVidraScript();
fs.writeFileSync("output/sdk_test_web.vidra", dsl);
console.log("✅ Wrote VidraScript =>\n" + dsl);
