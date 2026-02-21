# Issue 002 — Strict Image Decoding Proof

## How to verify

```bash
cd issues/002-strict-image-decoding/proof

# Create a JPEG file but save it with a .png extension
cp /path/to/any/photo.jpg mismatched.png

vidra render test.vidra --output result.mp4
```

**Before fix:** Crashes with `Format error decoding Png: Invalid PNG signature`.

**After fix:** The engine reads the raw bytes, detects the true format via magic bytes (`FF D8` = JPEG), decodes correctly, and renders the image without error.

## What changed

- `crates/vidra-render/src/image_loader.rs` — replaced `image::open(path)` (extension-based) with `image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()` (magic-byte-based).
