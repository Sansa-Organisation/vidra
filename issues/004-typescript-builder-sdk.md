---
type: feature
status: open
priority: p1
---

# Issue Title: TypeScript / JavaScript Programmatic API for Vidra 

## Description
Currently, users must learn the custom domain-specific language (`VidraScript` / `.vidra`) to instruct the engine. While the DSL is concise, many front-end and full-stack developers are already deeply familiar with TypeScript (and its rich autocomplete ecosystems).

We want to allow users to build Vidra videos using a standard Node.js `@sansavision/vidra-sdk` package. 

They write pure TypeScript, the package intercepts that builder pattern, generates the identical `IR` JSON payload, and pipes it directly into the Engine's WASM or rust runtime.

## Reproducibility
*Feature Idea Workflow:*
1. Developer installs `npm install @sansavision/vidra-sdk`
2. Developer writes an `index.ts` file:
   ```typescript
   import { Project, Scene, Layer, Solid, Text, Easing } from "@sansavision/vidra-sdk";

   const p = new Project(1920, 1080, 60);
   
   const scene1 = new Scene("intro", 3.0); // 3 seconds
   
   const bg = new Layer("bg").setContent(new Solid("#1a1a2e"));
   
   const title = new Layer("title")
       .setContent(new Text("Written in TS!", "Inter", 100, "#ffffff"))
       .setPosition(960, 540)
       .animate("opacity", 0, 1, 1.0, Easing.EaseOut);
       
   scene1.addLayers(bg, title);
   p.addScene(scene1);

   // Export to IR or render natively
   p.render("output.mp4");
   ```
3. Run `npm run start` and get the video.

## Context & Environment
- **Vidra CLI Version:** 0.1.0
- **Scope:** `@sansavision/vidra-sdk` package

## Proposed Resolution
We already have the scaffold logic started inside `packages/vidra-sdk`.
1. Expand the TypeScript typings (`src/types.ts`) to exactly mirror the JSON output of the `crates/vidra-ir` definition in Rust.
2. Build utility functions so that `project.toJSON()` outputs a 1:1 match with the AST compiler's IR payload.
3. Once we have a 1:1 match, add a utility script `vidra.compileToTS(file.vidra)` or vice versa, enabling seamless translation between the raw DSL format and the TypeScript builder code, acting as a two-way street for AI generation and human-readable code!
