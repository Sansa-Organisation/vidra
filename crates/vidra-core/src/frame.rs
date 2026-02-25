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

    /// Alpha-composite `src` onto `self` by projecting the source rectangle into an arbitrary
    /// quad on the destination.
    ///
    /// `dst_corners` are in pixel coordinates, ordered: top-left, top-right, bottom-right, bottom-left.
    ///
    /// This is a CPU fallback intended for 2.5D transforms (perspective/tilt). It uses inverse
    /// mapping + bilinear sampling.
    pub fn composite_over_projected(&mut self, src: &FrameBuffer, dst_corners: [[f64; 2]; 4]) {
        if self.format != PixelFormat::Rgba8 || src.format != PixelFormat::Rgba8 {
            return;
        }
        if src.width == 0 || src.height == 0 || self.width == 0 || self.height == 0 {
            return;
        }

        let w = src.width as f64;
        let h = src.height as f64;

        let src_pts = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];

        let Some(h_mat) = homography_from_points(src_pts, dst_corners) else {
            return;
        };
        let Some(inv) = invert_3x3(h_mat) else {
            return;
        };

        let mut min_x = dst_corners[0][0];
        let mut max_x = dst_corners[0][0];
        let mut min_y = dst_corners[0][1];
        let mut max_y = dst_corners[0][1];
        for p in &dst_corners[1..] {
            min_x = min_x.min(p[0]);
            max_x = max_x.max(p[0]);
            min_y = min_y.min(p[1]);
            max_y = max_y.max(p[1]);
        }

        // Expand a tiny bit to account for rounding.
        let min_x = (min_x.floor() as i32).saturating_sub(1);
        let max_x = (max_x.ceil() as i32).saturating_add(1);
        let min_y = (min_y.floor() as i32).saturating_sub(1);
        let max_y = (max_y.ceil() as i32).saturating_add(1);

        let dst_w = self.width as i32;
        let dst_h = self.height as i32;

        let start_x = min_x.clamp(0, dst_w);
        let end_x = max_x.clamp(0, dst_w);
        let start_y = min_y.clamp(0, dst_h);
        let end_y = max_y.clamp(0, dst_h);
        if start_x >= end_x || start_y >= end_y {
            return;
        }

        let dst_stride = (self.width as usize) * 4;
        for y in start_y..end_y {
            let row_off = (y as usize) * dst_stride;
            for x in start_x..end_x {
                // Map destination pixel center to source coordinates.
                let sx_sy_sw = mul_3x3_vec(inv, [x as f64 + 0.5, y as f64 + 0.5, 1.0]);
                let sw = sx_sy_sw[2];
                if sw.abs() < 1e-9 {
                    continue;
                }
                let sx = sx_sy_sw[0] / sw;
                let sy = sx_sy_sw[1] / sw;
                if sx < 0.0 || sy < 0.0 || sx >= w || sy >= h {
                    continue;
                }

                let s = sample_bilinear_rgba8(src, sx, sy);
                let sa = s[3] as u32;
                if sa == 0 {
                    continue;
                }

                let dst_idx = row_off + (x as usize) * 4;
                let d = [
                    self.data[dst_idx],
                    self.data[dst_idx + 1],
                    self.data[dst_idx + 2],
                    self.data[dst_idx + 3],
                ];

                if sa == 255 {
                    self.data[dst_idx..dst_idx + 4].copy_from_slice(&s);
                    continue;
                }

                let da = d[3] as u32;
                let inv_sa = 255 - sa;
                let out_a = sa + ((da * inv_sa) / 255);
                if out_a == 0 {
                    continue;
                }

                let s_r = s[0] as u32;
                let s_g = s[1] as u32;
                let s_b = s[2] as u32;
                let d_r = d[0] as u32;
                let d_g = d[1] as u32;
                let d_b = d[2] as u32;

                let out_r = (s_r * sa * 255 + d_r * da * inv_sa) / (out_a * 255);
                let out_g = (s_g * sa * 255 + d_g * da * inv_sa) / (out_a * 255);
                let out_b = (s_b * sa * 255 + d_b * da * inv_sa) / (out_a * 255);

                self.data[dst_idx] = out_r as u8;
                self.data[dst_idx + 1] = out_g as u8;
                self.data[dst_idx + 2] = out_b as u8;
                self.data[dst_idx + 3] = out_a as u8;
            }
        }
    }
}

fn sample_bilinear_rgba8(src: &FrameBuffer, x: f64, y: f64) -> [u8; 4] {
    let w = src.width as i32;
    let h = src.height as i32;
    let x0 = (x.floor() as i32).clamp(0, w.saturating_sub(1));
    let y0 = (y.floor() as i32).clamp(0, h.saturating_sub(1));
    let x1 = (x0 + 1).clamp(0, w.saturating_sub(1));
    let y1 = (y0 + 1).clamp(0, h.saturating_sub(1));
    let fx = (x - x0 as f64).clamp(0.0, 1.0) as f32;
    let fy = (y - y0 as f64).clamp(0.0, 1.0) as f32;

    let p00 = src.get_pixel(x0 as u32, y0 as u32).unwrap_or([0, 0, 0, 0]);
    let p10 = src.get_pixel(x1 as u32, y0 as u32).unwrap_or([0, 0, 0, 0]);
    let p01 = src.get_pixel(x0 as u32, y1 as u32).unwrap_or([0, 0, 0, 0]);
    let p11 = src.get_pixel(x1 as u32, y1 as u32).unwrap_or([0, 0, 0, 0]);

    let mut out = [0u8; 4];
    for c in 0..4 {
        let a = p00[c] as f32;
        let b = p10[c] as f32;
        let c0 = p01[c] as f32;
        let d = p11[c] as f32;
        let top = a + (b - a) * fx;
        let bottom = c0 + (d - c0) * fx;
        let v = top + (bottom - top) * fy;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    out
}

fn mul_3x3_vec(m: [f64; 9], v: [f64; 3]) -> [f64; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

fn invert_3x3(m: [f64; 9]) -> Option<[f64; 9]> {
    let a = m[0];
    let b = m[1];
    let c = m[2];
    let d = m[3];
    let e = m[4];
    let f = m[5];
    let g = m[6];
    let h = m[7];
    let i = m[8];

    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;

    let m00 = (e * i - f * h) * inv_det;
    let m01 = (c * h - b * i) * inv_det;
    let m02 = (b * f - c * e) * inv_det;

    let m10 = (f * g - d * i) * inv_det;
    let m11 = (a * i - c * g) * inv_det;
    let m12 = (c * d - a * f) * inv_det;

    let m20 = (d * h - e * g) * inv_det;
    let m21 = (b * g - a * h) * inv_det;
    let m22 = (a * e - b * d) * inv_det;

    Some([m00, m01, m02, m10, m11, m12, m20, m21, m22])
}

fn homography_from_points(src: [[f64; 2]; 4], dst: [[f64; 2]; 4]) -> Option<[f64; 9]> {
    // Solve for h11..h32 with h33 = 1 using 8 equations.
    //
    // x' = (h11 x + h12 y + h13) / (h31 x + h32 y + 1)
    // y' = (h21 x + h22 y + h23) / (h31 x + h32 y + 1)
    //
    // Rearranged linear system A * h = b.
    let mut a = [[0.0f64; 9]; 8];
    let mut b = [0.0f64; 8];

    for i in 0..4 {
        let x = src[i][0];
        let y = src[i][1];
        let xp = dst[i][0];
        let yp = dst[i][1];

        // Row 2i
        a[2 * i][0] = x;
        a[2 * i][1] = y;
        a[2 * i][2] = 1.0;
        a[2 * i][6] = -x * xp;
        a[2 * i][7] = -y * xp;
        b[2 * i] = xp;

        // Row 2i+1
        a[2 * i + 1][3] = x;
        a[2 * i + 1][4] = y;
        a[2 * i + 1][5] = 1.0;
        a[2 * i + 1][6] = -x * yp;
        a[2 * i + 1][7] = -y * yp;
        b[2 * i + 1] = yp;

        // h33 is fixed to 1.0; move to RHS implicitly.
        // (It's already accounted for by leaving column 8 as 0 and treating it as constant 1.)
    }

    // Solve 8x8 by augmenting a last column? We'll build an 8x8 by taking first 8 unknowns.
    // Unknown vector: [h11 h12 h13 h21 h22 h23 h31 h32]
    let mut m = [[0.0f64; 9]; 8];
    for r in 0..8 {
        for c in 0..8 {
            m[r][c] = a[r][c];
        }
        m[r][8] = b[r];
    }

    // Gauss-Jordan elimination.
    for col in 0..8 {
        // Pivot.
        let mut pivot = col;
        let mut best = m[col][col].abs();
        for r in (col + 1)..8 {
            let v = m[r][col].abs();
            if v > best {
                best = v;
                pivot = r;
            }
        }
        if best < 1e-12 {
            return None;
        }
        if pivot != col {
            m.swap(pivot, col);
        }

        let div = m[col][col];
        for c in col..=8 {
            m[col][c] /= div;
        }
        for r in 0..8 {
            if r == col {
                continue;
            }
            let factor = m[r][col];
            if factor.abs() < 1e-12 {
                continue;
            }
            for c in col..=8 {
                m[r][c] -= factor * m[col][c];
            }
        }
    }

    let h11 = m[0][8];
    let h12 = m[1][8];
    let h13 = m[2][8];
    let h21 = m[3][8];
    let h22 = m[4][8];
    let h23 = m[5][8];
    let h31 = m[6][8];
    let h32 = m[7][8];

    Some([h11, h12, h13, h21, h22, h23, h31, h32, 1.0])
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
    fn test_composite_over_projected_simple_quad() {
        let mut dst = FrameBuffer::new(6, 6, PixelFormat::Rgba8);
        let mut src = FrameBuffer::new(1, 1, PixelFormat::Rgba8);
        src.set_pixel(0, 0, [10, 20, 30, 255]);

        // Project a 1x1 source rect onto the pixel at (2, 3).
        let corners = [[2.0, 3.0], [3.0, 3.0], [3.0, 4.0], [2.0, 4.0]];
        dst.composite_over_projected(&src, corners);

        assert_eq!(dst.get_pixel(2, 3), Some([10, 20, 30, 255]));
    }

    #[test]
    fn test_frame_to_timestamp() {
        let frame = Frame::new(30);
        let ts = frame.to_timestamp(30.0);
        assert!((ts.as_seconds() - 1.0).abs() < 0.001);
    }
}
