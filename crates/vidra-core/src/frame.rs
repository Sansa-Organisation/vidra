use serde::{Deserialize, Serialize};

/// Pixel format of a frame buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 8-bit RGBA (4 bytes per pixel).
    Rgba8,
    /// 8-bit RGB (3 bytes per pixel, no alpha).
    Rgb8,
}

impl PixelFormat {
    /// Bytes per pixel for this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgba8 => 4,
            PixelFormat::Rgb8 => 3,
        }
    }
}

/// A single video frame as a raw pixel buffer.
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// Raw pixel data.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format.
    pub format: PixelFormat,
}

impl FrameBuffer {
    /// Create a new frame buffer filled with zeros (transparent black).
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        let size = (width as usize) * (height as usize) * format.bytes_per_pixel();
        Self {
            data: vec![0u8; size],
            width,
            height,
            format,
        }
    }

    /// Create a frame buffer filled with a solid color.
    pub fn solid(width: u32, height: u32, color: &crate::Color) -> Self {
        let format = PixelFormat::Rgba8;
        let pixel = color.to_rgba8();
        let pixel_count = (width as usize) * (height as usize);
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&pixel);
        }
        Self {
            data,
            width,
            height,
            format,
        }
    }

    /// Total number of pixels.
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// Total byte size of the pixel data.
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    /// Get the RGBA value at a pixel coordinate. Returns None if out of bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = ((y as usize) * (self.width as usize) + (x as usize)) * bpp;
        match self.format {
            PixelFormat::Rgba8 => Some([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ]),
            PixelFormat::Rgb8 => Some([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                255,
            ]),
        }
    }

    /// Set the RGBA value at a pixel coordinate. No-op if out of bounds.
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = ((y as usize) * (self.width as usize) + (x as usize)) * bpp;
        match self.format {
            PixelFormat::Rgba8 => {
                self.data[offset] = rgba[0];
                self.data[offset + 1] = rgba[1];
                self.data[offset + 2] = rgba[2];
                self.data[offset + 3] = rgba[3];
            }
            PixelFormat::Rgb8 => {
                self.data[offset] = rgba[0];
                self.data[offset + 1] = rgba[1];
                self.data[offset + 2] = rgba[2];
            }
        }
    }

    /// Apply an alpha mask to this layer. Pixels outside the mask become transparent.
    pub fn apply_mask(&mut self, mask: &FrameBuffer, ox: i32, oy: i32) {
        if self.format != PixelFormat::Rgba8 || mask.format != PixelFormat::Rgba8 { return; }
        
        let start_y = std::cmp::max(0, oy);
        let end_y = std::cmp::min(self.height as i32, oy + mask.height as i32);
        let start_x = std::cmp::max(0, ox);
        let end_x = std::cmp::min(self.width as i32, ox + mask.width as i32);
        
        for y in 0..(self.height as i32) {
            for x in 0..(self.width as i32) {
                let dst_idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
                
                if x >= start_x && x < end_x && y >= start_y && y < end_y {
                    let mask_x = (x - ox) as u32;
                    let mask_y = (y - oy) as u32;
                    let mask_idx = ((mask_y as usize) * (mask.width as usize) + (mask_x as usize)) * 4;
                    // Multiply existing alpha by mask alpha or luminance. Here we use the mask alpha.
                    let mask_a = mask.data[mask_idx + 3] as f32 / 255.0;
                    let current_a = self.data[dst_idx + 3] as f32;
                    self.data[dst_idx + 3] = (current_a * mask_a) as u8;
                } else {
                    self.data[dst_idx + 3] = 0;
                }
            }
        }
    }

    /// Alpha-composite `src` on top of `self` at position (dx, dy).
    /// Uses highly optimized SIMD-friendly integer math for auto-vectorization.
    pub fn composite_over(&mut self, src: &FrameBuffer, dx: i32, dy: i32) {
        if self.format != PixelFormat::Rgba8 || src.format != PixelFormat::Rgba8 {
            // Fallback or ignore for unsupported formats in this fast path
            return;
        }

        let dst_width = self.width as i32;
        let dst_height = self.height as i32;
        
        let mut start_y = 0;
        let mut end_y = src.height as i32;
        let mut start_x = 0;
        let mut end_x = src.width as i32;

        if dy < 0 { start_y = -dy; }
        if dy + end_y > dst_height { end_y = dst_height - dy; }
        if dx < 0 { start_x = -dx; }
        if dx + end_x > dst_width { end_x = dst_width - dx; }

        if start_x >= end_x || start_y >= end_y {
            return;
        }

        let src_stride = (src.width * 4) as usize;
        let dst_stride = (self.width * 4) as usize;

        for sy in start_y..end_y {
            let dst_y = dy + sy;
            let src_row_start = (sy as usize * src_stride) + (start_x as usize * 4);
            let dst_row_start = (dst_y as usize * dst_stride) + ((dx + start_x) as usize * 4);
            let len = (end_x - start_x) as usize * 4;

            let src_slice = &src.data[src_row_start..src_row_start + len];
            let dst_slice = &mut self.data[dst_row_start..dst_row_start + len];

            // 4 bytes per pixel loop (auto-vectorizes well)
            for (s, d) in src_slice.chunks_exact(4).zip(dst_slice.chunks_exact_mut(4)) {
                let sa = s[3] as u32;
                if sa == 0 {
                    continue;
                }
                if sa == 255 {
                    d.copy_from_slice(s);
                    continue;
                }

                let da = d[3] as u32;
                let inv_sa = 255 - sa;
                let out_a = sa + ((da * inv_sa) / 255);
                
                if out_a == 0 { continue; }

                let s_r = s[0] as u32;
                let s_g = s[1] as u32;
                let s_b = s[2] as u32;
                let d_r = d[0] as u32;
                let d_g = d[1] as u32;
                let d_b = d[2] as u32;

                let out_r = (s_r * sa * 255 + d_r * da * inv_sa) / (out_a * 255);
                let out_g = (s_g * sa * 255 + d_g * da * inv_sa) / (out_a * 255);
                let out_b = (s_b * sa * 255 + d_b * da * inv_sa) / (out_a * 255);

                d[0] = out_r as u8;
                d[1] = out_g as u8;
                d[2] = out_b as u8;
                d[3] = out_a as u8;
            }
        }
    }
}

/// Represents a frame in a video timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Frame {
    /// Zero-based frame index.
    pub index: u64,
}

impl Frame {
    pub fn new(index: u64) -> Self {
        Self { index }
    }

    /// Convert a frame index to a timestamp given a frame rate.
    pub fn to_timestamp(&self, fps: f64) -> crate::Timestamp {
        crate::Timestamp::from_seconds(self.index as f64 / fps)
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Frame({})", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_frame_buffer_new() {
        let fb = FrameBuffer::new(1920, 1080, PixelFormat::Rgba8);
        assert_eq!(fb.width, 1920);
        assert_eq!(fb.height, 1080);
        assert_eq!(fb.byte_size(), 1920 * 1080 * 4);
        assert_eq!(fb.pixel_count(), 1920 * 1080);
    }

    #[test]
    fn test_frame_buffer_solid() {
        let fb = FrameBuffer::solid(2, 2, &Color::RED);
        assert_eq!(fb.get_pixel(0, 0), Some([255, 0, 0, 255]));
        assert_eq!(fb.get_pixel(1, 1), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_frame_buffer_get_set_pixel() {
        let mut fb = FrameBuffer::new(10, 10, PixelFormat::Rgba8);
        fb.set_pixel(5, 5, [128, 64, 32, 255]);
        assert_eq!(fb.get_pixel(5, 5), Some([128, 64, 32, 255]));
    }

    #[test]
    fn test_frame_buffer_out_of_bounds() {
        let fb = FrameBuffer::new(10, 10, PixelFormat::Rgba8);
        assert_eq!(fb.get_pixel(10, 0), None);
        assert_eq!(fb.get_pixel(0, 10), None);
    }

    #[test]
    fn test_composite_over_opaque() {
        let mut dst = FrameBuffer::solid(4, 4, &Color::BLUE);
        let src = FrameBuffer::solid(2, 2, &Color::RED);
        dst.composite_over(&src, 1, 1);
        // Composited area should be red
        assert_eq!(dst.get_pixel(1, 1), Some([255, 0, 0, 255]));
        assert_eq!(dst.get_pixel(2, 2), Some([255, 0, 0, 255]));
        // Non-composited area should still be blue
        assert_eq!(dst.get_pixel(0, 0), Some([0, 0, 255, 255]));
    }

    #[test]
    fn test_composite_over_transparent() {
        let mut dst = FrameBuffer::solid(4, 4, &Color::WHITE);
        let src = FrameBuffer::new(2, 2, PixelFormat::Rgba8); // all transparent
        dst.composite_over(&src, 0, 0);
        // Should remain white
        assert_eq!(dst.get_pixel(0, 0), Some([255, 255, 255, 255]));
    }

    #[test]
    fn test_composite_over_semi_transparent() {
        let mut dst = FrameBuffer::solid(2, 2, &Color::WHITE);
        let mut src = FrameBuffer::new(1, 1, PixelFormat::Rgba8);
        src.set_pixel(0, 0, [255, 0, 0, 128]); // semi-transparent red

        dst.composite_over(&src, 0, 0);

        let pixel = dst.get_pixel(0, 0).unwrap();
        // Red should be blended with white — result should be pinkish
        assert!(pixel[0] > 200); // high red
        assert!(pixel[1] > 50 && pixel[1] < 200); // some green from white
        assert!(pixel[2] > 50 && pixel[2] < 200); // some blue from white
    }

    #[test]
    fn test_frame_to_timestamp() {
        let frame = Frame::new(30);
        let ts = frame.to_timestamp(30.0);
        assert!((ts.as_seconds() - 1.0).abs() < 0.001);
    }
}
