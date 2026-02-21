//! Text rendering module.
//! Uses fontdue for CPU-based font rasterization.
//! Phase 0: supports single-line and multi-line text with alignment.
//! Phase 1: will add subpixel rendering, text decoration, rich text.

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use fontdue::{Font, FontSettings};
use vidra_core::frame::FrameBuffer;
use vidra_core::{Color, PixelFormat};

/// Default embedded font bytes (Inter-Regular).
/// We embed a basic font so text rendering works out of the box.
static DEFAULT_FONT: OnceLock<Font> = OnceLock::new();

/// Text horizontal alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

/// Get or initialize the default font (a built-in sans-serif fallback).
fn default_font() -> &'static Font {
    DEFAULT_FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../assets/Inter-Regular.ttf") as &[u8],
            FontSettings::default(),
        )
        .expect("embedded Inter-Regular.ttf font must be valid")
    })
}

/// Text renderer — rasterizes text to a FrameBuffer.
pub struct TextRenderer {
    font_cache: HashMap<String, Font>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            font_cache: HashMap::new(),
        }
    }

    /// Load a font from a file path.
    pub fn load_font(&mut self, name: &str, path: &Path) -> Result<(), String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("failed to read font file {}: {}", path.display(), e))?;
        let font = Font::from_bytes(data, FontSettings::default())
            .map_err(|e| format!("failed to parse font {}: {}", name, e))?;
        self.font_cache.insert(name.to_string(), font);
        Ok(())
    }

    /// Get a font by family name, falling back to the default.
    fn get_font(&self, font_family: &str) -> &Font {
        self.font_cache
            .get(font_family)
            .unwrap_or_else(|| default_font())
    }

    /// Render text into a FrameBuffer.
    ///
    /// Supports multi-line text (splits on `\n`).
    /// Returns a buffer sized to fit the rendered text.
    pub fn render_text(
        &self,
        text: &str,
        font_family: &str,
        font_size: f32,
        color: &Color,
    ) -> FrameBuffer {
        self.render_text_aligned(text, font_family, font_size, color, TextAlign::Left)
    }

    /// Render text with alignment into a FrameBuffer.
    ///
    /// Supports multi-line text (splits on `\n`).
    /// Returns a buffer sized to fit the rendered text.
    pub fn render_text_aligned(
        &self,
        text: &str,
        font_family: &str,
        font_size: f32,
        color: &Color,
        align: TextAlign,
    ) -> FrameBuffer {
        let font = self.get_font(font_family);
        let lines: Vec<&str> = text.split('\n').collect();

        if lines.is_empty() || text.is_empty() {
            return FrameBuffer::new(1, 1, PixelFormat::Rgba8);
        }

        // First pass: measure each line
        let mut line_metrics: Vec<LineMeasure> = Vec::with_capacity(lines.len());
        let mut max_width: i32 = 0;

        let line_spacing = (font_size * 1.3) as i32; // ~130% line height

        for &line_text in &lines {
            let measure = self.measure_line(font, line_text, font_size);
            max_width = max_width.max(measure.width);
            line_metrics.push(measure);
        }

        let total_height = if lines.len() == 1 {
            line_metrics[0].ascent + line_metrics[0].descent
        } else {
            line_spacing * (lines.len() as i32 - 1)
                + line_metrics.last().map_or(0, |m| m.ascent + m.descent)
        };

        let canvas_width = max_width.max(1) as u32;
        let canvas_height = total_height.max(1) as u32;

        let mut fb = FrameBuffer::new(canvas_width, canvas_height, PixelFormat::Rgba8);
        let [r, g, b, a] = color.to_rgba8();

        // Second pass: render each line
        let mut y_offset: i32 = 0;

        for (i, &line_text) in lines.iter().enumerate() {
            let measure = &line_metrics[i];

            // Calculate x offset based on alignment
            let x_offset = match align {
                TextAlign::Left => 0,
                TextAlign::Center => (max_width - measure.width) / 2,
                TextAlign::Right => max_width - measure.width,
            };

            self.render_line_into(
                &mut fb,
                font,
                line_text,
                font_size,
                [r, g, b, a],
                x_offset,
                y_offset,
                measure.ascent,
            );

            y_offset += line_spacing;
        }

        fb
    }

    /// Measure a single line of text.
    fn measure_line(&self, font: &Font, text: &str, font_size: f32) -> LineMeasure {
        let mut total_width: i32 = 0;
        let mut max_ascent: i32 = 0;
        let mut max_descent: i32 = 0;

        for ch in text.chars() {
            let (metrics, _) = font.rasterize(ch, font_size);
            let ascent = metrics.height as i32 + metrics.ymin;
            let descent = -metrics.ymin;
            max_ascent = max_ascent.max(ascent);
            max_descent = max_descent.max(descent);
            total_width += metrics.advance_width as i32;
        }

        // For empty lines, use font metrics from a space character
        if text.is_empty() {
            let (metrics, _) = font.rasterize(' ', font_size);
            let ascent = metrics.height as i32 + metrics.ymin;
            let descent = -metrics.ymin;
            max_ascent = max_ascent.max(ascent);
            max_descent = max_descent.max(descent);
        }

        LineMeasure {
            width: total_width,
            ascent: max_ascent,
            descent: max_descent,
        }
    }

    /// Render a single line of text into an existing FrameBuffer.
    fn render_line_into(
        &self,
        fb: &mut FrameBuffer,
        font: &Font,
        text: &str,
        font_size: f32,
        color_rgba: [u8; 4],
        x_offset: i32,
        y_offset: i32,
        line_ascent: i32,
    ) {
        let [r, g, b, a] = color_rgba;
        let mut cursor_x: i32 = x_offset;

        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, font_size);
            let glyph_x = cursor_x + metrics.xmin;
            let glyph_y = y_offset + line_ascent - (metrics.height as i32 + metrics.ymin);

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let coverage = bitmap[gy * metrics.width + gx];
                    if coverage == 0 {
                        continue;
                    }

                    let px = glyph_x + gx as i32;
                    let py = glyph_y + gy as i32;

                    if px >= 0 && px < fb.width as i32 && py >= 0 && py < fb.height as i32 {
                        // Alpha composite: coverage * text alpha
                        let glyph_alpha = (coverage as f32 / 255.0) * (a as f32 / 255.0);
                        let final_alpha = (glyph_alpha * 255.0) as u8;
                        fb.set_pixel(px as u32, py as u32, [r, g, b, final_alpha]);
                    }
                }
            }

            cursor_x += metrics.advance_width as i32;
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Measurements for a single line of text.
#[derive(Debug, Clone)]
struct LineMeasure {
    /// Total advance width.
    width: i32,
    /// Max ascent (above baseline).
    ascent: i32,
    /// Max descent (below baseline).
    descent: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_single_line() {
        let renderer = TextRenderer::new();
        let fb = renderer.render_text("Hello", "Inter", 24.0, &Color::WHITE);
        assert!(fb.width > 0);
        assert!(fb.height > 0);
        // Text should have some non-zero pixels
        let has_content = (0..fb.width)
            .any(|x| (0..fb.height).any(|y| fb.get_pixel(x, y).map_or(false, |p| p[3] > 0)));
        assert!(has_content, "rendered text should have visible pixels");
    }

    #[test]
    fn test_render_multi_line() {
        let renderer = TextRenderer::new();
        let single = renderer.render_text("Hello", "Inter", 24.0, &Color::WHITE);
        let multi = renderer.render_text("Hello\nWorld", "Inter", 24.0, &Color::WHITE);
        // Multi-line should be taller than single line
        assert!(
            multi.height > single.height,
            "multi-line text should be taller"
        );
    }

    #[test]
    fn test_render_empty_string() {
        let renderer = TextRenderer::new();
        let fb = renderer.render_text("", "Inter", 24.0, &Color::WHITE);
        // Should produce a 1x1 fallback
        assert!(fb.width >= 1);
        assert!(fb.height >= 1);
    }

    #[test]
    fn test_render_aligned_center() {
        let renderer = TextRenderer::new();
        let fb = renderer.render_text_aligned(
            "Hi\nHello World",
            "Inter",
            24.0,
            &Color::WHITE,
            TextAlign::Center,
        );
        // The second line is longer, so the first line should be centered
        assert!(fb.width > 0);
        assert!(fb.height > 0);
    }

    #[test]
    fn test_render_aligned_right() {
        let renderer = TextRenderer::new();
        let fb = renderer.render_text_aligned(
            "Hi\nHello World",
            "Inter",
            24.0,
            &Color::WHITE,
            TextAlign::Right,
        );
        assert!(fb.width > 0);
        assert!(fb.height > 0);
    }

    #[test]
    fn test_text_color() {
        let renderer = TextRenderer::new();
        let fb = renderer.render_text("X", "Inter", 48.0, &Color::RED);
        // Find a non-zero pixel and verify it has red channel
        let mut found_red = false;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if let Some([r, _, _, a]) = fb.get_pixel(x, y) {
                    if a > 0 && r > 0 {
                        found_red = true;
                    }
                }
            }
        }
        assert!(found_red, "red text should have red-channel pixels");
    }

    #[test]
    fn test_load_custom_font() {
        let mut renderer = TextRenderer::new();
        // Try loading a non-existent font
        let result = renderer.load_font("missing", Path::new("/nonexistent/font.ttf"));
        assert!(result.is_err());
    }
}
