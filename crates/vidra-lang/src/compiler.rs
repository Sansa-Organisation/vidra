//! VidraScript compiler — AST → Vidra IR.

use crate::ast::*;
use vidra_core::{Color, VidraError};
use vidra_ir::animation::{AnimatableProperty, Animation};
use vidra_ir::asset::{Asset, AssetId, AssetType};
use vidra_ir::layer::{Layer, LayerContent, LayerId};
use vidra_ir::project::{Project, ProjectSettings};
use vidra_ir::scene::{Scene, SceneId};

use std::collections::HashMap;

/// Compiles a VidraScript AST into a Vidra IR Project.
pub struct Compiler {
    components: HashMap<String, ComponentNode>,
    layer_overrides: HashMap<String, Vec<PropertyNode>>,
}

impl Compiler {
    /// Compile a ProjectNode AST into a Project IR.
    pub fn compile(ast: &ProjectNode) -> Result<Project, VidraError> {
        let mut layer_overrides: HashMap<String, Vec<PropertyNode>> = HashMap::new();
        let target_aspect = ast.width as f64 / ast.height as f64;

        for rule_group in &ast.layout_rules {
            for rule in &rule_group.rules {
                if let Some((w_str, h_str)) = rule.aspect.split_once(':') {
                    if let (Ok(w), Ok(h)) = (w_str.parse::<f64>(), h_str.parse::<f64>()) {
                        let rule_aspect = w / h;
                        if (target_aspect - rule_aspect).abs() < 0.01 {
                            Self::extract_overrides(&rule.items, &mut layer_overrides);
                        }
                    }
                }
            }
        }

        let mut compiler = Self {
            components: HashMap::new(),
            layer_overrides,
        };
        
        for comp in &ast.components {
            compiler.components.insert(comp.name.clone(), comp.clone());
        }

        let settings = ProjectSettings::custom(ast.width, ast.height, ast.fps);
        let mut project = Project::new(settings);

        for asset in &ast.assets {
            let asset_type = match asset.asset_type.as_str() {
                "font" => vidra_ir::asset::AssetType::Font,
                "image" => vidra_ir::asset::AssetType::Image,
                "video" => vidra_ir::asset::AssetType::Video,
                "audio" => vidra_ir::asset::AssetType::Audio,
                _ => continue,
            };
            project.assets.register(vidra_ir::asset::Asset::new(
                vidra_ir::asset::AssetId::new(&asset.id),
                asset_type,
                &asset.path,
            ));
        }

        let mut global_env = HashMap::new();
        for var in &ast.variables {
            global_env.insert(var.name.clone(), var.value.clone());
        }

        for scene_node in &ast.scenes {
            let scene = compiler.compile_scene(scene_node, &mut project, &global_env)?;
            project.add_scene(scene);
        }

        Ok(project)
    }

    fn extract_overrides(items: &[LayerBlockItem], overrides: &mut HashMap<String, Vec<PropertyNode>>) {
        for item in items {
            if let LayerBlockItem::Layer(l) = item {
                overrides.entry(l.name.clone()).or_default().extend(l.properties.clone());
                Self::extract_overrides(&l.children, overrides);
            } else if let LayerBlockItem::If { then_branch, else_branch, .. } = item {
                Self::extract_overrides(then_branch, overrides);
                if let Some(eb) = else_branch {
                    Self::extract_overrides(eb, overrides);
                }
            }
        }
    }

    fn compile_scene(&self, scene_node: &SceneNode, project: &mut Project, global_env: &HashMap<String, ValueNode>) -> Result<Scene, VidraError> {
        let dur_val = match &scene_node.duration {
            ValueNode::Identifier(id) => global_env.get(id).unwrap_or(&scene_node.duration),
            other => other,
        };
        let duration_secs = Self::value_to_f64(dur_val)?;
        let duration = vidra_core::Duration::from_seconds(duration_secs);
        let mut scene = Scene::new(SceneId::new(&scene_node.name), duration);

        let mut staggers = Vec::new();
        for item in &scene_node.items {
            if let crate::ast::LayerBlockItem::Transition { transition_type, duration: dur_val, easing, span: _ } = item {
                let dur = Self::value_to_f64(dur_val)?;
                let ease = match easing.as_deref() {
                    Some("easeIn") => vidra_core::types::Easing::EaseIn,
                    Some("easeOut") => vidra_core::types::Easing::EaseOut,
                    Some("easeInOut") => vidra_core::types::Easing::EaseInOut,
                    _ => vidra_core::types::Easing::Linear,
                };
                let effect = match transition_type.as_str() {
                    "crossfade" => vidra_ir::transition::TransitionType::Crossfade,
                    "wipe" => vidra_ir::transition::TransitionType::Wipe { direction: "right".to_string() },
                    "push" => vidra_ir::transition::TransitionType::Push { direction: "right".to_string() },
                    "slide" => vidra_ir::transition::TransitionType::Slide { direction: "right".to_string() },
                    _ => vidra_ir::transition::TransitionType::Crossfade,
                };
                scene.transition = Some(vidra_ir::transition::Transition {
                    effect,
                    duration: vidra_core::Duration::from_seconds(dur),
                    easing: ease,
                });
            } else if let crate::ast::LayerBlockItem::AnimationStagger { args, animations, span: _ } = item {
                staggers.push((args.clone(), animations.clone()));
            } else {
                let compiled_layers = self.compile_layer_block_item(item, project, global_env, &[])?;
                for layer in compiled_layers {
                    scene.add_layer(layer);
                }
            }
        }

        for (args, animations) in staggers {
            let mut offset = 0.0;
            let mut target_layers: Vec<String> = Vec::new();
            
            for arg in &args {
                if arg.name == "offset" {
                    offset = Self::value_to_f64(&arg.value).unwrap_or(0.0);
                } else if arg.name == "layers" {
                    if let ValueNode::Array(arr) = &arg.value {
                        for item in arr {
                            if let Ok(layer_name) = Self::value_to_string(item) {
                                target_layers.push(layer_name);
                            }
                        }
                    }
                }
            }
            
            for (i, layer_name) in target_layers.iter().enumerate() {
                if let Some(layer) = scene.layers.iter_mut().find(|l| l.id.0.as_str() == layer_name) {
                    let total_delay_offset = offset * (i as f64);
                    
                    for anim_node in &animations {
                        if let PropertyNode::Animation { property, args, .. } = anim_node {
                            let mut anim = Self::compile_animation(property, args, global_env)?;
                            let existing_delay = anim.delay.as_seconds();
                            anim.delay = vidra_core::Duration::from_seconds(existing_delay + total_delay_offset);
                            layer.animations.push(anim);
                        }
                    }
                }
            }
        }

        Ok(scene)
    }

    fn is_truthy(val: &ValueNode) -> bool {
        match val {
            ValueNode::Number(n) => *n != 0.0,
            ValueNode::String(s) => !s.is_empty(),
            ValueNode::Color(_) => true,
            ValueNode::Duration(d) => *d > 0.0,
            ValueNode::Identifier(_) => true, // Usually identifiers themselves are truths unless evaled
            ValueNode::BrandReference(_) => true,
            ValueNode::Array(arr) => !arr.is_empty(),
        }
    }

    fn compile_layer_block_item(
        &self,
        item: &crate::ast::LayerBlockItem,
        project: &mut Project,
        env: &HashMap<String, ValueNode>,
        slots: &[Layer],
    ) -> Result<Vec<Layer>, VidraError> {
        match item {
            crate::ast::LayerBlockItem::Layer(layer_node) => {
                let l = self.compile_layer(layer_node, project, env, slots)?;
                Ok(vec![l])
            }
            crate::ast::LayerBlockItem::If { condition, then_branch, else_branch, .. } => {
                let eval_cond = if let ValueNode::Identifier(id) = condition {
                    env.get(id).unwrap_or(condition)
                } else {
                    condition
                };

                let mut out = Vec::new();
                if Self::is_truthy(eval_cond) {
                    for child in then_branch {
                        let mut compiled = self.compile_layer_block_item(child, project, env, slots)?;
                        out.append(&mut compiled);
                    }
                } else if let Some(else_branch) = else_branch {
                    for child in else_branch {
                        let mut compiled = self.compile_layer_block_item(child, project, env, slots)?;
                        out.append(&mut compiled);
                    }
                }
                Ok(out)
            }
            crate::ast::LayerBlockItem::Transition { .. } => {
                // Ignore, handled at scene level
                Ok(Vec::new())
            }
            crate::ast::LayerBlockItem::ComponentUse { .. } => {
                // Ignore for now, handled elsewhere or unused
                Ok(Vec::new())
            }
            crate::ast::LayerBlockItem::AnimationStagger { args: _, animations: _, span: _ } => {
                // To be implemented: apply stagger stagger to scene's matching layers
                Ok(Vec::new())
            }
        }
    }

    fn compile_layer(&self, layer_node: &LayerNode, project: &mut Project, env: &HashMap<String, ValueNode>, slots: &[Layer]) -> Result<Layer, VidraError> {
        let content = self.compile_layer_content(&layer_node.content, project, env)?;
        let mut layer = Layer::new(LayerId::new(&layer_node.name), content);

        // Apply layout overrides if any exist for this layer name
        let mut active_props = layer_node.properties.clone();
        if let Some(overrides) = self.layer_overrides.get(&layer_node.name) {
            active_props.extend(overrides.clone());
        }

        // Process properties
        for prop in &active_props {
            match prop {
                PropertyNode::Position { x, y, .. } => {
                    let mut resolved_x = x.clone();
                    if let ValueNode::Identifier(id) = x {
                        if let Some(val) = env.get(id) {
                            resolved_x = val.clone();
                        }
                    }
                    let mut resolved_y = y.clone();
                    if let ValueNode::Identifier(id) = y {
                        if let Some(val) = env.get(id) {
                            resolved_y = val.clone();
                        }
                    }

                    layer.transform.position.x = Self::value_to_f64(&resolved_x)?;
                    layer.transform.position.y = Self::value_to_f64(&resolved_y)?;
                }
                PropertyNode::Animation { property, args, .. } => {
                    let anim = Self::compile_animation(property, args, env)?;
                    layer.animations.push(anim);
                }
                PropertyNode::FunctionCall {
                    name,
                    args,
                    named_args: _,
                    ..
                } => {
                    if name == "effect" && !args.is_empty() {
                        let eff_name = if let ValueNode::Identifier(id) = &args[0] {
                            env.get(id).unwrap_or(&args[0])
                        } else { &args[0] };

                        if let Ok(effect_type) = Self::value_to_string(eff_name) {
                            match effect_type.as_str() {
                                "blur" => {
                                    let radius = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(0.0) } else { 0.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Blur(radius));
                                }
                                "grayscale" => {
                                    let intensity = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Grayscale(intensity));
                                }
                                "invert" => {
                                    let intensity = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Invert(intensity));
                                }
                                "brightness" => {
                                    let amount = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Brightness(amount));
                                }
                                "contrast" => {
                                    let amount = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Contrast(amount));
                                }
                                "saturation" => {
                                    let amount = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Saturation(amount));
                                }
                                "hue_rotate" | "hueRotate" => {
                                    let degrees = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(0.0) } else { 0.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::HueRotate(degrees));
                                }
                                "vignette" => {
                                    let amount = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(1.0) } else { 1.0 };
                                    layer.effects.push(vidra_core::types::LayerEffect::Vignette(amount));
                                }
                                _ => tracing::warn!("Unknown effect: {}", effect_type),
                            }
                        }
                    } else if name == "preset" && !args.is_empty() {
                        let preset_name = if let ValueNode::Identifier(id) = &args[0] {
                            env.get(id).unwrap_or(&args[0])
                        } else { &args[0] };
                        
                        let delay = if args.len() > 1 { Self::value_to_f64(&args[1]).unwrap_or(0.0) } else { 0.0 };
                        
                        if let Ok(preset_type) = Self::value_to_string(preset_name) {
                            match preset_type.as_str() {
                                "fadeInUp" => {
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::Opacity, 0.0, 1.0, vidra_core::Duration::from_seconds(0.5), vidra_core::types::Easing::EaseOut).with_delay(vidra_core::Duration::from_seconds(delay)));
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::PositionY, 50.0, 0.0, vidra_core::Duration::from_seconds(0.5), vidra_core::types::Easing::EaseOut).with_delay(vidra_core::Duration::from_seconds(delay)));
                                }
                                "bounceIn" => {
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::ScaleX, 0.0, 1.0, vidra_core::Duration::from_seconds(0.6), vidra_core::types::Easing::EaseOutBack).with_delay(vidra_core::Duration::from_seconds(delay)));
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::ScaleY, 0.0, 1.0, vidra_core::Duration::from_seconds(0.6), vidra_core::types::Easing::EaseOutBack).with_delay(vidra_core::Duration::from_seconds(delay)));
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::Opacity, 0.0, 1.0, vidra_core::Duration::from_seconds(0.3), vidra_core::types::Easing::Linear).with_delay(vidra_core::Duration::from_seconds(delay)));
                                }
                                "typewriter" => {
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::Opacity, 0.0, 1.0, vidra_core::Duration::from_seconds(0.2), vidra_core::types::Easing::Linear).with_delay(vidra_core::Duration::from_seconds(delay)));
                                }
                                "glitch" => {
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::PositionX, -10.0, 0.0, vidra_core::Duration::from_seconds(0.2), vidra_core::types::Easing::EaseIn).with_delay(vidra_core::Duration::from_seconds(delay)));
                                    layer.animations.push(vidra_ir::animation::Animation::from_to(vidra_ir::animation::AnimatableProperty::ScaleX, 1.5, 1.0, vidra_core::Duration::from_seconds(0.2), vidra_core::types::Easing::Linear).with_delay(vidra_core::Duration::from_seconds(delay)));
                                }
                                _ => tracing::warn!("Unknown preset: {}", preset_type),
                            }
                        }
                    } else if name == "mask" && !args.is_empty() {
                        let mask_layer_name = if let ValueNode::Identifier(id) = &args[0] {
                            env.get(id).unwrap_or(&args[0])
                        } else { &args[0] };
                        if let Ok(mask_str) = Self::value_to_string(mask_layer_name) {
                            layer.mask = Some(LayerId::new(mask_str));
                        }
                    } else {
                        // Handle generic function calls — extensible for enter/exit/etc.
                        tracing::debug!("unhandled function call: {}", name);
                    }
                }
                PropertyNode::AnimationGroup { animations, span: _ } => {
                    // All start at 0 (or delay inside)
                    for ag_prop in animations {
                        if let PropertyNode::Animation { property, args, .. } = ag_prop {
                            let anim = Self::compile_animation(property, args, env)?;
                            layer.animations.push(anim);
                        }
                    }
                }
                PropertyNode::AnimationSequence { animations, span: _ } => {
                    let mut current_time = 0.0;
                    for seq_prop in animations {
                        match seq_prop {
                            PropertyNode::Wait { duration, .. } => {
                                current_time += Self::value_to_f64(duration).unwrap_or(0.0);
                            }
                            PropertyNode::Animation { property, args, .. } => {
                                let mut anim = Self::compile_animation(property, args, env)?;
                                let delay = anim.delay.as_seconds() + current_time;
                                anim.delay = vidra_core::Duration::from_seconds(delay);
                                current_time += anim.duration().as_seconds();
                                layer.animations.push(anim);
                            }
                            _ => {} // Group within Sequence not yet handled
                        }
                    }
                }
                PropertyNode::Wait { .. } => {
                    // no-op if outside of a sequence
                }
            }
        }

        // Process children / Slots
        if let LayerContentNode::Component { name, args } = &layer_node.content {
            // For components, the children of this node act as slots. Compile them with outer env.
            let mut compiled_slots = Vec::new();
            for child_item in &layer_node.children {
                let mut compiled = self.compile_layer_block_item(child_item, project, env, &[])?;
                compiled_slots.append(&mut compiled);
            }

            if let Some(comp_def) = self.components.get(name) {
                let mut comp_env = env.clone();
                // 1. apply defaults
                for prop in &comp_def.props {
                    if let Some(def_val) = &prop.default_value {
                        comp_env.insert(prop.name.clone(), def_val.clone());
                    }
                }
                
                // 1.5 extract variant argument and apply overrides
                let mut active_variant = None;
                for arg in args {
                    if arg.name == "variant" {
                        let val = match &arg.value {
                            ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value).clone(),
                            _ => arg.value.clone(),
                        };
                        if let Ok(v_str) = Self::value_to_string(&val) {
                            active_variant = Some(v_str);
                        }
                    }
                }
                if let Some(v_name) = active_variant {
                    if let Some(v_def) = comp_def.variants.iter().find(|v| v.name == v_name) {
                        for override_arg in &v_def.overrides {
                            comp_env.insert(override_arg.name.clone(), override_arg.value.clone());
                        }
                    }
                }

                // 2. apply arguments (resolving from current environment if variables) overriding variants
                for arg in args {
                    // Do not expose variant to component props explicitly (optional)
                    if arg.name == "variant" { continue; }
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value).clone(),
                        _ => arg.value.clone(),
                    };
                    comp_env.insert(arg.name.clone(), val);
                }
                // 3. instantiate template layers, passing compiled_slots
                for comp_layer_item in &comp_def.items {
                    let compiled = self.compile_layer_block_item(comp_layer_item, project, &comp_env, &compiled_slots)?;
                    for mut child_layer in compiled {
                        child_layer.id = vidra_ir::layer::LayerId::new(format!("{}_{}", layer.id.0, child_layer.id.0));
                        layer.add_child(child_layer);
                    }
                }
            } else {
                return Err(VidraError::Compile(format!("unknown component: {}", name)));
            }
        } else {
            // Standard layer, process its explicit children normally
            for child_item in &layer_node.children {
                let compiled = self.compile_layer_block_item(child_item, project, env, slots)?;
                for child in compiled {
                    layer.add_child(child);
                }
            }
        }

        if matches!(layer_node.content, LayerContentNode::Slot) {
            for slot_layer in slots {
                layer.add_child(slot_layer.clone());
            }
        }

        Ok(layer)
    }

    fn compile_layer_content(
        &self,
        content: &LayerContentNode,
        project: &mut Project,
        env: &HashMap<String, ValueNode>,
    ) -> Result<LayerContent, VidraError> {
        match content {
            LayerContentNode::Text { text, args } => {
                let mut font_family = "Inter".to_string();
                let mut font_size = 24.0;
                let mut color = Color::WHITE;

                for arg in args {
                    let val = env.get(&arg.name).unwrap_or(&arg.value);
                    match arg.name.as_str() {
                        "font" => font_family = Self::value_to_string(val)?,
                        "size" => font_size = Self::value_to_f64(val)?,
                        "color" => color = Self::value_to_color(val)?,
                        _ => {}
                    }
                }

                let text_val = if let ValueNode::Identifier(id) = text {
                    env.get(id).unwrap_or(text)
                } else { text };
                let resolved_text = Self::value_to_string(text_val)?;

                Ok(LayerContent::Text {
                    text: resolved_text,
                    font_family,
                    font_size,
                    color,
                })
            }
            LayerContentNode::Image { path, args: _args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else { path };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());
                
                if project.assets.get(&asset_id).is_none() {
                    project
                        .assets
                        .register(Asset::new(asset_id.clone(), AssetType::Image, resolved_path));
                }
                
                Ok(LayerContent::Image { asset_id })
            }
            LayerContentNode::Video { path, args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else { path };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());
                
                if project.assets.get(&asset_id).is_none() {
                    project
                        .assets
                        .register(Asset::new(asset_id.clone(), AssetType::Video, resolved_path));
                }
                
                let mut trim_start = vidra_core::Duration::zero();
                let mut trim_end = None;

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "trim_start" => trim_start = vidra_core::Duration::from_seconds(Self::value_to_f64(val)?),
                        "trim_end" => trim_end = Some(vidra_core::Duration::from_seconds(Self::value_to_f64(val)?)),
                        _ => {}
                    }
                }
                
                Ok(LayerContent::Video {
                    asset_id,
                    trim_start,
                    trim_end,
                })
            }
            LayerContentNode::Audio { path, args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else { path };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());
                
                if project.assets.get(&asset_id).is_none() {
                    project
                        .assets
                        .register(Asset::new(asset_id.clone(), AssetType::Audio, resolved_path));
                }
                
                let mut trim_start = vidra_core::Duration::zero();
                let mut trim_end = None;
                let mut volume = 1.0;

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "trim_start" => trim_start = vidra_core::Duration::from_seconds(Self::value_to_f64(val)?),
                        "trim_end" => trim_end = Some(vidra_core::Duration::from_seconds(Self::value_to_f64(val)?)),
                        "volume" => volume = Self::value_to_f64(val)?,
                        _ => {}
                    }
                }
                
                Ok(LayerContent::Audio {
                    asset_id,
                    trim_start,
                    trim_end,
                    volume,
                })
            }
            LayerContentNode::TTS { text, voice, args } => {
                let resolved_text = Self::value_to_string(text)?;
                let resolved_voice = Self::value_to_string(voice)?;
                let mut volume = 1.0;
                
                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    if arg.name == "volume" {
                        volume = Self::value_to_f64(val)?;
                    }
                }
                
                Ok(LayerContent::TTS {
                    text: resolved_text,
                    voice: resolved_voice,
                    volume,
                })
            }
            LayerContentNode::AutoCaption { audio_source, args } => {
                let path_str = Self::value_to_string(if let ValueNode::Identifier(id) = audio_source {
                    env.get(id).unwrap_or(audio_source)
                } else { audio_source })?;
                let asset_id = AssetId::new(path_str);
                
                let mut font_family = "Inter".to_string();
                let mut font_size = 48.0;
                let mut color = Color::WHITE;
                
                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "font" => font_family = Self::value_to_string(val)?,
                        "size" => font_size = Self::value_to_f64(val)?,
                        "color" => color = Self::value_to_color(val)?,
                        _ => {}
                    }
                }
                
                Ok(LayerContent::AutoCaption {
                    asset_id,
                    font_family,
                    font_size,
                    color,
                })
            }
            LayerContentNode::Solid { color } => {
                let color_val = if let ValueNode::Identifier(id) = color {
                    env.get(id).unwrap_or(color)
                } else { color };
                let c = Self::value_to_color(color_val)?;
                Ok(LayerContent::Solid { color: c })
            }
            LayerContentNode::Shape { shape_type, args } => {
                let mut stroke_color = None;
                let mut stroke_w = 0.0;
                let mut fill_color = Some(Color::WHITE);

                let get_val = |key: &str| -> Option<ValueNode> {
                    args.iter().find(|a| a.name == key).map(|a| {
                        if let ValueNode::Identifier(id) = &a.value {
                            env.get(id).cloned().unwrap_or(a.value.clone())
                        } else {
                            a.value.clone()
                        }
                    })
                };

                if let Some(c) = get_val("fill").or_else(|| get_val("color")) {
                    fill_color = Some(Self::value_to_color(&c)?);
                }
                if let Some(c) = get_val("stroke") {
                    stroke_color = Some(Self::value_to_color(&c)?);
                }
                if let Some(w) = get_val("strokeWidth") {
                    stroke_w = Self::value_to_f64(&w)?;
                }

                let shape = match shape_type.as_str() {
                    "rect" | "rectangle" => {
                        let width = get_val("width").map(|v| Self::value_to_f64(&v).unwrap_or(100.0)).unwrap_or(100.0);
                        let height = get_val("height").map(|v| Self::value_to_f64(&v).unwrap_or(100.0)).unwrap_or(100.0);
                        let corner_radius = get_val("cornerRadius").map(|v| Self::value_to_f64(&v).unwrap_or(0.0)).unwrap_or(0.0);
                        vidra_core::types::ShapeType::Rect { width, height, corner_radius }
                    }
                    "circle" => {
                        let radius = get_val("radius").map(|v| Self::value_to_f64(&v).unwrap_or(50.0)).unwrap_or(50.0);
                        vidra_core::types::ShapeType::Circle { radius }
                    }
                    "ellipse" => {
                        let rx = get_val("rx").map(|v| Self::value_to_f64(&v).unwrap_or(50.0)).unwrap_or(50.0);
                        let ry = get_val("ry").map(|v| Self::value_to_f64(&v).unwrap_or(50.0)).unwrap_or(50.0);
                        vidra_core::types::ShapeType::Ellipse { rx, ry }
                    }
                    _ => {
                        return Err(VidraError::Compile(
                            format!("unknown shape type: {}", shape_type),
                        ));
                    }
                };

                Ok(LayerContent::Shape {
                    shape,
                    fill: fill_color,
                    stroke: stroke_color,
                    stroke_width: stroke_w,
                })
            }
            LayerContentNode::Component { .. } | LayerContentNode::Slot | LayerContentNode::Empty => Ok(LayerContent::Empty),
        }
    }

    fn compile_animation(property: &str, args: &[NamedArg], env: &HashMap<String, ValueNode>) -> Result<Animation, VidraError> {
        let animatable = match property {
            "opacity" => AnimatableProperty::Opacity,
            "position.x" | "positionX" | "x" => AnimatableProperty::PositionX,
            "position.y" | "positionY" | "y" => AnimatableProperty::PositionY,
            "scale.x" | "scaleX" => AnimatableProperty::ScaleX,
            "scale.y" | "scaleY" => AnimatableProperty::ScaleY,
            "scale" => AnimatableProperty::ScaleX, // convenience
            "rotation" => AnimatableProperty::Rotation,
            _ => {
                return Err(VidraError::Compile(format!(
                    "unknown animatable property: {}",
                    property
                )));
            }
        };

        let mut from_val = 0.0;
        let mut to_val = 1.0;
        let mut duration = 1.0;
        let mut easing = vidra_core::types::Easing::Linear;
        let mut delay = 0.0;

        for arg in args {
            let val = env.get(&arg.name).unwrap_or(&arg.value);
            // Re-resolve if the value itself is an identifier (e.g. `duration: compDelay`)
            let resolved_val = if let ValueNode::Identifier(id) = val {
                env.get(id).unwrap_or(val)
            } else { val };

            match arg.name.as_str() {
                "from" => from_val = Self::value_to_f64(resolved_val)?,
                "to" => to_val = Self::value_to_f64(resolved_val)?,
                "duration" => duration = Self::value_to_duration(resolved_val)?,
                "delay" => delay = Self::value_to_duration(resolved_val)?,
                "ease" | "easing" => {
                    easing = Self::value_to_easing(resolved_val)?;
                }
                _ => {}
            }
        }

        let mut anim = Animation::from_to(
            animatable,
            from_val,
            to_val,
            vidra_core::Duration::from_seconds(duration),
            easing,
        );

        if delay > 0.0 {
            anim = anim.with_delay(vidra_core::Duration::from_seconds(delay));
        }

        Ok(anim)
    }

    // --- Value converters ---

    fn value_to_f64(value: &ValueNode) -> Result<f64, VidraError> {
        match value {
            ValueNode::Number(n) => Ok(*n),
            ValueNode::Duration(d) => Ok(*d),
            ValueNode::BrandReference(key) => {
                if key.contains("radius") || key.contains("size") {
                    Ok(12.0)
                } else {
                    Ok(1.0)
                }
            }
            _ => Err(VidraError::Compile(format!(
                "expected number, got {:?}",
                value
            ))),
        }
    }

    fn value_to_string(value: &ValueNode) -> Result<String, VidraError> {
        match value {
            ValueNode::String(s) => Ok(s.clone()),
            ValueNode::Identifier(s) => Ok(s.clone()),
            ValueNode::BrandReference(key) => {
                if key.contains("font") {
                    Ok("Inter".to_string())
                } else if key.contains("logo") {
                    Ok("brand_logo.png".to_string())
                } else {
                    Ok(format!("brand_{}", key))
                }
            }
            _ => Err(VidraError::Compile(format!(
                "expected string, got {:?}",
                value
            ))),
        }
    }

    fn value_to_color(value: &ValueNode) -> Result<Color, VidraError> {
        match value {
            ValueNode::Color(hex) => Color::from_hex(hex)
                .map_err(|e| VidraError::Compile(format!("invalid color: {}", e))),
            ValueNode::BrandReference(key) => {
                let hex = if key.contains("bg") || key.contains("background") {
                    "111111"
                } else if key.contains("text") || key.contains("primary") {
                    "FFFFFF"
                } else if key.contains("accent") {
                    "FF3366"
                } else {
                    "888888"
                };
                Color::from_hex(hex).map_err(|e| VidraError::Compile(format!("invalid brand color fallback: {}", e)))
            }
            _ => Err(VidraError::Compile(format!(
                "expected color, got {:?}",
                value
            ))),
        }
    }

    fn value_to_duration(value: &ValueNode) -> Result<f64, VidraError> {
        match value {
            ValueNode::Duration(d) => Ok(*d),
            ValueNode::Number(n) => Ok(*n),
            ValueNode::BrandReference(_) => Ok(1.0), // mock
            _ => Err(VidraError::Compile(format!(
                "expected duration, got {:?}",
                value
            ))),
        }
    }

    fn value_to_easing(value: &ValueNode) -> Result<vidra_core::types::Easing, VidraError> {
        let name = Self::value_to_string(value)?;
        match name.as_str() {
            "linear" => Ok(vidra_core::types::Easing::Linear),
            "easeIn" => Ok(vidra_core::types::Easing::EaseIn),
            "easeOut" => Ok(vidra_core::types::Easing::EaseOut),
            "easeInOut" => Ok(vidra_core::types::Easing::EaseInOut),
            "cubicIn" => Ok(vidra_core::types::Easing::CubicIn),
            "cubicOut" => Ok(vidra_core::types::Easing::CubicOut),
            "cubicInOut" => Ok(vidra_core::types::Easing::CubicInOut),
            "easeOutBack" => Ok(vidra_core::types::Easing::EaseOutBack),
            _ => Err(VidraError::Compile(format!(
                "unknown easing function: {}",
                name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(src: &str) -> Project {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, "test.vidra");
        let ast = parser.parse().unwrap();
        Compiler::compile(&ast).unwrap()
    }

    #[test]
    fn test_compile_basic_project() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("intro", 5s) {
                    layer("bg") {
                        solid(#000000)
                    }
                }
            }
        "#,
        );

        assert_eq!(project.settings.width, 1920);
        assert_eq!(project.settings.height, 1080);
        assert_eq!(project.scenes.len(), 1);
        assert_eq!(project.scenes[0].layers.len(), 1);

        match &project.scenes[0].layers[0].content {
            LayerContent::Solid { color } => {
                assert_eq!(color.to_rgba8(), [0, 0, 0, 255]);
            }
            _ => panic!("expected solid layer"),
        }
    }

    #[test]
    fn test_compile_text_layer() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("s", 3s) {
                    layer("title") {
                        text("Hello", font: "Inter Bold", size: 72, color: #FFFFFF)
                        position(100, 200)
                    }
                }
            }
        "#,
        );

        let layer = &project.scenes[0].layers[0];
        match &layer.content {
            LayerContent::Text {
                text,
                font_family,
                font_size,
                color,
            } => {
                assert_eq!(text, "Hello");
                assert_eq!(font_family, "Inter Bold");
                assert!((font_size - 72.0).abs() < 0.001);
                assert_eq!(color.to_rgba8(), [255, 255, 255, 255]);
            }
            _ => panic!("expected text layer"),
        }
        assert!((layer.transform.position.x - 100.0).abs() < 0.001);
        assert!((layer.transform.position.y - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_compile_animation() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("s", 5s) {
                    layer("bg") {
                        solid(#0000FF)
                        animation(opacity, from: 0, to: 1, duration: 2s, ease: easeInOut)
                    }
                }
            }
        "#,
        );

        let layer = &project.scenes[0].layers[0];
        assert_eq!(layer.animations.len(), 1);
        assert_eq!(layer.animations[0].property, AnimatableProperty::Opacity);
        assert_eq!(layer.animations[0].keyframes.len(), 2);
    }

    #[test]
    fn test_compile_end_to_end_with_render() {
        let project = compile(
            r#"
            project(320, 240, 10) {
                scene("test", 1s) {
                    layer("bg") {
                        solid(#FF0000)
                    }
                    layer("text") {
                        text("Hello Vidra", font: "Inter", size: 48, color: #FFFFFF)
                        position(50, 100)
                        animation(opacity, from: 0, to: 1, duration: 1s, ease: linear)
                    }
                }
            }
        "#,
        );

        // Verify the project can be rendered
        let result = vidra_render::RenderPipeline::render(&project).unwrap();
        assert_eq!(result.frame_count, 10);
        assert_eq!(result.frames.len(), 10);
    }
    #[test]
    fn test_compile_component() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                component("Button", label_text: String, color: Color = #FF0000, t_duration: Duration = 1s, posX: Number = 100) {
                    layer("bg") {
                        solid(color)
                        animation(opacity, from: 0, to: 1, duration: t_duration, ease: linear)
                    }
                    layer("label") {
                        text(label_text, font: "Inter", size: 24, color: #FFFFFF)
                        position(posX, 50)
                    }
                }

                scene("main", 5s) {
                    layer("btn1") {
                        Button(label_text: "Click Me", color: #0000FF)
                        position(500, 500)
                    }
                }
            }
        "#,
        );

        let main_scene = &project.scenes[0];
        assert_eq!(main_scene.layers.len(), 1);
        
        let btn1_wrapper = &main_scene.layers[0];
        assert!(matches!(btn1_wrapper.content, LayerContent::Empty));
        assert_eq!(btn1_wrapper.children.len(), 2);
        assert!((btn1_wrapper.transform.position.x - 500.0).abs() < 0.001);

        let btn_bg = &btn1_wrapper.children[0];
        assert!(matches!(btn_bg.content, LayerContent::Solid { color: c } if c.to_rgba8() == [0, 0, 255, 255]));
        assert_eq!(btn_bg.animations[0].duration().as_seconds(), 1.0); // Default t_duration
        
        let btn_label = &btn1_wrapper.children[1];
        assert!(matches!(&btn_label.content, LayerContent::Text { text, .. } if text == "Click Me"));
        assert!((btn_label.transform.position.x - 100.0).abs() < 0.001); // Default posX
    }

    #[test]
    fn test_compile_slot_component() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                component("Box") {
                    layer("bg") {
                        solid(#FF0000)
                    }
                    layer("content") {
                        slot()
                        position(10, 10)
                    }
                }

                scene("main", 5s) {
                    layer("box1") {
                        Box()
                        position(50, 50)
                        
                        layer("slotted_child") {
                            text("Inside Slot")
                        }
                    }
                }
            }
        "#,
        );
        let main_scene = &project.scenes[0];
        let box1 = &main_scene.layers[0];
        // box1 should have 'bg' and 'content' as children
        assert_eq!(box1.children.len(), 2);
        
        let box_content = &box1.children[1];
        assert!(matches!(box_content.content, LayerContent::Empty));
        assert!((box_content.transform.position.x - 10.0).abs() < 0.001);
        
        // box_content should contain the slotted child
        assert_eq!(box_content.children.len(), 1);
        let slotted = &box_content.children[0];
        assert!(matches!(&slotted.content, LayerContent::Text { text, .. } if text == "Inside Slot"));
    }

    #[test]
    fn test_compile_conditional_logic() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                component("Box", showRed: Number = 0) {
                    if (showRed) {
                        layer("red_bg") {
                            solid(#FF0000)
                        }
                    } else {
                        layer("blue_bg") {
                            solid(#0000FF)
                        }
                    }
                }

                scene("main", 5s) {
                    layer("btn_y") {
                        Box(showRed: 1)
                    }
                    layer("btn_n") {
                        Box(showRed: 0)
                    }
                }
            }
        "#,
        );
        let main_scene = &project.scenes[0];
        
        let btn_y = &main_scene.layers[0];
        assert_eq!(btn_y.children.len(), 1); // Should have "red_bg" derived from showRed: 1
        assert_eq!(btn_y.children[0].id.0.as_str(), "btn_y_red_bg");

        let btn_n = &main_scene.layers[1];
        assert_eq!(btn_n.children.len(), 1); // Should have "blue_bg" derived from showRed: 0
        assert_eq!(btn_n.children[0].id.0.as_str(), "btn_n_blue_bg");
    }

    #[test]
    fn test_compile_variants() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                component("Box", color: Color = #FFFFFF) {
                    variant("primary", color: #0000FF)
                    variant("danger", color: #FF0000)

                    layer("bg") {
                        solid(color)
                    }
                }

                scene("main", 5s) {
                    layer("btn_default") { Box() }
                    layer("btn_primary") { Box(variant: "primary") }
                    layer("btn_danger") { Box(variant: "danger") }
                }
            }
        "#,
        );

        let main_scene = &project.scenes[0];
        assert_eq!(main_scene.layers.len(), 3);
        
        let btn_default = &main_scene.layers[0];
        assert_eq!(btn_default.children.len(), 1);
        match &btn_default.children[0].content {
            LayerContent::Solid { color } => assert_eq!(color.to_rgba8(), [255, 255, 255, 255]),
            _ => panic!("Expected solid content"),
        }

        let btn_primary = &main_scene.layers[1];
        assert_eq!(btn_primary.children.len(), 1);
        match &btn_primary.children[0].content {
            LayerContent::Solid { color } => assert_eq!(color.to_rgba8(), [0, 0, 255, 255]),
            _ => panic!("Expected solid content"),
        }

        let btn_danger = &main_scene.layers[2];
        assert_eq!(btn_danger.children.len(), 1);
        match &btn_danger.children[0].content {
            LayerContent::Solid { color } => assert_eq!(color.to_rgba8(), [255, 0, 0, 255]),
            _ => panic!("Expected solid content"),
        }
    }

    #[test]
    fn test_compile_layout_rules() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                layout rules {
                    when aspect(16:9) {
                        layer("box") { position(100, 100) }
                    }
                    when aspect(9:16) {
                        layer("box") { position(50, 50) }
                    }
                }

                scene("main", 5s) {
                    layer("box") { 
                        solid(#FFFFFF)
                        position(0, 0)
                    }
                }
            }
            "#,
        );
        
        let main_scene = &project.scenes[0];
        assert_eq!(main_scene.layers.len(), 1);
        
        let box_layer = &main_scene.layers[0];
        // The project is 1920x1080 (16:9 aspect), so it should match the first rule
        // which sets the position to 100, 100
        assert_eq!(box_layer.transform.position.x, 100.0);
        assert_eq!(box_layer.transform.position.y, 100.0);
    }

    #[test]
    fn test_compile_variables_and_presets() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                @var dur = 5
                @var accent = #FFCC00
                @var entrance = 1.0

                scene("main", dur) {
                    layer("box") { 
                        solid(accent)
                        preset("fadeInUp", entrance)
                    }
                }
            }
            "#,
        );
        let scene = &project.scenes[0];
        assert_eq!(scene.duration.as_seconds(), 5.0);
        let box_layer = &scene.layers[0];
        assert_eq!(box_layer.animations.len(), 2);
        
        match &box_layer.content {
            LayerContent::Solid { color } => assert_eq!(color.to_rgba8(), [255, 204, 0, 255]),
            _ => panic!("Expected solid content"),
        }
    }

    #[test]
    fn test_compile_animation_features() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("main", 5s) {
                    layer("item1") { text("1") }
                    layer("item2") { text("2") }
                    
                    animate.stagger(layers: ["item1", "item2"], offset: 0.5) {
                        animation(opacity, from: 0.0, to: 1.0, duration: 1.0)
                    }
                    
                    layer("seq_item") {
                        text("seq")
                        animate.sequence {
                            animation(opacity, from: 0.0, to: 1.0, duration: 1.0)
                            wait(0.5)
                            animation(scale, from: 1.0, to: 2.0, duration: 1.0)
                        }
                    }
                }
            }
        "#,
        );
        let scene = &project.scenes[0];
        
        let item1 = &scene.layers[0];
        assert_eq!(item1.animations.len(), 1);
        assert_eq!(item1.animations[0].delay.as_seconds(), 0.0);
        
        let item2 = &scene.layers[1];
        assert_eq!(item2.animations.len(), 1);
        assert_eq!(item2.animations[0].delay.as_seconds(), 0.5);
        
        let seq_item = &scene.layers[2];
        assert_eq!(seq_item.animations.len(), 2);
        assert_eq!(seq_item.animations[0].delay.as_seconds(), 0.0);
        assert_eq!(seq_item.animations[1].delay.as_seconds(), 1.5);
    }
}
