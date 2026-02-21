---
type: qol
status: closed
priority: p2
---

# Issue Title: Strict file extension decoding crashes compilation on magic byte mismatch

## Description
When integrating AI pipelines (such as the MCP server prompting image generation tools), images might get saved to disk with a mismatched extension (e.g., saved as `file.png` but the actual generated binary data is encoded as a JPEG).

Vidra's asset loader uses the explicit file extension as the source of truth for the decoding logic, which crashes with `Invalid PNG signature`.

## Reproducibility
1. Save a valid JPEG file but name it `asset.png`.
2. Reference it in VidraScript:
   ```javascript
   layer("bg") {
       image("asset.png")
   }
   ```
3. Run `vidra render`.

**Expected Behavior:** The engine detects the file stream is actually a JPEG via its magic bytes (`FF D8`) and loads it smoothly.
**Actual Behavior:** Hard crash parsing the asset: `Format error decoding Png: Invalid PNG signature. ("asset.png")`.

## Context & Environment
- **Vidra CLI Version:** 0.1.0
- **Component:** `vidra-render/src/image_loader.rs`

## Proposed Resolution
In `vidra-render` crate, rely on the `image` crate's format-guessing heuristic rather than extension forcing:
```rust
// Current (approximate guess):
// image::open(&path)

// Proposed Fix:
let img_bytes = std::fs::read(&path)?;
let cursor = std::io::Cursor::new(img_bytes);
let img = image::ImageReader::new(cursor)
    .with_guessed_format()?
    .decode()?;
```
This guarantees that regardless of the URL or file name extension, as long as the binary header represents a valid image, it will decode without crashing.
