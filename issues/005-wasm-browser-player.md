---
type: feature
status: open
priority: p2
---

# Issue Title: WebAssembly (WASM) Player capabilities for Real-Time Browser Rendering

## Description
Currently, Vidra runs natively on desktop/server environments utilizing the OS GPU (`wgpu` via Metal/Vulkan/DX12).

To achieve parity with tools like Remotion and allow for real-time interactive video previews embedded directly in a web browser, we need to compile the core engine to WebAssembly (WASM) targeting WebGL2/WebGPU. 

This enables developers to pass live JavaScript state (e.g., fetched weather APIs, interactive React state, localized user data) directly as properties into the `vidra-render` engine running client-side, enabling dynamic video templates rendering at 60fps in the browser `<canvas>` without burning server compute.

## Reproducibility
*Feature Idea Workflow:*
1. Developer installs a future package: `npm install @sansavision/vidra-player`
2. Usage in a standard React app:
   ```tsx
   import { VidraPlayer } from "@sansavision/vidra-player";
   import { useState, useEffect } from "react";

   export default function App() {
       const [weather, setWeather] = useState("Loading...");
       
       useEffect(() => {
           fetch("/api/weather").then(res => res.json()).then(data => setWeather(data.temp));
       }, []);

       return (
           <VidraPlayer 
               src="main.vidra" 
               props={{ temperature: weather }}
               width={1920}
               height={1080}
               fps={60}
               controls={true}
           />
       );
   }
   ```
3. When `weather` updates, the player instantly redraws the frame utilizing the browser's GPU.

## Context & Environment
- **Vidra CLI Version:** 0.1.0 (Phase 3 milestone)
- **Scope:** New `vidra-player` (or `vidra-wasm`) workspace package

## Proposed Resolution
1. **Engine adjustments:** Ensure `vidra-lang` and `vidra-render` correctly build via `wasm-pack`. `wgpu` already has first-class WebGL/WebGPU support, so the port should be straight forward.
2. **Expose WASM bindings:** Expose functions to parse `.vidra` content, inject JSON variables into the IR context, and step the `Compositor` to an explicit frame/time.
3. **Web SDK:** Create an NPM wrapper (e.g., `@sansavision/vidra-player`) providing React/Vue bindings around the injected `wgpu` canvas context, managing requestAnimationFrame and syncing HTML5 video controls to the engine timeline.
