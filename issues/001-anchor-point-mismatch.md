---
type: bug
status: open
priority: p1
---

# Issue Title: Bounding box anchor coordinates default to top-left instead of center (Doc Mismatch)

## Description
There is a critical mismatch between what the Vidra documentation states and what the `vidra-render` GPU engine actually executes regarding layer positioning. The documentation indicates that `position(x, y)` anchors elements by their **center** (`0.5, 0.5`). 

However, visual observation of the rendered MP4s (e.g., `cinematic_mcp_showcase.mp4`) shows that placing text and images at `position(960, 540)` on a 1920x1080 canvas causes them to render originating from the exact center and overflowing entirely into the bottom-right quadrant. This confirms the engine is anchoring at the top-left `(0,0)` pixel of the element's bounding box.

## Reproducibility
1. Create a `main.vidra` file:
   ```javascript
   project(1920, 1080, 60) {
       scene("test", 1s) {
           layer("title") {
               text("CENTERED TEXT", font: "Inter", size: 100, color: #ffffff)
               position(960, 540)
           }
       }
   }
   ```
2. Render: `vidra render main.vidra`

**Expected Behavior:** The text bounding box should be perfectly centered in the middle of the video frame.
**Actual Behavior:** The text starts exactly at the middle of the frame and flows dynamically to the right and down.

## Context & Environment
- **Vidra CLI Version:** 0.1.0
- **OS:** macOS (Apple Silicon) / Windows
- **Component:** `vidra-render` / WebGPU compositor shader `compositor.wgsl`

## Proposed Resolution
There are two ways to resolve this:
1. **Engine Level (Preferred):** Modify the WGSL vertex shader logic in `vidra-render/src/compositor.wgsl` and the affine transform matrix logic in `vidra-core/src/math.rs` to offset positions by `-width/2` and `-height/2` of the element boundary.
2. **Docs Level:** Update the documentation and standard libraries to clarify that top-left is standard, and introduce an `anchor(x, y)` primitive (e.g., `anchor(0.5, 0.5)`) to allow shifting the bounding box.
