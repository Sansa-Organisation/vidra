use dashmap::DashMap;
use rayon::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use vidra_core::frame::FrameBuffer;
use vidra_core::hash::{self, ContentHash};
use vidra_core::Color;
use vidra_ir::animation::AnimatableProperty;
use vidra_ir::asset::AssetId;
use vidra_ir::layer::{Layer, LayerContent};
use vidra_ir::project::Project;
use vidra_ir::scene::Scene;

use evalexpr::{build_operator_tree, ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value};

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

    /// Mouse position in pixel coordinates for interactive previews.
    pub mouse_x: f64,
    pub mouse_y: f64,

    /// Runtime numeric state vars (used by interactive previews).
    pub state_vars: HashMap<String, f64>,
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
    shader_cache: DashMap<String, String>,
    #[allow(dead_code)]
    gpu_ctx: std::sync::Arc<crate::gpu::GpuContext>,
    compositor: crate::compositor::GpuCompositor,
    shader_renderer: crate::custom_shader::CustomShaderRenderer,
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
        let shader_renderer = crate::custom_shader::CustomShaderRenderer::new(gpu_ctx.clone());

        Ok(Self {
            text_renderer: TextRenderer::new(),
            video_decoder: VideoDecoder::new(),
            image_cache: DashMap::new(),
            shader_cache: DashMap::new(),
            gpu_ctx,
            compositor,
            shader_renderer,
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
            } else if asset.path.extension().map(|e| e == "wgsl").unwrap_or(false) {
                tracing::info!("Loading shader {} from {}", asset.id.0, asset.path.display());
                let source = std::fs::read_to_string(&asset.path)
                    .map_err(|e| vidra_core::VidraError::Render(format!("Failed to read custom shader {}: {}", asset.id.0, e)))?;
                self.shader_cache.insert(asset.id.to_string(), source);
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
            mouse_x: 0.0,
            mouse_y: 0.0,
            state_vars: HashMap::new(),
        };

        let total_frames = project.total_frames();
        let frames: Result<Vec<FrameBuffer>, _> = (0..total_frames)
            .into_par_iter()
            .map(|global_frame| {
                pipeline.render_frame_index(project, global_frame)
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
            mouse_x: 0.0,
            mouse_y: 0.0,
            state_vars: HashMap::new(),
        };

        let mut current_global = 0u64;
        
        let mut target_scenes = Vec::new();

        for (i, scene) in project.scenes.iter().enumerate() {
            let sf = scene.frame_count(project.settings.fps);
            let trans_f = if i > 0 {
                if let Some(trans) = &scene.transition {
                    let max_overlap = project.scenes[i - 1].frame_count(project.settings.fps).min(sf);
                    trans.duration.frame_count(project.settings.fps).min(max_overlap)
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
            return Err(vidra_core::VidraError::Render(format!("frame out of bounds: {}", global_frame)));
        }

        if target_scenes.len() == 1 {
            let (scene, local_f) = target_scenes[0];
            self.render_frame(&ctx, project, scene, local_f, global_frame)
        } else {
            // Composite the two scenes for transition
            // They are added in order, so index 0 is the older scene, index 1 is the entering scene
            let (scene1, local_f1) = target_scenes[0];
            let (scene2, local_f2) = target_scenes[1];
            
            let frame1 = self.render_frame(&ctx, project, scene1, local_f1, global_frame)?;
            let frame2 = self.render_frame(&ctx, project, scene2, local_f2, global_frame)?;
            
            let trans = scene2.transition.as_ref().unwrap();
            let trans_frames = trans.duration.frame_count(project.settings.fps) as f64;
            let progress = local_f2 as f64 / trans_frames;
            let eased_progress = trans.easing.apply(progress);

            self.apply_transition(frame1, frame2, &trans.effect, eased_progress, ctx.width, ctx.height)
        }
    }
    
    fn apply_transition(&self, frame1: FrameBuffer, frame2: FrameBuffer, effect: &vidra_ir::transition::TransitionType, progress: f64, width: u32, height: u32) -> Result<FrameBuffer, vidra_core::VidraError> {
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
        
        Ok(out)
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
            mouse_x: 0.0,
            mouse_y: 0.0,
            state_vars: HashMap::new(),
        };

        let mut current_global = 0u64;
        let mut target_scenes = Vec::new();

        for (i, scene) in project.scenes.iter().enumerate() {
            let sf = scene.frame_count(project.settings.fps);
            let trans_f = if i > 0 {
                if let Some(trans) = &scene.transition {
                    let max_overlap = project.scenes[i - 1].frame_count(project.settings.fps).min(sf);
                    trans.duration.frame_count(project.settings.fps).min(max_overlap)
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

        let scene = target_scenes.last().map(|(s, _)| *s).ok_or_else(|| {
            vidra_core::VidraError::Render(format!("frame out of bounds: {}", global_frame))
        })?;
        let local_f = target_scenes.last().map(|(_, f)| *f).unwrap();

        let mut bounds = Vec::new();

        for layer in &scene.layers {
            if !layer.visible {
                continue;
            }
            let (content, _) = Self::compute_layer_animated_state(&ctx, layer, local_f);
            if let Ok(layer_buf) = self.render_layer(&ctx, project, layer, &content, local_f) {
                let (dx, dy) = Self::compute_layer_position(&ctx, layer, local_f);
                let (cx, cy) = Self::apply_anchor(dx, dy, &layer_buf, layer, &content);
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
            let (content, effects) = Self::compute_layer_animated_state(ctx, layer, local_frame);
            if let Ok(mut layer_buf) = self.render_layer(ctx, project, layer, &content, local_frame) {
                let (dx, dy) = Self::compute_layer_position(ctx, layer, local_frame);
                
                if let Some(mask_id) = &layer.mask {
                    if let Some(mask_layer) = scene.layers.iter().find(|l| &l.id == mask_id) {
                        let (m_content, _) = Self::compute_layer_animated_state(ctx, mask_layer, local_frame);
                        if let Ok(mask_buf) = self.render_layer(ctx, project, mask_layer, &m_content, local_frame) {
                            let (mdx, mdy) = Self::compute_layer_position(ctx, mask_layer, local_frame);
                            let (mcx, mcy) = Self::apply_anchor(mdx, mdy, &mask_buf, mask_layer, &m_content);
                            // For masks, keep the existing 2D anchor-based alignment (masking is applied
                            // in the layer's local buffer space).
                            let (cx, cy) = Self::apply_anchor(dx, dy, &layer_buf, layer, &content);
                            let rel_x = mcx - cx;
                            let rel_y = mcy - cy;
                            layer_buf.apply_mask(&mask_buf, rel_x, rel_y);
                        }
                    }
                }

                let transform = Self::compute_layer_transform(ctx, layer, local_frame, dx, dy);
                if Self::needs_projective_composite(&transform) {
                    let corners = transform.project_corners(layer_buf.width as f64, layer_buf.height as f64);
                    self.compositor
                        .composite_projected(&mut canvas, &layer_buf, corners, &effects);
                } else {
                    let (cx, cy) = Self::apply_anchor(dx, dy, &layer_buf, layer, &content);
                    self.compositor.composite(&mut canvas, &layer_buf, cx, cy, &effects);
                }
            }
        }

        Ok(canvas)
    }

    fn compute_layer_transform(
        ctx: &RenderContext,
        layer: &Layer,
        frame: u64,
        dx: i32,
        dy: i32,
    ) -> vidra_core::Transform2D {
        let mut t = layer.transform;
        t.position.x = dx as f64;
        t.position.y = dy as f64;

        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        for anim in &layer.animations {
            if let Some(value) = Self::evaluate_animation(ctx, anim, time) {
                match anim.property {
                    AnimatableProperty::Rotation => t.rotation = value,
                    AnimatableProperty::TranslateZ => t.translate_z = value,
                    AnimatableProperty::RotateX => t.rotate_x = value,
                    AnimatableProperty::RotateY => t.rotate_y = value,
                    AnimatableProperty::Perspective => t.perspective = value,
                    _ => {}
                }
            }
        }
        t
    }

    fn needs_projective_composite(t: &vidra_core::Transform2D) -> bool {
        // Use projective path for any non-trivial rotation or 2.5D parameters.
        t.rotation.abs() > f64::EPSILON
            || t.translate_z.abs() > f64::EPSILON
            || t.rotate_x.abs() > f64::EPSILON
            || t.rotate_y.abs() > f64::EPSILON
            || t.perspective > 0.0
    }

    /// Compute the animated position of a layer at a given frame.
    fn compute_layer_position(ctx: &RenderContext, layer: &Layer, frame: u64) -> (i32, i32) {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut x = layer.transform.position.x;
        let mut y = layer.transform.position.y;

        for anim in &layer.animations {
            if let Some(value) = Self::evaluate_animation(ctx, anim, time) {
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
            if let Some(value) = Self::evaluate_animation(ctx, anim, time) {
                match anim.property {
                    AnimatableProperty::ScaleX => sx = value,
                    AnimatableProperty::ScaleY => sy = value,
                    _ => {}
                }
            }
        }

        (sx, sy)
    }

    fn compute_layer_opacity(ctx: &RenderContext, layer: &Layer, frame: u64) -> f64 {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut opacity = layer.transform.opacity;

        for anim in &layer.animations {
            if let Some(value) = Self::evaluate_animation(ctx, anim, time) {
                if matches!(anim.property, AnimatableProperty::Opacity) {
                    opacity = value;
                }
            }
        }

        opacity.clamp(0.0, 1.0)
    }

    /// Compute the layer content and effects updated with animation values at the given frame.
    fn compute_layer_animated_state(ctx: &RenderContext, layer: &Layer, frame: u64) -> (LayerContent, Vec<vidra_core::types::LayerEffect>) {
        let time = vidra_core::Duration::from_seconds(frame as f64 / ctx.fps);
        let mut content = layer.content.clone();
        let mut effects = layer.effects.clone();

        for anim in &layer.animations {
            if let Some(value) = Self::evaluate_animation(ctx, anim, time) {
                match anim.property {
                    AnimatableProperty::ColorR => match &mut content {
                        LayerContent::Text { color, .. } => color.r = value as f32,
                        LayerContent::Solid { color } => color.r = value as f32,
                        LayerContent::Shape { fill, .. } => if let Some(c) = fill { c.r = value as f32; }
                        _ => {}
                    },
                    AnimatableProperty::ColorG => match &mut content {
                        LayerContent::Text { color, .. } => color.g = value as f32,
                        LayerContent::Solid { color } => color.g = value as f32,
                        LayerContent::Shape { fill, .. } => if let Some(c) = fill { c.g = value as f32; }
                        _ => {}
                    },
                    AnimatableProperty::ColorB => match &mut content {
                        LayerContent::Text { color, .. } => color.b = value as f32,
                        LayerContent::Solid { color } => color.b = value as f32,
                        LayerContent::Shape { fill, .. } => if let Some(c) = fill { c.b = value as f32; }
                        _ => {}
                    },
                    AnimatableProperty::ColorA => match &mut content {
                        LayerContent::Text { color, .. } => color.a = value as f32,
                        LayerContent::Solid { color } => color.a = value as f32,
                        LayerContent::Shape { fill, .. } => if let Some(c) = fill { c.a = value as f32; }
                        _ => {}
                    },
                    AnimatableProperty::FontSize => {
                        if let LayerContent::Text { font_size, .. } = &mut content {
                            *font_size = value;
                        }
                    },
                    AnimatableProperty::CornerRadius => {
                        if let LayerContent::Shape { shape: vidra_core::types::ShapeType::Rect { corner_radius, .. }, .. } = &mut content {
                            *corner_radius = value;
                        }
                    },
                    AnimatableProperty::StrokeWidth => {
                        if let LayerContent::Shape { stroke_width, .. } = &mut content {
                            *stroke_width = value;
                        }
                    },
                    AnimatableProperty::Volume => {
                        if let LayerContent::Audio { volume, .. } | LayerContent::TTS { volume, .. } = &mut content {
                            *volume = value;
                        }
                    }
                    AnimatableProperty::BlurRadius => {
                        for effect in &mut effects {
                            if let vidra_core::types::LayerEffect::Blur(radius) = effect {
                                *radius = value;
                            }
                        }
                    }
                    AnimatableProperty::BrightnessLevel => {
                        for effect in &mut effects {
                            if let vidra_core::types::LayerEffect::Brightness(level) = effect {
                                *level = value;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        (content, effects)
    }

    fn evaluate_animation(
        ctx: &RenderContext,
        anim: &vidra_ir::animation::Animation,
        time: vidra_core::Duration,
    ) -> Option<f64> {
        if let Some(expr) = anim.expr.as_deref() {
            let effective_secs = time.as_seconds() - anim.delay.as_seconds();
            if effective_secs < 0.0 {
                return None;
            }

            let duration_secs = anim
                .expr_duration
                .as_ref()
                .map(|d| d.as_seconds())
                .unwrap_or(0.0);
            let t = if duration_secs > 0.0 {
                effective_secs.min(duration_secs)
            } else {
                effective_secs
            };
            let p = if duration_secs > 0.0 {
                (t / duration_secs).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let compiled = build_operator_tree::<DefaultNumericTypes>(expr).ok()?;
            let mut context = HashMapContext::new();
            let _ = context.set_value("t".to_string(), Value::Float(t));
            let _ = context.set_value("p".to_string(), Value::Float(p));
            let _ = context.set_value("T".to_string(), Value::Float(duration_secs));
            let _ = context.set_value("mouse_x".to_string(), Value::Float(ctx.mouse_x));
            let _ = context.set_value("mouse_y".to_string(), Value::Float(ctx.mouse_y));
            let _ = context.set_value("audio_amp".to_string(), Value::Float(0.0));
            for (k, v) in &ctx.state_vars {
                if k != "t" && k != "p" && k != "T" && k != "mouse_x" && k != "mouse_y" && k != "audio_amp" {
                    let _ = context.set_value(k.clone(), Value::Float(*v));
                }
            }

            return compiled
                .eval_with_context(&context)
                .ok()
                .and_then(|v| v.as_number().ok());
        }

        anim.evaluate(time)
    }

    /// Determine whether a layer's content has an intrinsic bounding box
    /// (i.e., it's not a full-canvas fill). Only layers with intrinsic sizes
    /// should have anchor-point offsets applied.
    fn has_intrinsic_size(content: &LayerContent) -> bool {
        matches!(
            content,
            LayerContent::Text { .. }
                | LayerContent::Image { .. }
                | LayerContent::Video { .. }
                | LayerContent::Shape { .. }
                | LayerContent::TTS { .. }
                | LayerContent::AutoCaption { .. }
                | LayerContent::Waveform { .. }
        )
    }

    /// Apply anchor-point offset to the raw position.
    /// For full-canvas layers (Solid, Empty, Audio), the position is returned unchanged.
    fn apply_anchor(dx: i32, dy: i32, buf: &FrameBuffer, layer: &Layer, content: &LayerContent) -> (i32, i32) {
        if Self::has_intrinsic_size(content) {
            let cx = dx - (buf.width as f64 * layer.transform.anchor.x).round() as i32;
            let cy = dy - (buf.height as f64 * layer.transform.anchor.y).round() as i32;
            (cx, cy)
        } else {
            (dx, dy)
        }
    }

    /// Render a single layer to its own FrameBuffer.
    fn render_layer(
        &self,
        ctx: &RenderContext,
        project: &Project,
        layer: &Layer,
        content: &LayerContent,
        frame: u64,
    ) -> Result<FrameBuffer, vidra_core::VidraError> {
        let opacity = Self::compute_layer_opacity(ctx, layer, frame);

        let mut buf = match content {
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
            LayerContent::Spritesheet {
                asset_id,
                frame_width,
                frame_height,
                fps,
                start_frame,
                frame_count,
            } => {
                let sheet = self.load_image_asset(project, asset_id, opacity);
                self.render_spritesheet_frame(&sheet, *frame_width, *frame_height, *fps, *start_frame, *frame_count, frame, ctx.fps)
            }
            LayerContent::Waveform { .. } => {
                // If waveform materialization didn't run, show a placeholder.
                self.text_renderer
                    .render_text("[Waveform]", "Inter", 28.0, &Color::WHITE)
            }
            LayerContent::Video {
                asset_id,
                trim_start,
                ..
            } => self.render_video_frame(ctx, project, asset_id, trim_start, frame, opacity),
            LayerContent::TTS { text, .. } => {
                // Audio visualization component
                self.text_renderer
                    .render_text(&format!("🔊 {}", text), "Inter", 32.0, &Color::WHITE)
            }
            LayerContent::AutoCaption { .. } => {
                // AI AutoCaption component — fallback text display
                self.text_renderer
                    .render_text("[Auto Caption]", "Inter", 28.0, &Color::WHITE)
            }
            LayerContent::Shader { asset_id } => {
                // If we don't know the exact resolution of a custom shader, we default to the project resolution.
                // Or maybe the user set a custom scale/size on the layer. We'll default to project for this prototype phase.
                if let Some(source) = self.shader_cache.get(&asset_id.to_string()) {
                    let time_sec = frame as f32 / ctx.fps as f32;
                    match self.shader_renderer.render(source.value(), ctx.width, ctx.height, time_sec) {
                        Ok(fb) => fb,
                        Err(e) => {
                            tracing::error!("Custom shader render failed for {}: {}", asset_id.to_string(), e);
                            FrameBuffer::new(ctx.width, ctx.height, vidra_core::PixelFormat::Rgba8)
                        }
                    }
                } else {
                    tracing::warn!("Custom shader {} not found in cache", asset_id.to_string());
                    FrameBuffer::new(ctx.width, ctx.height, vidra_core::PixelFormat::Rgba8)
                }
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
            let (c_content, _) = Self::compute_layer_animated_state(ctx, child, frame);
            let child_buf = self.render_layer(ctx, project, child, &c_content, frame)?;
            let (dx, dy) = Self::compute_layer_position(ctx, child, frame);
            let (cx, cy) = Self::apply_anchor(dx, dy, &child_buf, child, &c_content);
            buf.composite_over(&child_buf, cx, cy);
        }

        Ok(buf)
    }

    fn render_spritesheet_frame(
        &self,
        sheet: &FrameBuffer,
        frame_w: u32,
        frame_h: u32,
        sheet_fps: f64,
        start_frame: u32,
        frame_count: Option<u32>,
        local_frame: u64,
        timeline_fps: f64,
    ) -> FrameBuffer {
        if sheet.format != vidra_core::frame::PixelFormat::Rgba8 || frame_w == 0 || frame_h == 0 {
            return sheet.clone();
        }

        let cols = (sheet.width / frame_w).max(1);
        let rows = (sheet.height / frame_h).max(1);
        let derived_total = cols.saturating_mul(rows) as u32;
        let total = frame_count.unwrap_or(derived_total).max(1).min(derived_total.max(1));

        let t = local_frame as f64 / timeline_fps;
        let idx = if sheet_fps <= 0.0 {
            0
        } else {
            ((t * sheet_fps).floor() as u32) % total
        };

        let frame_idx = start_frame.saturating_add(idx) % total;
        let x = (frame_idx % cols as u32) * frame_w;
        let y = (frame_idx / cols as u32) * frame_h;

        let mut out = FrameBuffer::new(frame_w, frame_h, vidra_core::frame::PixelFormat::Rgba8);
        for yy in 0..frame_h {
            for xx in 0..frame_w {
                if let Some(px) = sheet.get_pixel(x + xx, y + yy) {
                    out.set_pixel(xx, yy, px);
                }
            }
        }
        out
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
                // Return magenta fallback for missing images
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
        // Video layer with a missing asset should fall back to a cyan frame
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
