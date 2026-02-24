use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use vidra_core::frame::FrameBuffer;
use vidra_core::VidraError;

/// Native APNG (Animated PNG) encoder using the `png` crate.
/// Lossless animation format ideal for stickers, UI animations, and high-quality shorts.
pub struct ApngEncoder;

impl ApngEncoder {
    /// Encode a sequence of RGBA frame buffers to an Animated PNG (APNG).
    ///
    /// # Arguments
    /// * `frames` - Ordered sequence of frame buffers
    /// * `width` - Frame width in pixels
    /// * `height` - Frame height in pixels
    /// * `fps` - Frames per second
    /// * `output_path` - Path for the output .apng / .png file
    /// * `loop_count` - Number of loops (0 = infinite)
    pub fn encode(
        frames: &[FrameBuffer],
        width: u32,
        height: u32,
        fps: f64,
        output_path: &Path,
        loop_count: Option<u32>,
    ) -> Result<(), VidraError> {
        if frames.is_empty() {
            return Err(VidraError::Encode("no frames to encode for APNG".into()));
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(output_path)
            .map_err(|e| VidraError::Encode(format!("failed to create APNG file: {}", e)))?;
        let writer = BufWriter::new(file);

        // Frame delay: numerator / denominator seconds
        // e.g., 30fps => each frame is 1/30 seconds
        let delay_num = 1u16;
        let delay_den = fps.round() as u16;

        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_animated(frames.len() as u32, loop_count.unwrap_or(0))
            .map_err(|e| VidraError::Encode(format!("failed to set APNG animation: {}", e)))?;
        encoder.set_frame_delay(delay_num, delay_den)
            .map_err(|e| VidraError::Encode(format!("failed to set APNG frame delay: {}", e)))?;

        let mut writer = encoder
            .write_header()
            .map_err(|e| VidraError::Encode(format!("failed to write APNG header: {}", e)))?;

        for (i, frame) in frames.iter().enumerate() {
            if frame.width != width || frame.height != height {
                return Err(VidraError::Encode(format!(
                    "frame {} has dimensions {}x{}, expected {}x{}",
                    i, frame.width, frame.height, width, height
                )));
            }

            // Set per-frame delay for consistent timing
            writer.set_frame_delay(delay_num, delay_den)
                .map_err(|e| VidraError::Encode(format!("failed to set delay on frame {}: {}", i, e)))?;

            writer.write_image_data(&frame.data)
                .map_err(|e| VidraError::Encode(format!("failed to write APNG frame {}: {}", i, e)))?;
        }

        writer.finish()
            .map_err(|e| VidraError::Encode(format!("failed to finalize APNG: {}", e)))?;

        tracing::info!(
            "Encoded {} frames to APNG at {} ({}x{} @ {}fps)",
            frames.len(),
            output_path.display(),
            width,
            height,
            fps,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apng_encode_empty_frames() {
        let result = ApngEncoder::encode(&[], 320, 240, 30.0, Path::new("/tmp/test.apng"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_apng_encode_solid_frames() {
        let mut frames = Vec::new();
        for i in 0..5 {
            let mut fb = FrameBuffer::new(4, 4, vidra_core::PixelFormat::Rgba8);
            for y in 0..4 {
                for x in 0..4 {
                    fb.set_pixel(x, y, [0, (i * 50) as u8, 255, 255]);
                }
            }
            frames.push(fb);
        }

        let out = std::env::temp_dir().join("vidra_test_apng.png");
        let result = ApngEncoder::encode(&frames, 4, 4, 10.0, &out, None);
        assert!(result.is_ok(), "APNG encode failed: {:?}", result.err());

        let meta = std::fs::metadata(&out).unwrap();
        assert!(meta.len() > 0);

        let _ = std::fs::remove_file(&out);
    }
}
