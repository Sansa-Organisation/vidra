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
    image_cache: HashMap<String, FrameBuffer>,
}

impl WasmRenderer {
    pub fn new() -> Self {
        Self {
            font: default_font(),
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

        let mut current_global = 0u64;
        let mut target_scenes = Vec::new();

        for (i, scene) in project.scenes.iter().enumerate() {
            let sf = (scene.duration.as_seconds() * ctx.fps).ceil() as u64;
            let trans_f = if i > 0 {
                if let Some(trans) = &scene.transition {
                    let prev_sf = (project.scenes[i - 1].duration.as_seconds() * ctx.fps).ceil() as u64;
                    let max_overlap = prev_sf.min(sf);
                    let tf = (trans.duration.as_seconds() * ctx.fps).ceil() as u64;
                    tf.min(max_overlap)
                } else {
                    0
                }
            } else {
                0
            };
            
            let start_f = current_global.saturating_sub(trans_f);
            let end_f = start_f + sf;
            
            if global_frame >= start_f && global_frame < end_f {
                let local_f = global_frame - start_f;
                target_scenes.push((scene, local_f));
            }
            
            current_global = end_f;
        }

        if target_scenes.is_empty() {
            return FrameBuffer::solid(ctx.width, ctx.height, &project.settings.background);
        }

        if target_scenes.len() == 1 {
            let (scene, local_f) = target_scenes[0];
            self.render_scene_frame(&ctx, project, scene, local_f)
        } else {
            let (scene1, local_f1) = target_scenes[0];
            let (scene2, local_f2) = target_scenes[1];
            
            let frame1 = self.render_scene_frame(&ctx, project, scene1, local_f1);
            let frame2 = self.render_scene_frame(&ctx, project, scene2, local_f2);
            
            let trans = scene2.transition.as_ref().unwrap();
            let trans_frames = (trans.duration.as_seconds() * ctx.fps).ceil() as f64;
            let progress = local_f2 as f64 / trans_frames;
            let eased_progress = trans.easing.apply(progress);

            self.apply_transition(frame1, frame2, &trans.effect, eased_progress, ctx.width, ctx.height)
        }
    }

    fn apply_transition(&self, frame1: FrameBuffer, frame2: FrameBuffer, effect: &vidra_ir::transition::TransitionType, progress: f64, width: u32, height: u32) -> FrameBuffer {
        let mut out = frame1.clone();
        
        match effect {
            vidra_ir::transition::TransitionType::Crossfade => {
                for y in 0..height {
                    for x in 0..width {
                        let c1 = frame1.get_pixel(x, y).unwrap_or([0, 0, 0, 0]);
                        let c2 = frame2.get_pixel(x, y).unwrap_or([0, 0, 0, 0]);
                        
                        let r = (c1[0] as f64 * (1.0 - progress) + c2[0] as f64 * progress) as u8;
                        let g = (c1[1] as f64 * (1.0 - progress) + c2[1] as f64 * progress) as u8;
                        let b = (c1[2] as f64 * (1.0 - progress) + c2[2] as f64 * progress) as u8;
                        let a = (c1[3] as f64 * (1.0 - progress) + c2[3] as f64 * progress) as u8;
                        
                        out.set_pixel(x, y, [r, g, b, a]);
                    }
                }
            }
            vidra_ir::transition::TransitionType::Wipe { direction } => {
                let offset_x = (width as f64 * progress) as u32;
                let offset_y = (height as f64 * progress) as u32;
                for y in 0..height {
                    for x in 0..width {
                        let show_frame2 = match direction.as_str() {
                            "left" => x >= width - offset_x,
                            "up" => y >= height - offset_y,
                            "down" => y < offset_y,
                            _ => x < offset_x, // right
                        };
                        if show_frame2 {
                            out.set_pixel(x, y, frame2.get_pixel(x, y).unwrap_or([0, 0, 0, 0]));
                        }
                    }
                }
            }
            vidra_ir::transition::TransitionType::Push { direction } => {
                let offset_x = (width as f64 * progress) as u32;
                let offset_y = (height as f64 * progress) as u32;
                for y in 0..height {
                    for x in 0..width {
                        match direction.as_str() {
                            "left" => {
                                if x >= width - offset_x {
                                    out.set_pixel(x, y, frame2.get_pixel(x - (width - offset_x), y).unwrap_or([0, 0, 0, 0]));
                                } else {
                                    out.set_pixel(x, y, frame1.get_pixel(x + offset_x, y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            "up" => {
                                if y >= height - offset_y {
                                    out.set_pixel(x, y, frame2.get_pixel(x, y - (height - offset_y)).unwrap_or([0, 0, 0, 0]));
                                } else {
                                    out.set_pixel(x, y, frame1.get_pixel(x, y + offset_y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            "down" => {
                                if y < offset_y {
                                    out.set_pixel(x, y, frame2.get_pixel(x, height - offset_y + y).unwrap_or([0, 0, 0, 0]));
                                } else {
                                    out.set_pixel(x, y, frame1.get_pixel(x, y - offset_y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            _ => { // right
                                if x < offset_x {
                                    out.set_pixel(x, y, frame2.get_pixel(width - offset_x + x, y).unwrap_or([0, 0, 0, 0]));
                                } else {
                                    out.set_pixel(x, y, frame1.get_pixel(x - offset_x, y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                        }
                    }
                }
            }
            vidra_ir::transition::TransitionType::Slide { direction } => {
                let offset_x = (width as f64 * progress) as u32;
                let offset_y = (height as f64 * progress) as u32;
                for y in 0..height {
                    for x in 0..width {
                        match direction.as_str() {
                            "left" => {
                                if x >= width - offset_x {
                                    out.set_pixel(x, y, frame2.get_pixel(x - (width - offset_x), y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            "up" => {
                                if y >= height - offset_y {
                                    out.set_pixel(x, y, frame2.get_pixel(x, y - (height - offset_y)).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            "down" => {
                                if y < offset_y {
                                    out.set_pixel(x, y, frame2.get_pixel(x, height - offset_y + y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                            _ => { // right
                                if x < offset_x {
                                    out.set_pixel(x, y, frame2.get_pixel(width - offset_x + x, y).unwrap_or([0, 0, 0, 0]));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        out
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
            if let Some(mut layer_buf) = self.render_layer(ctx, project, layer, local_frame) {
                let (dx, dy) = Self::compute_position(ctx, layer, local_frame);
                let (cx, cy) = Self::apply_anchor(dx, dy, &layer_buf, layer);
                
                if let Some(mask_id) = &layer.mask {
                    if let Some(mask_layer) = scene.layers.iter().find(|l| &l.id == mask_id) {
                        if let Some(mask_buf) = self.render_layer(ctx, project, mask_layer, local_frame) {
                            let (mdx, mdy) = Self::compute_position(ctx, mask_layer, local_frame);
                            let (mcx, mcy) = Self::apply_anchor(mdx, mdy, &mask_buf, mask_layer);
                            let rel_x = mcx - cx;
                            let rel_y = mcy - cy;
                            layer_buf.apply_mask(&mask_buf, rel_x, rel_y);
                        }
                    }
                }
                
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
                    // Draw a fallback rectangle
                    FrameBuffer::solid(200, 200, &Color::rgba(0.5, 0.5, 0.5, 1.0))
                }
            }
            LayerContent::Shape { shape, fill, .. } => {
                let fill_color = fill.unwrap_or(Color::WHITE);
                match shape {
                    vidra_core::types::ShapeType::Rect { width, height, .. } => {
                        let mut c = fill_color;
                        c.a *= opacity as f32;
                        FrameBuffer::solid(*width as u32, *height as u32, &c)
                    }
                    vidra_core::types::ShapeType::Circle { radius } => {
                        let size = (*radius * 2.0) as u32;
                        let mut fb = FrameBuffer::new(size, size, vidra_core::frame::PixelFormat::Rgba8);
                        let rgba = {
                            let mut c = fill_color;
                            c.a *= opacity as f32;
                            c.to_rgba8()
                        };
                        let center = *radius;
                        for y in 0..size {
                            for x in 0..size {
                                let dx = x as f64 - center;
                                let dy = y as f64 - center;
                                if dx * dx + dy * dy <= center * center {
                                    fb.set_pixel(x, y, rgba);
                                }
                            }
                        }
                        fb
                    }
                    vidra_core::types::ShapeType::Ellipse { rx, ry } => {
                        let w = (*rx * 2.0) as u32;
                        let h = (*ry * 2.0) as u32;
                        let mut fb = FrameBuffer::new(w, h, vidra_core::frame::PixelFormat::Rgba8);
                        let rgba = {
                            let mut c = fill_color;
                            c.a *= opacity as f32;
                            c.to_rgba8()
                        };
                        for y in 0..h {
                            for x in 0..w {
                                let dx = (x as f64 - *rx) / rx;
                                let dy = (y as f64 - *ry) / ry;
                                if dx * dx + dy * dy <= 1.0 {
                                    fb.set_pixel(x, y, rgba);
                                }
                            }
                        }
                        fb
                    }
                }
            }
            LayerContent::Empty | LayerContent::Audio { .. } => return None,
            _ => {
                // TTS, AutoCaption, Video — not yet implemented in WASM
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
