# Issue 001 — Anchor Point Mismatch Proof

## How to verify

```bash
cd issues/001-anchor-point-mismatch/proof
vidra render test.vidra --output result.mp4
```

**Before fix:** The text "CENTERED" starts at pixel (960, 540) and overflows into the bottom-right quadrant.

**After fix:** The text bounding box is offset by `(-width*0.5, -height*0.5)` using the layer's `transform.anchor` (default `0.5, 0.5`), so the word appears visually centered in the frame.

## What changed

- `crates/vidra-render/src/pipeline.rs` — all three compositor call-sites (`render_frame`, `inspect_frame_bounds`, child layers) now subtract `(layer_buf.width * anchor.x, layer_buf.height * anchor.y)` from the raw position before blitting.
