//! Standalone CPU renderer for the WASM target.
//!
//! This is a single-threaded, CPU-only compositor that mirrors the logic in
//! `vidra-render::pipeline` but without any native-only dependencies
//! (`rayon`, `dashmap`, `pollster`, `wgpu`).

use std::collections::HashMap;

use fontdue::{Font, FontSettings};
use vidra_core::frame::{FrameBuffer, PixelFormat};
use vidra_core::Color;
use vidra_ir::animation::AnimatableProperty;
use vidra_ir::layer::{Layer, LayerContent};
use vidra_ir::project::Project;
use vidra_ir::scene::Scene;

// ─── Embedded default font ─────────────────────────────────────────

static DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../vidra-render/assets/Inter-Regular.ttf");

fn default_font() -> Font {
    Font::from_bytes(DEFAULT_FONT_BYTES, FontSettings::default())
        .expect("embedded Inter font must be valid")
}

// ─── Render context ─────────────────────────────────────────────────

struct RenderContext {
    width: u32,
    height: u32,
    fps: f64,
}

// ─── CPU Renderer ───────────────────────────────────────────────────

pub struct WasmRenderer {
    font: Font,
    font_cache: HashMap<String, Font>,
    image_cache: HashMap<String, FrameBuffer>,
}

impl WasmRenderer {
    pub fn new() -> Self {
        Self {
            font: default_font(),
            font_cache: HashMap::new(),
            image_cache: HashMap::new(),
        }
    }

    /// Load image assets from embedded bytes or base64 data.
    pub fn load_image_bytes(&mut self, asset_id: &str, data: &[u8]) {
        if let Ok(img) = image::load_from_memory(data) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let fb = FrameBuffer {
                width: w,
                height: h,
                format: PixelFormat::Rgba8,
                data: rgba.into_raw(),
            };
            self.image_cache.insert(asset_id.to_string(), fb);
        }
    }

    /// Render a single frame at the given global frame index.
    pub fn render_frame(&self, project: &Project, global_frame: u64) -> FrameBuffer {
        let ctx = RenderContext {
            width: project.settings.width,
            height: project.settings.height,
            fps: project.settings.fps,
        };

        // Find which scene this frame belongs to
        let mut frame_offset: u64 = 0;
        for scene in &project.scenes {
            let scene_frames = (scene.duration.as_seconds() * ctx.fps).ceil() as u64;
            if global_frame < frame_offset + scene_frames {
                let local_frame = global_frame - frame_offset;
                return self.render_scene_frame(&ctx, project, scene, local_frame);
            }
            frame_offset += scene_frames;
        }

        // Past the end — return black
        FrameBuffer::solid(ctx.width, ctx.height, &project.settings.background)
    }

    fn render_scene_frame(
        &self,
        ctx: &RenderContext,
        project: &Project,
        scene: &Scene,
        local_frame: u64,
    ) -> FrameBuffer {
        let mut canvas = FrameBuffer::solid(ctx.width, ctx.height, &project.settings.background);

        for layer in &scene.layers {
            if !layer.visible {
                continue;
            }
            if let Some(layer_buf) = self.render_layer(ctx, project, layer, local_frame) {
                let (dx, dy) = Self::compute_position(ctx, layer, local_frame);
                let (cx, cy) = Self::apply_anchor(dx, dy, &layer_buf, layer);
                canvas.composite_over(&layer_buf, cx, cy);
            }
        }

        canvas
    }

    fn render_layer(
        &self,
        ctx: &RenderContext,
        project: &Project,
        layer: &Layer,
        frame: u64,
    ) -> Option<FrameBuffer> {
        let opacity = Self::compute_opacity(ctx, layer, frame);
        if opacity <= 0.0 {
            return None;
        }

        let mut buf = match &layer.content {
            LayerContent::Solid { color } => {
                FrameBuffer::solid(ctx.width, ctx.height, color)
            }
            LayerContent::Text {
                text,
                font_family: _,
                font_size,
                color,
            } => {
                self.render_text(text, *font_size as f32, color)
            }
            LayerContent::Image { asset_id } => {
                let id_str = &asset_id.0;
                if let Some(cached) = self.image_cache.get(id_str) {
                    cached.clone()
                } else {
                    // Draw a placeholder rectangle
                    FrameBuffer::solid(200, 200, &Color::rgba(0.5, 0.5, 0.5, 1.0))
                }
            }
            LayerContent::Empty => return None,
            LayerContent::Audio { .. } => return None,
            _ => {
                // TTS, AutoCaption, Video, Shape — not yet implemented in WASM
                FrameBuffer::solid(1, 1, &Color::TRANSPARENT)
            }
        };

        // Apply scale
        let (sx, sy) = Self::compute_scale(ctx, layer, frame);
        if (sx - 1.0).abs() > 0.001 || (sy - 1.0).abs() > 0.001 {
            let new_w = ((buf.width as f64) * sx).round().max(1.0) as u32;
            let new_h = ((buf.height as f64) * sy).round().max(1.0) as u32;
            buf = resize_framebuffer(&buf, new_w, new_h);
        }

        // Apply opacity by scaling alpha channel
        if opacity < 1.0 {
            let alpha_scale = (opacity * 255.0) as u16;
            for chunk in buf.data.chunks_exact_mut(4) {
                let a = chunk[3] as u16;
                chunk[3] = ((a * alpha_scale) / 255) as u8;
            }
        }

        // Render children
        for child in &layer.children {
            if !child.visible {
                continue;
            }
            if let Some(child_buf) = self.render_layer(ctx, project, child, frame) {
                let (dx, dy) = Self::compute_position(ctx, child, frame);
                let (cx, cy) = Self::apply_anchor(dx, dy, &child_buf, child);
                buf.composite_over(&child_buf, cx, cy);
            }
        }

        Some(buf)
    }

    // ── Text rendering (fontdue) ────────────────────────────────

    fn render_text(&self, text: &str, font_size: f32, color: &Color) -> FrameBuffer {
        let font = &self.font;
        let [r, g, b, a] = color.to_rgba8();

        // Measure
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

        let canvas_w = total_width.max(1) as u32;
        let canvas_h = (max_ascent + max_descent).max(1) as u32;
        let mut fb = FrameBuffer::new(canvas_w, canvas_h, PixelFormat::Rgba8);

        // Render glyphs
        let mut cursor_x: i32 = 0;
        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, font_size);
            let glyph_x = cursor_x + metrics.xmin;
            let glyph_y = max_ascent - (metrics.height as i32 + metrics.ymin);

            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let coverage = bitmap[gy * metrics.width + gx];
                    if coverage == 0 {
                        continue;
                    }
                    let px = glyph_x + gx as i32;
                    let py = glyph_y + gy as i32;
                    if px >= 0 && px < fb.width as i32 && py >= 0 && py < fb.height as i32 {
                        let glyph_alpha = (coverage as f32 / 255.0) * (a as f32 / 255.0);
                        let final_alpha = (glyph_alpha * 255.0) as u8;
                        fb.set_pixel(px as u32, py as u32, [r, g, b, final_alpha]);
                    }
                }
            }
            cursor_x += metrics.advance_width as i32;
        }

        fb
    }

    // ── Animation evaluation ────────────────────────────────────

    fn compute_position(ctx: &RenderContext, layer: &Layer, frame: u64) -> (i32, i32) {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut x = layer.transform.position.x;
        let mut y = layer.transform.position.y;
        for anim in &layer.animations {
            if let Some(value) = anim.evaluate(time) {
                match anim.property {
                    AnimatableProperty::PositionX => x = value,
                    AnimatableProperty::PositionY => y = value,
                    _ => {}
                }
            }
        }
        (x.round() as i32, y.round() as i32)
    }

    fn compute_scale(ctx: &RenderContext, layer: &Layer, frame: u64) -> (f64, f64) {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut sx = layer.transform.scale.x;
        let mut sy = layer.transform.scale.y;
        for anim in &layer.animations {
            if let Some(value) = anim.evaluate(time) {
                match anim.property {
                    AnimatableProperty::ScaleX => sx = value,
                    AnimatableProperty::ScaleY => sy = value,
                    _ => {}
                }
            }
        }
        (sx, sy)
    }

    fn compute_opacity(ctx: &RenderContext, layer: &Layer, frame: u64) -> f64 {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut opacity = layer.transform.opacity;
        for anim in &layer.animations {
            if let Some(value) = anim.evaluate(time) {
                if matches!(anim.property, AnimatableProperty::Opacity) {
                    opacity = value;
                }
            }
        }
        opacity.clamp(0.0, 1.0)
    }

    fn has_intrinsic_size(layer: &Layer) -> bool {
        matches!(
            layer.content,
            LayerContent::Text { .. }
                | LayerContent::Image { .. }
                | LayerContent::Video { .. }
                | LayerContent::Shape { .. }
                | LayerContent::TTS { .. }
                | LayerContent::AutoCaption { .. }
        )
    }

    fn apply_anchor(dx: i32, dy: i32, buf: &FrameBuffer, layer: &Layer) -> (i32, i32) {
        if Self::has_intrinsic_size(layer) {
            let cx = dx - (buf.width as f64 * layer.transform.anchor.x).round() as i32;
            let cy = dy - (buf.height as f64 * layer.transform.anchor.y).round() as i32;
            (cx, cy)
        } else {
            (dx, dy)
        }
    }
}

// ─── Resize helper (nearest-neighbor, no image crate needed) ────

fn resize_framebuffer(src: &FrameBuffer, new_w: u32, new_h: u32) -> FrameBuffer {
    let mut dst = FrameBuffer::new(new_w, new_h, PixelFormat::Rgba8);
    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = (x as f64 * src.width as f64 / new_w as f64) as u32;
            let src_y = (y as f64 * src.height as f64 / new_h as f64) as u32;
            if let Some(pixel) = src.get_pixel(src_x.min(src.width - 1), src_y.min(src.height - 1))
            {
                dst.set_pixel(x, y, pixel);
            }
        }
    }
    dst
}
