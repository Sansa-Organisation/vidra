use dashmap::DashMap;
use rayon::prelude::*;
use std::path::Path;

use vidra_core::frame::FrameBuffer;
use vidra_core::hash::{self, ContentHash};
use vidra_core::Color;
use vidra_ir::animation::AnimatableProperty;
use vidra_ir::asset::AssetId;
use vidra_ir::layer::{Layer, LayerContent};
use vidra_ir::project::Project;
use vidra_ir::scene::Scene;

use crate::text::TextRenderer;
use crate::video_decoder::VideoDecoder;

/// Context for rendering a single frame.
pub struct RenderContext {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Current frame rate.
    pub fps: f64,
}

/// Result of a complete render.
pub struct RenderResult {
    /// All rendered frames in order.
    pub frames: Vec<FrameBuffer>,
    /// Total number of frames.
    pub frame_count: u64,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// FPS.
    pub fps: f64,
}

impl RenderResult {
    /// Compute the content hash of the entire render output.
    ///
    /// This hash covers all frames (including dimensions and pixel data)
    /// and can be used for deterministic output verification — the same IR
    /// rendered on the same engine version produces the same hash.
    pub fn content_hash(&self) -> ContentHash {
        hash::hash_frames(&self.frames)
    }

    /// Compute the content hash of a single frame by index.
    pub fn frame_hash(&self, index: usize) -> Option<ContentHash> {
        self.frames.get(index).map(hash::hash_frame)
    }
}

/// The spatial bounds of a layer on the canvas.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayerBounds {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// The render pipeline — takes a Project IR and produces frames.
pub struct RenderPipeline {
    text_renderer: TextRenderer,
    video_decoder: VideoDecoder,
    image_cache: DashMap<String, FrameBuffer>,
    gpu_ctx: std::sync::Arc<crate::gpu::GpuContext>,
    compositor: crate::compositor::GpuCompositor,
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub fn new() -> Result<Self, vidra_core::VidraError> {
        let gpu_ctx = std::sync::Arc::new(
            crate::gpu::GpuContext::init().map_err(|e| {
                vidra_core::VidraError::Render(format!("Failed to initialize WGPU context: {}", e))
            })?
        );
        let compositor = crate::compositor::GpuCompositor::new(gpu_ctx.clone());

        Ok(Self {
            text_renderer: TextRenderer::new(),
            video_decoder: VideoDecoder::new(),
            image_cache: DashMap::new(),
            gpu_ctx,
            compositor,
        })
    }

    /// Load fonts (and other assets later) from the Project into the pipeline
    pub fn load_assets(&mut self, project: &Project) -> Result<(), vidra_core::VidraError> {
        for asset in project.assets.all() {
            if asset.asset_type == vidra_ir::asset::AssetType::Font {
                tracing::info!("Loading font {} from {}", asset.id.0, asset.path.display());
                self.text_renderer.load_font(&asset.id.0, &asset.path)
                    .map_err(|e| vidra_core::VidraError::Render(format!("Asset load error {}: {}", asset.id.0, e)))?;
            } else if asset.asset_type == vidra_ir::asset::AssetType::Image {
                tracing::info!("Loading image {} from {}", asset.id.0, asset.path.display());
                let fb = crate::image_loader::load_image(&asset.path)
                    .map_err(|e| vidra_core::VidraError::Render(format!("Asset load error {}: {}", asset.id.0, e)))?;
                self.image_cache.insert(asset.id.to_string(), fb);
            }
        }
        Ok(())
    }

    /// Render the entire project to a sequence of FrameBuffers.
    pub fn render(project: &Project) -> Result<RenderResult, vidra_core::VidraError> {
        let mut pipeline = Self::new()?;
        pipeline.load_assets(project)?;
        
        let ctx = RenderContext {
            width: project.settings.width,
            height: project.settings.height,
            fps: project.settings.fps,
        };

        let total_frames = project.total_frames();
        let mut tasks = Vec::with_capacity(total_frames as usize);
        let mut global_frame: u64 = 0;

        for scene in &project.scenes {
            let scene_frames = scene.frame_count(project.settings.fps);
            for local_frame in 0..scene_frames {
                tasks.push((scene, local_frame, global_frame));
                global_frame += 1;
            }
        }

        let frames: Result<Vec<FrameBuffer>, _> = tasks
            .into_par_iter()
            .map(|(scene, local_frame, global_frame)| {
                pipeline.render_frame(&ctx, project, scene, local_frame, global_frame)
            })
            .collect();

        let frames = frames?;

        Ok(RenderResult {
            frames,
            frame_count: total_frames,
            width: ctx.width,
            height: ctx.height,
            fps: ctx.fps,
        })
    }

    /// Render exactly one frame by global frame index. Used for live preview.
    pub fn render_frame_index(
        &self,
        project: &Project,
        global_frame: u64,
    ) -> Result<FrameBuffer, vidra_core::VidraError> {
        let ctx = RenderContext {
            width: project.settings.width,
            height: project.settings.height,
            fps: project.settings.fps,
        };

        let mut current_global = 0.0;
        let mut target_scene = None;
        let mut local_f = 0;

        for scene in &project.scenes {
            let sf = scene.frame_count(project.settings.fps);
            if global_frame < (current_global as u64) + sf {
                target_scene = Some(scene);
                local_f = global_frame - (current_global as u64);
                break;
            }
            current_global += sf as f64;
        }

        let scene = target_scene.ok_or_else(|| {
            vidra_core::VidraError::Render(format!("frame out of bounds: {}", global_frame))
        })?;

        self.render_frame(&ctx, project, scene, local_f, global_frame)
    }

    /// Retrieve the bounding boxes of all visible layers at this exact frame
    pub fn inspect_frame_bounds(
        &self,
        project: &Project,
        global_frame: u64,
    ) -> Result<Vec<LayerBounds>, vidra_core::VidraError> {
        let ctx = RenderContext {
            width: project.settings.width,
            height: project.settings.height,
            fps: project.settings.fps,
        };

        let mut current_global = 0.0;
        let mut target_scene = None;
        let mut local_f = 0;

        for scene in &project.scenes {
            let sf = scene.frame_count(project.settings.fps);
            if global_frame < (current_global as u64) + sf {
                target_scene = Some(scene);
                local_f = global_frame - (current_global as u64);
                break;
            }
            current_global += sf as f64;
        }

        let scene = target_scene.ok_or_else(|| {
            vidra_core::VidraError::Render(format!("frame out of bounds: {}", global_frame))
        })?;

        let mut bounds = Vec::new();

        for layer in &scene.layers {
            if !layer.visible {
                continue;
            }
            if let Ok(layer_buf) = self.render_layer(&ctx, project, layer, local_f) {
                let (dx, dy) = Self::compute_layer_position(&ctx, layer, local_f);
                let cx = dx - (layer_buf.width as f64 * layer.transform.anchor.x).round() as i32;
                let cy = dy - (layer_buf.height as f64 * layer.transform.anchor.y).round() as i32;
                bounds.push(LayerBounds {
                    id: layer.id.to_string(),
                    x: cx,
                    y: cy,
                    width: layer_buf.width,
                    height: layer_buf.height,
                });
            }
        }

        Ok(bounds)
    }

    /// Render a single frame.
    fn render_frame(
        &self,
        ctx: &RenderContext,
        project: &Project,
        scene: &Scene,
        local_frame: u64,
        _global_frame: u64,
    ) -> Result<FrameBuffer, vidra_core::VidraError> {
        // Start with the background color
        let mut canvas = FrameBuffer::solid(ctx.width, ctx.height, &project.settings.background);

        // Composite layers bottom-to-top
        for layer in &scene.layers {
            if !layer.visible {
                continue;
            }
            if let Ok(layer_buf) = self.render_layer(ctx, project, layer, local_frame) {
                let (dx, dy) = Self::compute_layer_position(ctx, layer, local_frame);
                let cx = dx - (layer_buf.width as f64 * layer.transform.anchor.x).round() as i32;
                let cy = dy - (layer_buf.height as f64 * layer.transform.anchor.y).round() as i32;
                self.compositor.composite(&mut canvas, &layer_buf, cx, cy, &layer.effects);
            }
        }

        Ok(canvas)
    }

    /// Compute the animated position of a layer at a given frame.
    fn compute_layer_position(ctx: &RenderContext, layer: &Layer, frame: u64) -> (i32, i32) {
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

        (x as i32, y as i32)
    }

    /// Compute the animated scale of a layer at a given frame.
    fn compute_layer_scale(ctx: &RenderContext, layer: &Layer, frame: u64) -> (f64, f64) {
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

    /// Compute the animated opacity of a layer at a given frame.
    fn compute_layer_opacity(ctx: &RenderContext, layer: &Layer, frame: u64) -> f64 {
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

    /// Render a single layer to its own FrameBuffer.
    fn render_layer(
        &self,
        ctx: &RenderContext,
        project: &Project,
        layer: &Layer,
        frame: u64,
    ) -> Result<FrameBuffer, vidra_core::VidraError> {
        let opacity = Self::compute_layer_opacity(ctx, layer, frame);

        let mut buf = match &layer.content {
            LayerContent::Solid { color } => {
                let mut c = *color;
                c.a *= opacity as f32;
                FrameBuffer::solid(ctx.width, ctx.height, &c)
            }
            LayerContent::Empty | LayerContent::Audio { .. } => {
                FrameBuffer::new(ctx.width, ctx.height, vidra_core::frame::PixelFormat::Rgba8)
            }
            LayerContent::Text {
                text,
                font_family,
                font_size,
                color,
            } => {
                // Real text rendering using fontdue
                let mut c = *color;
                c.a *= opacity as f32;
                self.text_renderer
                    .render_text(text, font_family, *font_size as f32, &c)
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
                        let mut fb = FrameBuffer::new(size, size, vidra_core::PixelFormat::Rgba8);
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
                        let mut fb = FrameBuffer::new(w, h, vidra_core::PixelFormat::Rgba8);
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
            LayerContent::Image { asset_id } => self.load_image_asset(project, asset_id, opacity),
            LayerContent::Video {
                asset_id,
                trim_start,
                ..
            } => self.render_video_frame(ctx, project, asset_id, trim_start, frame, opacity),
            LayerContent::TTS { text, .. } => {
                // AI TTS node — render a placeholder caption frame
                self.text_renderer
                    .render_text(&format!("🔊 {}", text), "Inter", 32.0, &Color::WHITE)
            }
            LayerContent::AutoCaption { .. } => {
                // AI AutoCaption node — render placeholder until whisper inference is wired
                self.text_renderer
                    .render_text("[Auto Caption]", "Inter", 28.0, &Color::WHITE)
            }
        };

        // Apply scale generically via resize_to_fit if needed
        let (sx, sy) = Self::compute_layer_scale(ctx, layer, frame);
        if (sx - 1.0).abs() > f64::EPSILON || (sy - 1.0).abs() > f64::EPSILON {
            let max_w = (buf.width as f64 * sx).round() as u32;
            let max_h = (buf.height as f64 * sy).round() as u32;
            if max_w > 0 && max_h > 0 {
                // Easiest scaling fallback for prototype implementation
                buf = crate::image_loader::resize_to_fit(&buf, max_w.max(1), max_h.max(1));
            } else {
                // If scaled down to 0, just return empty frame buffer
                buf = FrameBuffer::new(1, 1, vidra_core::frame::PixelFormat::Rgba8);
            }
        }

        // Render child layers
        for child in &layer.children {
            if !child.visible {
                continue;
            }
            let child_buf = self.render_layer(ctx, project, child, frame)?;
            let (dx, dy) = Self::compute_layer_position(ctx, child, frame);
            let cx = dx - (child_buf.width as f64 * child.transform.anchor.x).round() as i32;
            let cy = dy - (child_buf.height as f64 * child.transform.anchor.y).round() as i32;
            buf.composite_over(&child_buf, cx, cy);
        }

        Ok(buf)
    }

    /// Load an image asset, with caching.
    fn load_image_asset(&self, project: &Project, asset_id: &AssetId, opacity: f64) -> FrameBuffer {
        let cache_key = asset_id.to_string();

        if !self.image_cache.contains_key(&cache_key) {
            // Try to find the asset in the registry and load it lazily if not pre-cached
            let loaded = project.assets.get(asset_id).and_then(|asset| {
                let path = Path::new(&asset.path);
                match crate::image_loader::load_image(path) {
                    Ok(fb) => Some(fb),
                    Err(e) => {
                        tracing::warn!("Failed to load image asset '{}': {}", asset_id, e);
                        None
                    }
                }
            });

            if let Some(fb) = loaded {
                self.image_cache.insert(cache_key.clone(), fb);
            } else {
                // Return magenta placeholder for missing images
                return FrameBuffer::solid(128, 128, &Color::rgba(1.0, 0.0, 1.0, opacity as f32));
            }
        }

        if let Some(cached) = self.image_cache.get(&cache_key) {
            if (opacity - 1.0).abs() < f64::EPSILON {
                cached.value().clone()
            } else {
                // Apply opacity to cached image
                let mut fb = cached.value().clone();
                for y in 0..fb.height {
                    for x in 0..fb.width {
                        if let Some([r, g, b, a]) = fb.get_pixel(x, y) {
                            let new_a = (a as f64 * opacity) as u8;
                            fb.set_pixel(x, y, [r, g, b, new_a]);
                        }
                    }
                }
                fb
            }
        } else {
            FrameBuffer::solid(128, 128, &Color::rgba(1.0, 0.0, 1.0, opacity as f32))
        }
    }

    /// Render a video frame by extracting the appropriate frame from the source video.
    fn render_video_frame(
        &self,
        ctx: &RenderContext,
        project: &Project,
        asset_id: &AssetId,
        trim_start: &vidra_core::Duration,
        frame: u64,
        opacity: f64,
    ) -> FrameBuffer {
        let asset = match project.assets.get(asset_id) {
            Some(a) => a,
            None => {
                tracing::warn!("Video asset '{}' not found in registry", asset_id);
                return FrameBuffer::solid(
                    ctx.width,
                    ctx.height,
                    &Color::rgba(0.0, 1.0, 1.0, opacity as f32),
                );
            }
        };

        let path = Path::new(&asset.path);

        // Calculate the timestamp within the source video
        let timeline_time = frame as f64 / ctx.fps;
        let source_time = trim_start.as_seconds() + timeline_time;

        match self
            .video_decoder
            .extract_frame(path, source_time, ctx.width, ctx.height)
        {
            Ok(mut fb) => {
                // Apply opacity if needed
                if (opacity - 1.0).abs() > f64::EPSILON {
                    for y in 0..fb.height {
                        for x in 0..fb.width {
                            if let Some([r, g, b, a]) = fb.get_pixel(x, y) {
                                let new_a = (a as f64 * opacity) as u8;
                                fb.set_pixel(x, y, [r, g, b, new_a]);
                            }
                        }
                    }
                }
                fb
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to extract video frame (asset '{}', t={:.3}s): {}",
                    asset_id,
                    source_time,
                    e
                );
                FrameBuffer::solid(
                    ctx.width,
                    ctx.height,
                    &Color::rgba(0.0, 1.0, 1.0, opacity as f32),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vidra_ir::layer::{Layer, LayerContent, LayerId};
    use vidra_ir::project::{Project, ProjectSettings};
    use vidra_ir::scene::{Scene, SceneId};

    fn test_project() -> Project {
        let mut project = Project::new(ProjectSettings::custom(320, 240, 30.0));
        let mut scene = Scene::new(
            SceneId::new("test"),
            vidra_core::Duration::from_seconds(1.0),
        );
        scene.add_layer(Layer::new(
            LayerId::new("bg"),
            LayerContent::Solid { color: Color::BLUE },
        ));
        scene.add_layer(
            Layer::new(
                LayerId::new("title"),
                LayerContent::Text {
                    text: "Hello".into(),
                    font_family: "Inter".into(),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            )
            .with_position(100.0, 100.0),
        );
        project.add_scene(scene);
        project
    }

    #[test]
    fn test_render_pipeline_produces_frames() {
        let project = test_project();
        let result = RenderPipeline::render(&project).unwrap();
        assert_eq!(result.frame_count, 30); // 1 second at 30fps
        assert_eq!(result.frames.len(), 30);
        assert_eq!(result.width, 320);
        assert_eq!(result.height, 240);
    }

    #[test]
    fn test_render_pipeline_background_color() {
        let mut project = Project::new(ProjectSettings::custom(4, 4, 1.0));
        project.settings.background = Color::RED;
        let scene = Scene::new(SceneId::new("s"), vidra_core::Duration::from_seconds(1.0));
        // Empty scene — should be all background color
        project.add_scene(scene);
        let result = RenderPipeline::render(&project).unwrap();
        let pixel = result.frames[0].get_pixel(0, 0).unwrap();
        assert_eq!(pixel, [255, 0, 0, 255]);
    }

    #[test]
    fn test_render_solid_layer() {
        let mut project = Project::new(ProjectSettings::custom(10, 10, 1.0));
        project.settings.background = Color::BLACK;
        let mut scene = Scene::new(SceneId::new("s"), vidra_core::Duration::from_seconds(1.0));
        scene.add_layer(Layer::new(
            LayerId::new("fill"),
            LayerContent::Solid {
                color: Color::GREEN,
            },
        ));
        project.add_scene(scene);
        let result = RenderPipeline::render(&project).unwrap();
        // The solid green layer should cover the entire canvas
        let pixel = result.frames[0].get_pixel(5, 5).unwrap();
        assert_eq!(pixel, [0, 255, 0, 255]);
    }

    #[test]
    fn test_render_content_hash_deterministic() {
        // Rendering the same project twice should produce identical hashes
        let project = test_project();
        let result1 = RenderPipeline::render(&project).unwrap();
        let result2 = RenderPipeline::render(&project).unwrap();

        let hash1 = result1.content_hash();
        let hash2 = result2.content_hash();
        assert_eq!(
            hash1, hash2,
            "same IR must produce identical content hashes"
        );
    }

    #[test]
    fn test_render_content_hash_differs_for_different_input() {
        // Different projects should produce different hashes
        let mut project1 = Project::new(ProjectSettings::custom(4, 4, 1.0));
        project1.settings.background = Color::RED;
        project1.add_scene(Scene::new(
            SceneId::new("s"),
            vidra_core::Duration::from_seconds(1.0),
        ));

        let mut project2 = Project::new(ProjectSettings::custom(4, 4, 1.0));
        project2.settings.background = Color::BLUE;
        project2.add_scene(Scene::new(
            SceneId::new("s"),
            vidra_core::Duration::from_seconds(1.0),
        ));

        let hash1 = RenderPipeline::render(&project1).unwrap().content_hash();
        let hash2 = RenderPipeline::render(&project2).unwrap().content_hash();
        assert_ne!(
            hash1, hash2,
            "different IR must produce different content hashes"
        );
    }

    #[test]
    fn test_render_frame_hash() {
        let project = test_project();
        let result = RenderPipeline::render(&project).unwrap();

        // All frames should be identical in this static scene
        let hash0 = result.frame_hash(0).unwrap();
        let hash1 = result.frame_hash(1).unwrap();
        assert_eq!(hash0, hash1, "static scene has identical frames");

        // Out of bounds should return None
        assert!(result.frame_hash(9999).is_none());
    }

    #[test]
    fn test_render_content_hash_hex_format() {
        let project = test_project();
        let result = RenderPipeline::render(&project).unwrap();
        let hash = result.content_hash();
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64, "SHA-256 hash should be 64 hex characters");
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_render_video_layer_fallback() {
        // Video layer with a missing asset should fall back to a cyan placeholder
        let mut project = Project::new(ProjectSettings::custom(10, 10, 1.0));
        project.settings.background = Color::BLACK;
        let mut scene = Scene::new(SceneId::new("s"), vidra_core::Duration::from_seconds(1.0));
        scene.add_layer(Layer::new(
            LayerId::new("vid"),
            LayerContent::Video {
                asset_id: vidra_ir::asset::AssetId::new("nonexistent"),
                trim_start: vidra_core::Duration::from_seconds(0.0),
                trim_end: None,
            },
        ));
        project.add_scene(scene);
        let result = RenderPipeline::render(&project).unwrap();
        assert_eq!(result.frame_count, 1);
        // Should render without panic — the fallback is a cyan-tinted frame
    }
}
