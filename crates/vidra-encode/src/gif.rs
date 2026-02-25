use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use vidra_core::frame::FrameBuffer;
use vidra_core::VidraError;

/// Native GIF encoder using the `image` crate.
/// Ideal for short clips, stickers, and social media content.
pub struct GifEncoder;

impl GifEncoder {
    /// Encode a sequence of RGBA frame buffers to an animated GIF.
    ///
    /// # Arguments
    /// * `frames` - Ordered sequence of frame buffers
    /// * `width` - Frame width in pixels
    /// * `height` - Frame height in pixels
    /// * `fps` - Frames per second (used to compute inter-frame delay)
    /// * `output_path` - Path for the output .gif file
    /// * `loop_count` - Number of loops (0 = infinite, None = infinite)
    pub fn encode(
        frames: &[FrameBuffer],
        width: u32,
        height: u32,
        fps: f64,
        output_path: &Path,
        loop_count: Option<u16>,
    ) -> Result<(), VidraError> {
        if frames.is_empty() {
            return Err(VidraError::Encode("no frames to encode for GIF".into()));
        }

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(output_path)
            .map_err(|e| VidraError::Encode(format!("failed to create GIF file: {}", e)))?;
        let writer = BufWriter::new(file);

        // Inter-frame delay in centiseconds (GIF unit)
        let delay_cs = ((100.0 / fps).round() as u16).max(2); // GIF minimum is ~2cs

        let mut encoder = image::codecs::gif::GifEncoder::new_with_speed(writer, 10);

        // Set repeat mode
        let repeat = match loop_count {
            None | Some(0) => image::codecs::gif::Repeat::Infinite,
            Some(n) => image::codecs::gif::Repeat::Finite(n),
        };
        encoder
            .set_repeat(repeat)
            .map_err(|e| VidraError::Encode(format!("failed to set GIF repeat: {}", e)))?;

        for (i, frame) in frames.iter().enumerate() {
            if frame.width != width || frame.height != height {
                return Err(VidraError::Encode(format!(
                    "frame {} has dimensions {}x{}, expected {}x{}",
                    i, frame.width, frame.height, width, height
                )));
            }

            let gif_frame = image::Frame::from_parts(
                image::RgbaImage::from_raw(width, height, frame.data.clone()).ok_or_else(|| {
                    VidraError::Encode(format!("invalid frame data at frame {}", i))
                })?,
                0,
                0,
                image::Delay::from_numer_denom_ms(delay_cs as u32 * 10, 1),
            );

            encoder.encode_frame(gif_frame).map_err(|e| {
                VidraError::Encode(format!("failed to encode GIF frame {}: {}", i, e))
            })?;
        }

        tracing::info!(
            "Encoded {} frames to GIF at {} ({}x{} @ {}fps, delay={}cs)",
            frames.len(),
            output_path.display(),
            width,
            height,
            fps,
            delay_cs,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_encode_empty_frames() {
        let result = GifEncoder::encode(&[], 320, 240, 30.0, Path::new("/tmp/test.gif"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_gif_encode_solid_frames() {
        let mut frames = Vec::new();
        for i in 0..5 {
            let mut fb = FrameBuffer::new(4, 4, vidra_core::PixelFormat::Rgba8);
            for y in 0..4 {
                for x in 0..4 {
                    fb.set_pixel(x, y, [255, (i * 50) as u8, 0, 255]);
                }
            }
            frames.push(fb);
        }

        let out = std::env::temp_dir().join("vidra_test_gif.gif");
        let result = GifEncoder::encode(&frames, 4, 4, 10.0, &out, None);
        assert!(result.is_ok(), "GIF encode failed: {:?}", result.err());

        // Verify a file was created and has content
        let meta = std::fs::metadata(&out).unwrap();
        assert!(meta.len() > 0);

        // Cleanup
        let _ = std::fs::remove_file(&out);
    }
}
