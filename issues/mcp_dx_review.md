# Vidra MCP & DX/UX Review

This document summarizes the developer experience (DX), user experience (UX), and Model Context Protocol (MCP) integrations based on our latest tests and end-to-end sandbox workflows.

## 1. The Good (Strengths & Wins)

*   **Blazing Fast Execution**: Compiling the AST to IR takes less than a millisecond. Render speeds hit 250+ FPS at 1080p and ~80 FPS at 4K. The tight Rust architecture provides an incredible experience natively.
*   **Clear Error Formats**: The compiler surfaces great errors when an unknown property or easing curve is provided (e.g., `unknown easing function: spring`). This helps LLMs quickly self-correct.
*   **Deterministic Workflows**: AI workflows rely heavily on predictable infrastructure. Because the IR is explicit and robust, an LLM generating scenes knows exactly how its text outputs translate to frames. 
*   **Easy Deployment**: Utilizing the `optionalDependencies` npm wrapper logic means any developer with Node (`bunx`, `npx`) can trigger the engine instantly without messy Rust/Clang dependencies.

## 2. Performance Gaps & System Bottlenecks

*   **Video Encoding Dominates Execution Time**: While rendering 4K frames took ~3.4 seconds, dragging it through FFmpeg took ~10.3s. *Opportunity*: Pipe frames directly to hardware encoders (NVENC/VideoToolbox) or parallelize frame dispatch to FFmpeg.
*   **Cold Starts & Magic Byte Loading**: Loading assets sequentially dynamically block the rendering hot path right before initialization.

## 3. Developer / AI Quality of Life (QoL) Friction

Several friction points emerged specifically from the perspective of an LLM or a new developer learning VidraScript:

### A. Anchor Points Misalignment with Documentation
According to the documentation (`docs/llm.md`), the default anchor for `position(x, y)` is the **center** `(width/2, height/2)`. However, rendered tests visually reveal that the engine uses the **top-left (0,0)** coordinate as the origin for both images and text bounding boxes. Placing an item at `(960, 540)` in a 1080p canvas anchors the item into the bottom right quadrant.
*   **Fix**: Update the engine to use center anchoring (which is the modern standard for motion graphics) or explicitly document and provide a `anchor(center, center)` property.

### B. Rigid Image Format Loading
When connecting LLM Image Generation APIs to Vidra, models often save files as `assets/file.png` but with JPEG magic bytes (or vice versa). Vidra panics (`Format error decoding Png: Invalid PNG signature`) and hard-fails if the file extension doesn't match the magic bytes perfectly. 
*   **Fix**: Auto-detect image formats using magic bytes (the `image` crate supports this via `ImageReader::new(Cursor::new(&bytes)).with_guessed_format()`) instead of relying solely on the file extension.

### C. Syntax "Guessing" for Easings / Properties
The language enforces strict casing (e.g., `easeOut` instead of `ease-out`, `positionY` instead of `position_y`). 
*   **Fix**: Introduce a more resilient parser that standardizes snake_case, kebab-case, and camelCase automatically for enum/property matching. Furthermore, the compiler error could suggest alternatives: `error: unknown easing function 'ease-out'. Did you mean 'easeOut'?`

---

## Next Steps

We will use the new `ISSUE_TEMPLATE.md` to track these discoveries systematically and dispatch them to the engineering backlog.
