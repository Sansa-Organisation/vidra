//! Image loading module.
//! Decodes PNG, JPEG, WebP, and other formats into FrameBuffers.

use std::path::Path;

use vidra_core::frame::FrameBuffer;
use vidra_core::{PixelFormat, VidraError};

/// Load an image file and convert it to a FrameBuffer.
pub fn load_image(path: &Path) -> Result<FrameBuffer, VidraError> {
    let img = image::open(path).map_err(|e| {
        VidraError::asset(
            format!("failed to load image '{}': {}", path.display(), e),
            path,
        )
    })?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut fb = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    fb.data = rgba.into_raw();

    Ok(fb)
}

/// Load an image from raw bytes (e.g., from an embedded asset).
pub fn load_image_from_bytes(data: &[u8]) -> Result<FrameBuffer, VidraError> {
    let img = image::load_from_memory(data)
        .map_err(|e| VidraError::asset(format!("failed to decode image: {}", e), "<memory>"))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut fb = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    fb.data = rgba.into_raw();

    Ok(fb)
}

/// Resize an image frame buffer to fit within the given max dimensions,
/// preserving aspect ratio.
pub fn resize_to_fit(fb: &FrameBuffer, max_width: u32, max_height: u32) -> FrameBuffer {
    let scale_x = max_width as f64 / fb.width as f64;
    let scale_y = max_height as f64 / fb.height as f64;
    let scale = scale_x.min(scale_y).min(1.0); // Never upscale

    let new_width = (fb.width as f64 * scale) as u32;
    let new_height = (fb.height as f64 * scale) as u32;

    if new_width == fb.width && new_height == fb.height {
        return fb.clone();
    }

    // Simple nearest-neighbor resize for now
    // Phase 1: use bilinear or Lanczos resampling
    let mut resized = FrameBuffer::new(new_width, new_height, fb.format);
    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = (x as f64 / scale) as u32;
            let src_y = (y as f64 / scale) as u32;
            if let Some(pixel) = fb.get_pixel(src_x.min(fb.width - 1), src_y.min(fb.height - 1)) {
                resized.set_pixel(x, y, pixel);
            }
        }
    }

    resized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image_missing_file() {
        let result = load_image(Path::new("/nonexistent/image.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_resize_to_fit_no_upscale() {
        let fb = FrameBuffer::solid(100, 100, &vidra_core::Color::RED);
        let resized = resize_to_fit(&fb, 200, 200);
        // Should not upscale
        assert_eq!(resized.width, 100);
        assert_eq!(resized.height, 100);
    }

    #[test]
    fn test_resize_to_fit_downscale() {
        let fb = FrameBuffer::solid(200, 100, &vidra_core::Color::RED);
        let resized = resize_to_fit(&fb, 100, 100);
        // Should scale to 100x50 (preserving 2:1 aspect ratio)
        assert_eq!(resized.width, 100);
        assert_eq!(resized.height, 50);
    }
}
