//! VidraScript compiler — AST → Vidra IR.

use crate::ast::*;
use vidra_core::{Color, VidraError};
use vidra_ir::animation::{AnimatableProperty, Animation};
use vidra_ir::asset::{Asset, AssetId, AssetType};
use vidra_ir::layer::{Layer, LayerContent, LayerId};
use vidra_ir::project::{Project, ProjectSettings};
use vidra_ir::scene::{Scene, SceneId};

use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};

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
                "lut" => vidra_ir::asset::AssetType::Lut,
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

    fn extract_overrides(
        items: &[LayerBlockItem],
        overrides: &mut HashMap<String, Vec<PropertyNode>>,
    ) {
        for item in items {
            if let LayerBlockItem::Layer(l) = item {
                overrides
                    .entry(l.name.clone())
                    .or_default()
                    .extend(l.properties.clone());
                Self::extract_overrides(&l.children, overrides);
            } else if let LayerBlockItem::If {
                then_branch,
                else_branch,
                ..
            } = item
            {
                Self::extract_overrides(then_branch, overrides);
                if let Some(eb) = else_branch {
                    Self::extract_overrides(eb, overrides);
                }
            }
        }
    }

    fn compile_scene(
        &self,
        scene_node: &SceneNode,
        project: &mut Project,
        global_env: &HashMap<String, ValueNode>,
    ) -> Result<Scene, VidraError> {
        let dur_val = match &scene_node.duration {
            ValueNode::Identifier(id) => global_env.get(id).unwrap_or(&scene_node.duration),
            other => other,
        };
        let duration_secs = Self::value_to_f64(dur_val)?;
        let duration = vidra_core::Duration::from_seconds(duration_secs);
        let mut scene = Scene::new(SceneId::new(&scene_node.name), duration);

        let mut staggers = Vec::new();
        for item in &scene_node.items {
            if let crate::ast::LayerBlockItem::Transition {
                transition_type,
                duration: dur_val,
                easing,
                span: _,
            } = item
            {
                let dur = Self::value_to_f64(dur_val)?;
                let ease = match easing.as_deref() {
                    Some("easeIn") => vidra_core::types::Easing::EaseIn,
                    Some("easeOut") => vidra_core::types::Easing::EaseOut,
                    Some("easeInOut") => vidra_core::types::Easing::EaseInOut,
                    _ => vidra_core::types::Easing::Linear,
                };
                let effect = match transition_type.as_str() {
                    "crossfade" => vidra_ir::transition::TransitionType::Crossfade,
                    "wipe" => vidra_ir::transition::TransitionType::Wipe {
                        direction: "right".to_string(),
                    },
                    "push" => vidra_ir::transition::TransitionType::Push {
                        direction: "right".to_string(),
                    },
                    "slide" => vidra_ir::transition::TransitionType::Slide {
                        direction: "right".to_string(),
                    },
                    _ => vidra_ir::transition::TransitionType::Crossfade,
                };
                scene.transition = Some(vidra_ir::transition::Transition {
                    effect,
                    duration: vidra_core::Duration::from_seconds(dur),
                    easing: ease,
                });
            } else if let crate::ast::LayerBlockItem::AnimationStagger {
                args,
                animations,
                span: _,
            } = item
            {
                staggers.push((args.clone(), animations.clone()));
            } else {
                let compiled_layers =
                    self.compile_layer_block_item(item, project, global_env, &[])?;
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
                if let Some(layer) = scene
                    .layers
                    .iter_mut()
                    .find(|l| l.id.0.as_str() == layer_name)
                {
                    let total_delay_offset = offset * (i as f64);

                    for anim_node in &animations {
                        if let PropertyNode::Animation { property, args, .. } = anim_node {
                            let anims = Self::compile_animation(property, args, global_env)?;
                            for mut anim in anims {
                                let existing_delay = anim.delay.as_seconds();
                                anim.delay = vidra_core::Duration::from_seconds(
                                    existing_delay + total_delay_offset,
                                );
                                layer.animations.push(anim);
                            }
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
            crate::ast::LayerBlockItem::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let eval_cond = if let ValueNode::Identifier(id) = condition {
                    env.get(id).unwrap_or(condition)
                } else {
                    condition
                };

                let mut out = Vec::new();
                if Self::is_truthy(eval_cond) {
                    for child in then_branch {
                        let mut compiled =
                            self.compile_layer_block_item(child, project, env, slots)?;
                        out.append(&mut compiled);
                    }
                } else if let Some(else_branch) = else_branch {
                    for child in else_branch {
                        let mut compiled =
                            self.compile_layer_block_item(child, project, env, slots)?;
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
            crate::ast::LayerBlockItem::AnimationStagger {
                args: _,
                animations: _,
                span: _,
            } => {
                // To be implemented: apply stagger stagger to scene's matching layers
                Ok(Vec::new())
            }
        }
    }

    fn compile_layer(
        &self,
        layer_node: &LayerNode,
        project: &mut Project,
        env: &HashMap<String, ValueNode>,
        slots: &[Layer],
    ) -> Result<Layer, VidraError> {
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
                    let anims = Self::compile_animation(property, args, env)?;
                    layer.animations.extend(anims);
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
                        } else {
                            &args[0]
                        };

                        if let Ok(effect_type) = Self::value_to_string(eff_name) {
                            match effect_type.as_str() {
                                "blur" => {
                                    let radius = if args.len() > 1 {
                                        Self::value_to_f64(&args[1]).unwrap_or(0.0)
                                    } else {
                                        0.0
                                    };
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::Blur(radius));
                                }
                                "grayscale" => {
                                    let intensity_val = if args.len() > 1 {
                                        Self::value_to_f64(&args[1])
                                    } else {
                                        Ok(1.0)
                                    };
                                    if let Ok(intensity) = intensity_val {
                                        let src = format!(
                                            "@effect __vidra_grayscale() {{\n    let c = source() -> grayscale({})\n    c\n}}\n",
                                            intensity
                                        );
                                        match vidra_fx::compile(&src) {
                                            Ok(wgsl) => layer.effects.push(
                                                vidra_core::types::LayerEffect::CustomShader {
                                                    wgsl_source: wgsl,
                                                },
                                            ),
                                            Err(_) => layer.effects.push(
                                                vidra_core::types::LayerEffect::Grayscale(
                                                    intensity,
                                                ),
                                            ),
                                        }
                                    } else {
                                        layer
                                            .effects
                                            .push(vidra_core::types::LayerEffect::Grayscale(1.0));
                                    }
                                }
                                "invert" => {
                                    let intensity_val = if args.len() > 1 {
                                        Self::value_to_f64(&args[1])
                                    } else {
                                        Ok(1.0)
                                    };
                                    if let Ok(intensity) = intensity_val {
                                        let src = format!(
                                            "@effect __vidra_invert() {{\n    let c = source() -> invert({})\n    c\n}}\n",
                                            intensity
                                        );
                                        match vidra_fx::compile(&src) {
                                            Ok(wgsl) => layer.effects.push(
                                                vidra_core::types::LayerEffect::CustomShader {
                                                    wgsl_source: wgsl,
                                                },
                                            ),
                                            Err(_) => layer.effects.push(
                                                vidra_core::types::LayerEffect::Invert(intensity),
                                            ),
                                        }
                                    } else {
                                        layer
                                            .effects
                                            .push(vidra_core::types::LayerEffect::Invert(1.0));
                                    }
                                }
                                "brightness" => {
                                    let amount_val = if args.len() > 1 {
                                        Self::value_to_f64(&args[1])
                                    } else {
                                        Ok(1.0)
                                    };
                                    if let Ok(amount) = amount_val {
                                        let src = format!(
                                            "@effect __vidra_brightness() {{\n    let c = source() -> brightness({})\n    c\n}}\n",
                                            amount
                                        );
                                        match vidra_fx::compile(&src) {
                                            Ok(wgsl) => layer.effects.push(
                                                vidra_core::types::LayerEffect::CustomShader {
                                                    wgsl_source: wgsl,
                                                },
                                            ),
                                            Err(_) => layer.effects.push(
                                                vidra_core::types::LayerEffect::Brightness(amount),
                                            ),
                                        }
                                    } else {
                                        layer
                                            .effects
                                            .push(vidra_core::types::LayerEffect::Brightness(1.0));
                                    }
                                }
                                "contrast" => {
                                    let amount = if args.len() > 1 {
                                        Self::value_to_f64(&args[1]).unwrap_or(1.0)
                                    } else {
                                        1.0
                                    };
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::Contrast(amount));
                                }
                                "saturation" => {
                                    let amount = if args.len() > 1 {
                                        Self::value_to_f64(&args[1]).unwrap_or(1.0)
                                    } else {
                                        1.0
                                    };
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::Saturation(amount));
                                }
                                "hue_rotate" | "hueRotate" => {
                                    let degrees = if args.len() > 1 {
                                        Self::value_to_f64(&args[1]).unwrap_or(0.0)
                                    } else {
                                        0.0
                                    };
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::HueRotate(degrees));
                                }
                                "vignette" => {
                                    let amount = if args.len() > 1 {
                                        Self::value_to_f64(&args[1]).unwrap_or(1.0)
                                    } else {
                                        1.0
                                    };
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::Vignette(amount));
                                }
                                "removeBackground" | "remove_background" | "remove-bg" => {
                                    layer
                                        .effects
                                        .push(vidra_core::types::LayerEffect::RemoveBackground);
                                }
                                "lut" | "LUT" => {
                                    if args.len() < 2 {
                                        tracing::warn!(
                                            "effect(lut, ...) requires a path or asset id"
                                        );
                                    } else {
                                        let lut_ref = if let ValueNode::Identifier(id) = &args[1] {
                                            env.get(id).unwrap_or(&args[1])
                                        } else {
                                            &args[1]
                                        };
                                        let lut_str =
                                            Self::value_to_string(lut_ref).unwrap_or_default();

                                        let lut_path = project
                                            .assets
                                            .get(&vidra_ir::asset::AssetId::new(lut_str.clone()))
                                            .map(|a| a.path.to_string_lossy().to_string())
                                            .unwrap_or(lut_str);

                                        let intensity = if args.len() > 2 {
                                            let iv = if let ValueNode::Identifier(id) = &args[2] {
                                                env.get(id).unwrap_or(&args[2])
                                            } else {
                                                &args[2]
                                            };
                                            Self::value_to_f64(iv).unwrap_or(1.0)
                                        } else {
                                            1.0
                                        };

                                        layer.effects.push(vidra_core::types::LayerEffect::Lut {
                                            path: lut_path,
                                            intensity,
                                        });
                                    }
                                }
                                _ => tracing::warn!("Unknown effect: {}", effect_type),
                            }
                        }
                    } else if name == "preset" && !args.is_empty() {
                        let preset_name = if let ValueNode::Identifier(id) = &args[0] {
                            env.get(id).unwrap_or(&args[0])
                        } else {
                            &args[0]
                        };

                        let delay = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };

                        if let Ok(preset_type) = Self::value_to_string(preset_name) {
                            match preset_type.as_str() {
                                "fadeInUp" => {
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::Opacity,
                                            0.0,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.5),
                                            vidra_core::types::Easing::EaseOut,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::PositionY,
                                            50.0,
                                            0.0,
                                            vidra_core::Duration::from_seconds(0.5),
                                            vidra_core::types::Easing::EaseOut,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                }
                                "bounceIn" => {
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::ScaleX,
                                            0.0,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.6),
                                            vidra_core::types::Easing::EaseOutBack,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::ScaleY,
                                            0.0,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.6),
                                            vidra_core::types::Easing::EaseOutBack,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::Opacity,
                                            0.0,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.3),
                                            vidra_core::types::Easing::Linear,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                }
                                "typewriter" => {
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::Opacity,
                                            0.0,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.2),
                                            vidra_core::types::Easing::Linear,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                }
                                "glitch" => {
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::PositionX,
                                            -10.0,
                                            0.0,
                                            vidra_core::Duration::from_seconds(0.2),
                                            vidra_core::types::Easing::EaseIn,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                    layer.animations.push(
                                        vidra_ir::animation::Animation::from_to(
                                            vidra_ir::animation::AnimatableProperty::ScaleX,
                                            1.5,
                                            1.0,
                                            vidra_core::Duration::from_seconds(0.2),
                                            vidra_core::types::Easing::Linear,
                                        )
                                        .with_delay(vidra_core::Duration::from_seconds(delay)),
                                    );
                                }
                                _ => tracing::warn!("Unknown preset: {}", preset_type),
                            }
                        }
                    } else if name == "mask" && !args.is_empty() {
                        let mask_layer_name = if let ValueNode::Identifier(id) = &args[0] {
                            env.get(id).unwrap_or(&args[0])
                        } else {
                            &args[0]
                        };
                        if let Ok(mask_str) = Self::value_to_string(mask_layer_name) {
                            layer.mask = Some(LayerId::new(mask_str));
                        }
                    } else if name == "center" && !args.is_empty() {
                        // center(horizontal), center(vertical), center(both) or center()
                        let axis_val =
                            Self::value_to_string(&args[0]).unwrap_or_else(|_| "both".to_string());
                        let axis = match axis_val.as_str() {
                            "horizontal" | "h" => vidra_ir::layout::CenterAxis::Horizontal,
                            "vertical" | "v" => vidra_ir::layout::CenterAxis::Vertical,
                            _ => vidra_ir::layout::CenterAxis::Both,
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Center(axis));
                    } else if name == "center" && args.is_empty() {
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Center(
                                vidra_ir::layout::CenterAxis::Both,
                            ));
                    } else if name == "pin" && !args.is_empty() {
                        // pin(top, 20), pin(left), pin(bottom, 40)
                        let edge_val =
                            Self::value_to_string(&args[0]).unwrap_or_else(|_| "top".to_string());
                        let margin = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        let edge = match edge_val.as_str() {
                            "top" => vidra_ir::layout::Edge::Top,
                            "bottom" => vidra_ir::layout::Edge::Bottom,
                            "left" => vidra_ir::layout::Edge::Left,
                            "right" => vidra_ir::layout::Edge::Right,
                            _ => vidra_ir::layout::Edge::Top,
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Pin { edge, margin });
                    } else if name == "below" && !args.is_empty() {
                        let anchor = Self::value_to_string(&args[0])?;
                        let spacing = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Below {
                                anchor_layer: anchor,
                                spacing,
                            });
                    } else if name == "above" && !args.is_empty() {
                        let anchor = Self::value_to_string(&args[0])?;
                        let spacing = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Above {
                                anchor_layer: anchor,
                                spacing,
                            });
                    } else if name == "rightOf" && !args.is_empty() {
                        let anchor = Self::value_to_string(&args[0])?;
                        let spacing = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::RightOf {
                                anchor_layer: anchor,
                                spacing,
                            });
                    } else if name == "leftOf" && !args.is_empty() {
                        let anchor = Self::value_to_string(&args[0])?;
                        let spacing = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::LeftOf {
                                anchor_layer: anchor,
                                spacing,
                            });
                    } else if name == "fill" && !args.is_empty() {
                        let axis_val =
                            Self::value_to_string(&args[0]).unwrap_or_else(|_| "both".to_string());
                        let padding = if args.len() > 1 {
                            Self::value_to_f64(&args[1]).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        let axis = match axis_val.as_str() {
                            "horizontal" | "h" => vidra_ir::layout::FillAxis::Horizontal,
                            "vertical" | "v" => vidra_ir::layout::FillAxis::Vertical,
                            _ => vidra_ir::layout::FillAxis::Both,
                        };
                        layer
                            .constraints
                            .push(vidra_ir::layout::LayoutConstraint::Fill { axis, padding });
                    } else if name == "scale" {
                        // scale(s) or scale(sx, sy)
                        if !args.is_empty() {
                            let ax = if let ValueNode::Identifier(id) = &args[0] {
                                env.get(id).unwrap_or(&args[0])
                            } else {
                                &args[0]
                            };
                            let sx = Self::value_to_f64(ax).unwrap_or(1.0);
                            let sy = if args.len() > 1 {
                                let ay = if let ValueNode::Identifier(id) = &args[1] {
                                    env.get(id).unwrap_or(&args[1])
                                } else {
                                    &args[1]
                                };
                                Self::value_to_f64(ay).unwrap_or(sx)
                            } else {
                                sx
                            };
                            layer.transform.scale.x = sx;
                            layer.transform.scale.y = sy;
                        }
                    } else if name == "rotation" {
                        if let Some(v) = args.get(0) {
                            layer.transform.rotation = Self::value_to_f64(v).unwrap_or(0.0);
                        }
                    } else if name == "opacity" {
                        if let Some(v) = args.get(0) {
                            layer.transform.opacity = Self::value_to_f64(v).unwrap_or(1.0);
                        }
                    } else if name == "anchor" {
                        if args.len() >= 2 {
                            layer.transform.anchor.x = Self::value_to_f64(&args[0]).unwrap_or(0.5);
                            layer.transform.anchor.y = Self::value_to_f64(&args[1]).unwrap_or(0.5);
                        }
                    } else if name == "translateZ" || name == "translate_z" {
                        if let Some(v) = args.get(0) {
                            layer.transform.translate_z = Self::value_to_f64(v).unwrap_or(0.0);
                        }
                    } else if name == "rotateX" || name == "rotate_x" {
                        if let Some(v) = args.get(0) {
                            layer.transform.rotate_x = Self::value_to_f64(v).unwrap_or(0.0);
                        }
                    } else if name == "rotateY" || name == "rotate_y" {
                        if let Some(v) = args.get(0) {
                            layer.transform.rotate_y = Self::value_to_f64(v).unwrap_or(0.0);
                        }
                    } else if name == "perspective" {
                        if let Some(v) = args.get(0) {
                            layer.transform.perspective = Self::value_to_f64(v).unwrap_or(0.0);
                        }
                    } else {
                        // Handle generic function calls — extensible for enter/exit/etc.
                        tracing::debug!("unhandled function call: {}", name);
                    }
                }
                PropertyNode::AnimationGroup {
                    animations,
                    span: _,
                } => {
                    // All start at 0 (or delay inside)
                    for ag_prop in animations {
                        if let PropertyNode::Animation { property, args, .. } = ag_prop {
                            let anims = Self::compile_animation(property, args, env)?;
                            layer.animations.extend(anims);
                        }
                    }
                }
                PropertyNode::AnimationSequence {
                    animations,
                    span: _,
                } => {
                    let mut current_time = 0.0;
                    for seq_prop in animations {
                        match seq_prop {
                            PropertyNode::Wait { duration, .. } => {
                                current_time += Self::value_to_f64(duration).unwrap_or(0.0);
                            }
                            PropertyNode::Animation { property, args, .. } => {
                                let anims = Self::compile_animation(property, args, env)?;
                                let mut max_dur = 0.0;
                                for mut anim in anims {
                                    let delay = anim.delay.as_seconds() + current_time;
                                    anim.delay = vidra_core::Duration::from_seconds(delay);
                                    let d = if let Some(last) = anim.keyframes.last() {
                                        last.time.as_seconds()
                                    } else {
                                        0.0
                                    };
                                    if d > max_dur {
                                        max_dur = d;
                                    }
                                    layer.animations.push(anim);
                                }
                                current_time += max_dur;
                            }
                            _ => {} // Group within Sequence not yet handled
                        }
                    }
                }
                PropertyNode::Wait { .. } => {
                    // no-op if outside of a sequence
                }
                PropertyNode::OnEvent { event, actions, .. } => {
                    let event_ty = match event.as_str() {
                        "click" => vidra_ir::layer::LayerEventType::Click,
                        other => {
                            tracing::warn!("unsupported @on event '{}'", other);
                            continue;
                        }
                    };

                    let compiled_actions = actions
                        .iter()
                        .map(|a| vidra_ir::layer::LayerAction::SetVar {
                            name: a.name.clone(),
                            expr: a.expr.clone(),
                        })
                        .collect::<Vec<_>>();

                    layer.events.push(vidra_ir::layer::LayerEventHandler {
                        event: event_ty,
                        actions: compiled_actions,
                    });
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
                    if arg.name == "variant" {
                        continue;
                    }
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value).clone(),
                        _ => arg.value.clone(),
                    };
                    comp_env.insert(arg.name.clone(), val);
                }
                // 3. instantiate template layers, passing compiled_slots
                for comp_layer_item in &comp_def.items {
                    let compiled = self.compile_layer_block_item(
                        comp_layer_item,
                        project,
                        &comp_env,
                        &compiled_slots,
                    )?;
                    for mut child_layer in compiled {
                        child_layer.id = vidra_ir::layer::LayerId::new(format!(
                            "{}_{}",
                            layer.id.0, child_layer.id.0
                        ));
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
                } else {
                    text
                };
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
                } else {
                    path
                };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Image,
                        resolved_path,
                    ));
                }

                Ok(LayerContent::Image { asset_id })
            }
            LayerContentNode::Spritesheet { path, args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else {
                    path
                };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Image,
                        resolved_path,
                    ));
                }

                let mut frame_width = 64.0;
                let mut frame_height = 64.0;
                let mut fps = 12.0;
                let mut start_frame = 0.0;
                let mut frame_count: Option<u32> = None;

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "frameWidth" | "frame_width" => frame_width = Self::value_to_f64(val)?,
                        "frameHeight" | "frame_height" => frame_height = Self::value_to_f64(val)?,
                        "fps" => fps = Self::value_to_f64(val)?,
                        "start" | "startFrame" | "start_frame" => {
                            start_frame = Self::value_to_f64(val)?
                        }
                        "frameCount" | "frame_count" => {
                            frame_count = Some(Self::value_to_f64(val)?.max(0.0) as u32)
                        }
                        _ => {}
                    }
                }

                Ok(LayerContent::Spritesheet {
                    asset_id,
                    frame_width: frame_width.max(1.0) as u32,
                    frame_height: frame_height.max(1.0) as u32,
                    fps,
                    start_frame: start_frame.max(0.0) as u32,
                    frame_count,
                })
            }
            LayerContentNode::Video { path, args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else {
                    path
                };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Video,
                        resolved_path,
                    ));
                }

                let mut trim_start = vidra_core::Duration::zero();
                let mut trim_end = None;

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "trim_start" => {
                            trim_start =
                                vidra_core::Duration::from_seconds(Self::value_to_f64(val)?)
                        }
                        "trim_end" => {
                            trim_end =
                                Some(vidra_core::Duration::from_seconds(Self::value_to_f64(val)?))
                        }
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
                } else {
                    path
                };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Audio,
                        resolved_path,
                    ));
                }

                let mut trim_start = vidra_core::Duration::zero();
                let mut trim_end = None;
                let mut volume = 1.0;
                let mut role: Option<String> = None;
                let mut duck: Option<f64> = None;

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "trim_start" => {
                            trim_start =
                                vidra_core::Duration::from_seconds(Self::value_to_f64(val)?)
                        }
                        "trim_end" => {
                            trim_end =
                                Some(vidra_core::Duration::from_seconds(Self::value_to_f64(val)?))
                        }
                        "volume" => volume = Self::value_to_f64(val)?,
                        "role" => role = Some(Self::value_to_string(val)?),
                        "duck" => duck = Some(Self::value_to_f64(val)?),
                        _ => {}
                    }
                }

                Ok(LayerContent::Audio {
                    asset_id,
                    trim_start,
                    trim_end,
                    volume,
                    role,
                    duck,
                })
            }
            LayerContentNode::Waveform { audio_source, args } => {
                let path_str =
                    Self::value_to_string(if let ValueNode::Identifier(id) = audio_source {
                        env.get(id).unwrap_or(audio_source)
                    } else {
                        audio_source
                    })?;
                let asset_id = AssetId::new(path_str.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Audio,
                        path_str,
                    ));
                }

                let mut width = 1024.0;
                let mut height = 256.0;
                let mut color = Color::WHITE;
                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "width" => width = Self::value_to_f64(val)?,
                        "height" => height = Self::value_to_f64(val)?,
                        "color" => color = Self::value_to_color(val)?,
                        _ => {}
                    }
                }

                Ok(LayerContent::Waveform {
                    asset_id,
                    width: width.max(1.0) as u32,
                    height: height.max(1.0) as u32,
                    color,
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
                    audio_asset_id: None,
                })
            }
            LayerContentNode::AutoCaption { audio_source, args } => {
                let path_str =
                    Self::value_to_string(if let ValueNode::Identifier(id) = audio_source {
                        env.get(id).unwrap_or(audio_source)
                    } else {
                        audio_source
                    })?;
                let asset_id = AssetId::new(path_str);

                // Ensure audio source is registered as an Audio asset so downstream tooling
                // (AI materializers, encoders) can resolve it consistently.
                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Audio,
                        asset_id.0.clone(),
                    ));
                }

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
                } else {
                    color
                };
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
                        let width = get_val("width")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(100.0))
                            .unwrap_or(100.0);
                        let height = get_val("height")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(100.0))
                            .unwrap_or(100.0);
                        let corner_radius = get_val("cornerRadius")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(0.0))
                            .unwrap_or(0.0);
                        vidra_core::types::ShapeType::Rect {
                            width,
                            height,
                            corner_radius,
                        }
                    }
                    "circle" => {
                        let radius = get_val("radius")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(50.0))
                            .unwrap_or(50.0);
                        vidra_core::types::ShapeType::Circle { radius }
                    }
                    "ellipse" => {
                        let rx = get_val("rx")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(50.0))
                            .unwrap_or(50.0);
                        let ry = get_val("ry")
                            .map(|v| Self::value_to_f64(&v).unwrap_or(50.0))
                            .unwrap_or(50.0);
                        vidra_core::types::ShapeType::Ellipse { rx, ry }
                    }
                    _ => {
                        return Err(VidraError::Compile(format!(
                            "unknown shape type: {}",
                            shape_type
                        )));
                    }
                };

                Ok(LayerContent::Shape {
                    shape,
                    fill: fill_color,
                    stroke: stroke_color,
                    stroke_width: stroke_w,
                })
            }
            LayerContentNode::Shader { path, args: _args } => {
                let path_val = if let ValueNode::Identifier(id) = path {
                    env.get(id).unwrap_or(path)
                } else {
                    path
                };
                let resolved_path = Self::value_to_string(path_val)?;
                let asset_id = AssetId::new(resolved_path.clone());

                if project.assets.get(&asset_id).is_none() {
                    project.assets.register(Asset::new(
                        asset_id.clone(),
                        AssetType::Shader,
                        resolved_path,
                    ));
                }

                Ok(LayerContent::Shader { asset_id })
            }
            LayerContentNode::Web { source, args } => {
                let source_val = if let ValueNode::Identifier(id) = source {
                    env.get(id).unwrap_or(source)
                } else {
                    source
                };
                let resolved_source = Self::value_to_string(source_val)?;

                let mut viewport_width = 1920;
                let mut viewport_height = 1080;
                let mut mode = vidra_ir::layer::WebCaptureMode::FrameAccurate;
                let mut wait_for = None;
                let mut variables = std::collections::HashMap::new();

                for arg in args {
                    let val = match &arg.value {
                        ValueNode::Identifier(id) => env.get(id).unwrap_or(&arg.value),
                        _ => &arg.value,
                    };
                    match arg.name.as_str() {
                        "viewport" => {
                            if let Ok(vp_str) = Self::value_to_string(val) {
                                let parts: Vec<&str> = vp_str.split('x').collect();
                                if parts.len() == 2 {
                                    viewport_width = parts[0].parse().unwrap_or(1920);
                                    viewport_height = parts[1].parse().unwrap_or(1080);
                                }
                            }
                        }
                        "mode" => {
                            if let Ok(m) = Self::value_to_string(val) {
                                if m == "realtime" {
                                    mode = vidra_ir::layer::WebCaptureMode::Realtime;
                                }
                            }
                        }
                        "wait_for" => wait_for = Some(Self::value_to_string(val)?),
                        _ => {}
                    }
                }

                Ok(LayerContent::Web {
                    source: resolved_source,
                    viewport_width,
                    viewport_height,
                    mode,
                    wait_for,
                    variables,
                })
            }
            LayerContentNode::Component { .. }
            | LayerContentNode::Slot
            | LayerContentNode::Empty => Ok(LayerContent::Empty),
        }
    }

    fn compile_animation(
        property: &str,
        args: &[NamedArg],
        env: &HashMap<String, ValueNode>,
    ) -> Result<Vec<Animation>, VidraError> {
        let animatable = match property {
            "opacity" => Some(AnimatableProperty::Opacity),
            "position.x" | "positionX" | "x" => Some(AnimatableProperty::PositionX),
            "position.y" | "positionY" | "y" => Some(AnimatableProperty::PositionY),
            "scale.x" | "scaleX" => Some(AnimatableProperty::ScaleX),
            "scale.y" | "scaleY" => Some(AnimatableProperty::ScaleY),
            "scale" => Some(AnimatableProperty::ScaleX), // convenience
            "rotation" => Some(AnimatableProperty::Rotation),
            "translateZ" | "translate_z" => Some(AnimatableProperty::TranslateZ),
            "rotateX" | "rotate_x" => Some(AnimatableProperty::RotateX),
            "rotateY" | "rotate_y" => Some(AnimatableProperty::RotateY),
            "perspective" => Some(AnimatableProperty::Perspective),
            "position" => None,                          // Special case for paths
            "color" => Some(AnimatableProperty::ColorR), // Pseudo-property, handled specially
            "fontSize" => Some(AnimatableProperty::FontSize),
            "cornerRadius" => Some(AnimatableProperty::CornerRadius),
            "strokeWidth" => Some(AnimatableProperty::StrokeWidth),
            "crop.top" | "cropTop" => Some(AnimatableProperty::CropTop),
            "crop.right" | "cropRight" => Some(AnimatableProperty::CropRight),
            "crop.bottom" | "cropBottom" => Some(AnimatableProperty::CropBottom),
            "crop.left" | "cropLeft" => Some(AnimatableProperty::CropLeft),
            "volume" => Some(AnimatableProperty::Volume),
            "blur" | "blurRadius" => Some(AnimatableProperty::BlurRadius),
            "brightness" | "brightnessLevel" => Some(AnimatableProperty::BrightnessLevel),
            _ => {
                return Err(VidraError::Compile(format!(
                    "unknown animatable property: {}",
                    property
                )));
            }
        };

        let mut from_val = 0.0;
        let mut to_val = 1.0;
        let mut from_color = None;
        let mut to_color = None;
        let mut duration = 1.0;
        let mut easing = vidra_core::types::Easing::Linear;
        let mut delay = 0.0;

        let mut stiffness = None;
        let mut damping = None;
        let mut velocity = None;
        let mut expr = None;
        let mut path = None;
        let mut audio_source = None;

        for arg in args {
            let val = env.get(&arg.name).unwrap_or(&arg.value);
            // Re-resolve if the value itself is an identifier (e.g. `duration: compDelay`)
            let resolved_val = if let ValueNode::Identifier(id) = val {
                env.get(id).unwrap_or(val)
            } else {
                val
            };

            match arg.name.as_str() {
                "from" => {
                    if property == "color" {
                        from_color = Some(Self::value_to_color(resolved_val)?);
                    } else {
                        from_val = Self::value_to_f64(resolved_val)?;
                    }
                }
                "to" => {
                    if property == "color" {
                        to_color = Some(Self::value_to_color(resolved_val)?);
                    } else {
                        to_val = Self::value_to_f64(resolved_val)?;
                    }
                }
                "duration" => duration = Self::value_to_duration(resolved_val)?,
                "delay" => delay = Self::value_to_duration(resolved_val)?,
                "ease" | "easing" => {
                    easing = Self::value_to_easing(resolved_val)?;
                }
                "stiffness" => stiffness = Some(Self::value_to_f64(resolved_val)?),
                "damping" => damping = Some(Self::value_to_f64(resolved_val)?),
                "velocity" | "initialVelocity" => {
                    velocity = Some(Self::value_to_f64(resolved_val)?)
                }
                "expr" | "expression" => expr = Some(Self::value_to_string(resolved_val)?),
                "audio" => audio_source = Some(Self::value_to_string(resolved_val)?),
                "path" => path = Some(Self::value_to_string(resolved_val)?),
                _ => {}
            }
        }

        let mut anims = Vec::new();

        if property == "color" {
            let fc = from_color.unwrap_or_else(|| Color::WHITE);
            let tc = to_color.unwrap_or_else(|| Color::WHITE);
            let dur = vidra_core::Duration::from_seconds(duration);
            let del = vidra_core::Duration::from_seconds(delay);

            let mut ar = Animation::from_to(
                AnimatableProperty::ColorR,
                fc.r as f64,
                tc.r as f64,
                dur,
                easing.clone(),
            );
            let mut ag = Animation::from_to(
                AnimatableProperty::ColorG,
                fc.g as f64,
                tc.g as f64,
                dur,
                easing.clone(),
            );
            let mut ab = Animation::from_to(
                AnimatableProperty::ColorB,
                fc.b as f64,
                tc.b as f64,
                dur,
                easing.clone(),
            );
            let mut aa = Animation::from_to(
                AnimatableProperty::ColorA,
                fc.a as f64,
                tc.a as f64,
                dur,
                easing.clone(),
            );

            if delay > 0.0 {
                ar = ar.with_delay(del);
                ag = ag.with_delay(del);
                ab = ab.with_delay(del);
                aa = aa.with_delay(del);
            }
            anims.push(ar);
            anims.push(ag);
            anims.push(ab);
            anims.push(aa);
        } else if let Some(p) = path {
            let (mut ax, mut ay) = crate::advanced_anim::compile_path_animations(&p, duration);
            if delay > 0.0 {
                ax = ax.with_delay(vidra_core::Duration::from_seconds(delay));
                ay = ay.with_delay(vidra_core::Duration::from_seconds(delay));
            }
            anims.push(ax);
            anims.push(ay);
        } else if let Some(e) = expr {
            let (rewritten_interactive, uses_mouse) = rewrite_interactive_state_expr(&e);
            if uses_mouse {
                if rewritten_interactive.contains("audio.amplitude") {
                    return Err(VidraError::Compile(
                        "interactive expressions with audio.amplitude are not supported yet"
                            .to_string(),
                    ));
                }

                let mut a = Animation::new(animatable.unwrap());
                a.expr = Some(rewritten_interactive);
                a.expr_duration = Some(vidra_core::Duration::from_seconds(duration));
                if delay > 0.0 {
                    a = a.with_delay(vidra_core::Duration::from_seconds(delay));
                }
                anims.push(a);
            } else {
                let (rewritten, amp_samples) = Self::prepare_audio_expression(
                    &rewritten_interactive,
                    audio_source.as_deref(),
                    duration,
                )?;
                let mut a = crate::advanced_anim::compile_expression(
                    animatable.unwrap(),
                    &rewritten,
                    duration,
                    amp_samples.as_deref(),
                );
                if delay > 0.0 {
                    a = a.with_delay(vidra_core::Duration::from_seconds(delay));
                }
                anims.push(a);
            }
        } else if let Some(s) = stiffness {
            let d = damping.unwrap_or(10.0);
            let v = velocity.unwrap_or(0.0);
            let mut a = crate::advanced_anim::compile_spring(
                animatable.unwrap(),
                from_val,
                to_val,
                s,
                d,
                v,
            );
            if delay > 0.0 {
                a = a.with_delay(vidra_core::Duration::from_seconds(delay));
            }
            anims.push(a);
        } else {
            let mut anim = Animation::from_to(
                animatable.unwrap(),
                from_val,
                to_val,
                vidra_core::Duration::from_seconds(duration),
                easing,
            );
            if delay > 0.0 {
                anim = anim.with_delay(vidra_core::Duration::from_seconds(delay));
            }
            anims.push(anim);
        }

        Ok(anims)
    }

    fn prepare_audio_expression(
        expr: &str,
        audio_source: Option<&str>,
        duration: f64,
    ) -> Result<(String, Option<Vec<f64>>), VidraError> {
        // Only do extra work if the expression references audio amplitude.
        if !expr.contains("audio.amplitude") {
            return Ok((expr.to_string(), None));
        }

        let rewritten = rewrite_audio_amplitude_expr(expr);

        // Determine audio path: prefer explicit `audio:` arg; else try to extract audio.amplitude("path").
        let audio_path = audio_source
            .map(|s| s.to_string())
            .or_else(|| extract_audio_amplitude_path(expr));

        let Some(audio_path) = audio_path else {
            // No audio file known; keep audio_amp=0.0.
            return Ok((rewritten, None));
        };

        // Best-effort: if ffmpeg isn't available, fall back to 0.
        if !is_ffmpeg_available() {
            return Ok((rewritten, None));
        }

        let samples = compute_audio_rms_envelope(&audio_path, duration).map_err(|e| {
            VidraError::Compile(format!("failed to compute audio amplitude: {}", e))
        })?;
        Ok((rewritten, Some(samples)))
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
                Color::from_hex(hex).map_err(|e| {
                    VidraError::Compile(format!("invalid brand color fallback: {}", e))
                })
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

fn rewrite_interactive_state_expr(expr: &str) -> (String, bool) {
    // evalexpr variable names cannot contain '@' or '.', so we rewrite.
    let rewritten = expr
        .replace("@mouse.x", "mouse_x")
        .replace("@mouse.y", "mouse_y")
        .replace("mouse.x", "mouse_x")
        .replace("mouse.y", "mouse_y")
        .replace("@t", "t")
        .replace("@p", "p")
        .replace("@T", "T");

    let uses_mouse = rewritten.contains("mouse_x") || rewritten.contains("mouse_y");
    (rewritten, uses_mouse)
}

fn rewrite_audio_amplitude_expr(expr: &str) -> String {
    // Replace audio.amplitude("...") or audio.amplitude(...) with audio_amp.
    // This is a simple rewrite so evalexpr can use a normal variable name.
    let mut out = String::new();
    let mut chars = expr.chars().peekable();
    while let Some(c) = chars.next() {
        if c == 'a' {
            // Quick check for substring.
            let mut lookahead = String::new();
            lookahead.push(c);
            for _ in 0..("udio.amplitude".len()) {
                if let Some(n) = chars.peek().copied() {
                    lookahead.push(n);
                    chars.next();
                }
            }
            if lookahead == "audio.amplitude" {
                // If next is '(', consume until matching ')'.
                if chars.peek() == Some(&'(') {
                    let mut depth = 0i32;
                    while let Some(n) = chars.next() {
                        if n == '(' {
                            depth += 1;
                        } else if n == ')' {
                            depth -= 1;
                            if depth <= 0 {
                                break;
                            }
                        }
                    }
                }
                out.push_str("audio_amp");
                continue;
            }
            out.push_str(&lookahead);
            continue;
        }
        out.push(c);
    }
    out
}

fn extract_audio_amplitude_path(expr: &str) -> Option<String> {
    // Look for audio.amplitude("path") and return path.
    let needle = "audio.amplitude(\"";
    let start = expr.find(needle)? + needle.len();
    let rest = &expr[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn compute_audio_rms_envelope(
    audio_path: &str,
    duration: f64,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // Decode to mono s16le at low sample rate to keep memory reasonable.
    let sample_rate: f64 = 8000.0;
    let fps: f64 = 60.0;
    let dt: f64 = 1.0 / fps;

    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(audio_path)
        .args(["-vn", "-ac", "1", "-ar", "8000", "-f", "s16le", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to open ffmpeg stdout")?;
    let mut pcm = Vec::new();
    stdout.read_to_end(&mut pcm)?;

    let status = child.wait()?;
    if !status.success() {
        return Err("ffmpeg decode failed".into());
    }

    let sample_count = pcm.len() / 2;
    let mut samples = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let lo = pcm[i * 2] as u16;
        let hi = (pcm[i * 2 + 1] as u16) << 8;
        let v = (lo | hi) as i16;
        samples.push(v as f64 / 32768.0);
    }

    let window = (sample_rate / fps).round().max(1.0) as usize;
    let frame_count = (duration / dt).ceil() as usize + 1;
    let mut out = Vec::with_capacity(frame_count);

    for frame_idx in 0..frame_count {
        let start = frame_idx * window;
        if start >= samples.len() {
            out.push(0.0);
            continue;
        }
        let end = (start + window).min(samples.len());
        let mut sum_sq = 0.0;
        for &s in &samples[start..end] {
            sum_sq += s * s;
        }
        let mean = sum_sq / (end - start).max(1) as f64;
        let rms = mean.sqrt().clamp(0.0, 1.0);
        out.push(rms);
    }

    Ok(out)
}

#[cfg(test)]
mod audio_expr_tests {
    use super::*;

    #[test]
    fn rewrite_replaces_audio_amplitude_calls() {
        let src = "1 + audio.amplitude(\"assets/a.mp3\") * 2";
        let out = rewrite_audio_amplitude_expr(src);
        assert_eq!(out, "1 + audio_amp * 2");
    }

    #[test]
    fn extract_path_from_audio_amplitude() {
        let src = "audio.amplitude(\"assets/a.mp3\")";
        assert_eq!(
            extract_audio_amplitude_path(src).as_deref(),
            Some("assets/a.mp3")
        );
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
    fn test_compile_web_layer() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("s", 3s) {
                    layer("web_view") {
                        web("http://localhost:3000", viewport: "1280x720", mode: "realtime", wait_for: ".loaded")
                        position(100, 200)
                    }
                }
            }
        "#,
        );
        let layer = &project.scenes[0].layers[0];
        match &layer.content {
            LayerContent::Web {
                source,
                viewport_width,
                viewport_height,
                mode,
                wait_for,
                ..
            } => {
                assert_eq!(source, "http://localhost:3000");
                assert_eq!(*viewport_width, 1280);
                assert_eq!(*viewport_height, 720);
                assert_eq!(*mode, vidra_ir::layer::WebCaptureMode::Realtime);
                assert_eq!(wait_for.as_deref(), Some(".loaded"));
            }
            _ => panic!("expected web layer"),
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
        assert!(
            matches!(btn_bg.content, LayerContent::Solid { color: c } if c.to_rgba8() == [0, 0, 255, 255])
        );
        assert_eq!(btn_bg.animations[0].duration().as_seconds(), 1.0); // Default t_duration

        let btn_label = &btn1_wrapper.children[1];
        assert!(
            matches!(&btn_label.content, LayerContent::Text { text, .. } if text == "Click Me")
        );
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
        assert!(
            matches!(&slotted.content, LayerContent::Text { text, .. } if text == "Inside Slot")
        );
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

    #[test]
    fn test_compile_advanced_animations() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("main", 5s) {
                    layer("spring_layer") {
                        animation(x, from: 0.0, to: 100.0, stiffness: 50.0, damping: 5.0)
                    }
                    layer("expr_layer") {
                        animation(y, expr: "t * 50.0", duration: 2.0)
                    }
                    layer("mouse_expr_layer") {
                        animation(x, expr: "@mouse.x * 2.0", duration: 2.0)
                    }
                    layer("path_layer") {
                        animation(position, path: "M0 0 L100 100", duration: 2.0)
                    }
                }
            }
        "#,
        );
        let s = &project.scenes[0];

        let l1 = &s.layers[0];
        assert_eq!(l1.animations.len(), 1);
        assert!(l1.animations[0].keyframes.len() > 2);

        let l2 = &s.layers[1];
        assert_eq!(l2.animations.len(), 1);
        assert!(l2.animations[0].keyframes.len() > 2);

        let l_mouse = &s.layers[2];
        assert_eq!(l_mouse.animations.len(), 1);
        assert!(l_mouse.animations[0].keyframes.is_empty());
        assert!(l_mouse.animations[0].expr.is_some());
        assert!(l_mouse.animations[0].expr_duration.is_some());

        let l3 = &s.layers[3];
        assert_eq!(l3.animations.len(), 2);
    }

    #[test]
    fn test_compile_extended_properties() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("main", 5s) {
                    layer("text_layer") {
                        text("Hello", fontSize: 50.0)
                        animation(fontSize, from: 50.0, to: 100.0, duration: 1.0)
                        animation(color, from: #000000, to: #FFFFFF, duration: 1.0)
                    }
                    layer("shape_layer") {
                        shape("rect", width: 100, height: 100, cornerRadius: 0.0)
                        animation(cornerRadius, from: 0.0, to: 50.0, duration: 1.0)
                    }
                }
            }
        "#,
        );
        let s = &project.scenes[0];
        let l1 = &s.layers[0];
        // text layer has fontSize animation (1) and color animations (4) = 5 total
        assert_eq!(l1.animations.len(), 5);
        let is_color_anim = l1
            .animations
            .iter()
            .any(|a| matches!(a.property, AnimatableProperty::ColorR));
        assert!(is_color_anim);

        let l2 = &s.layers[1];
        assert_eq!(l2.animations.len(), 1);
        assert!(matches!(
            l2.animations[0].property,
            AnimatableProperty::CornerRadius
        ));
    }

    #[test]
    fn test_compile_shader() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("main", 5s) {
                    layer("shader_layer") {
                        shader("fractal.wgsl")
                    }
                }
            }
        "#,
        );
        let l1 = &project.scenes[0].layers[0];
        assert!(matches!(l1.content, LayerContent::Shader { .. }));
        assert_eq!(project.assets.count(), 1);
        let asset = project.assets.all().next().unwrap();
        assert_eq!(asset.asset_type, AssetType::Shader);
    }

    #[test]
    fn test_compile_constraints() {
        let project = compile(
            r#"
            project(1920, 1080, 30) {
                scene("main", 5s) {
                    layer("title") {
                        text("Hello World")
                        center("horizontal")
                        pin("top", 100)
                    }
                    layer("subtitle") {
                        text("Sub")
                        center("horizontal")
                        below("title", 20)
                    }
                    layer("cta") {
                        solid(#FF0000)
                        pin("bottom", 40)
                        pin("right", 50)
                        fill("horizontal", 60)
                    }
                }
            }
        "#,
        );
        let s = &project.scenes[0];

        // Title: center(horizontal) + pin(top, 100)
        let title = &s.layers[0];
        assert_eq!(title.constraints.len(), 2);
        assert!(matches!(
            title.constraints[0],
            vidra_ir::layout::LayoutConstraint::Center(vidra_ir::layout::CenterAxis::Horizontal)
        ));
        assert!(
            matches!(title.constraints[1], vidra_ir::layout::LayoutConstraint::Pin { edge: vidra_ir::layout::Edge::Top, margin } if (margin - 100.0).abs() < 0.01)
        );

        // Subtitle: center(horizontal) + below("title", 20)
        let subtitle = &s.layers[1];
        assert_eq!(subtitle.constraints.len(), 2);
        assert!(
            matches!(&subtitle.constraints[1], vidra_ir::layout::LayoutConstraint::Below { anchor_layer, spacing } if anchor_layer == "title" && (*spacing - 20.0).abs() < 0.01)
        );

        // CTA: pin(bottom) + pin(right) + fill(horizontal)
        let cta = &s.layers[2];
        assert_eq!(cta.constraints.len(), 3);

        // Run the solver against two viewports to verify responsiveness
        let solver_input: Vec<_> = s
            .layers
            .iter()
            .map(|l| (l.id.0.clone(), 200.0, 50.0, l.constraints.clone()))
            .collect();

        let r_16_9 = vidra_ir::layout::LayoutSolver::solve(1920.0, 1080.0, &solver_input);
        let r_9_16 = vidra_ir::layout::LayoutSolver::solve(1080.0, 1920.0, &solver_input);

        // Title should be centered horizontally in both
        assert!((r_16_9[0].1.x - (1920.0 - 200.0) / 2.0).abs() < 0.01);
        assert!((r_9_16[0].1.x - (1080.0 - 200.0) / 2.0).abs() < 0.01);

        // Title pinned at top=100 in both
        assert!((r_16_9[0].1.y - 100.0).abs() < 0.01);
        assert!((r_9_16[0].1.y - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_compile_on_click_handler() {
        let project = compile(
            r#"
            project(640, 360, 30) {
                scene("main", 1s) {
                    layer("btn") {
                        solid(#ffffff)
                        position(10, 20)
                        @on click {
                            set count = count + 1
                        }
                    }
                }
            }
        "#,
        );

        let layer = &project.scenes[0].layers[0];
        assert_eq!(layer.id.0, "btn");
        assert_eq!(layer.events.len(), 1);
        assert!(matches!(
            layer.events[0].event,
            vidra_ir::layer::LayerEventType::Click
        ));
        assert_eq!(layer.events[0].actions.len(), 1);
        match &layer.events[0].actions[0] {
            vidra_ir::layer::LayerAction::SetVar { name, expr } => {
                assert_eq!(name, "count");
                assert!(expr.contains("count"));
            }
        }
    }

    #[test]
    fn test_compile_spritesheet_layer() {
        let project = compile(
            r#"
            project(640, 360, 30) {
                scene("main", 1s) {
                    layer("fx") {
                        spritesheet("assets/sheet.png", frameWidth: 32, frameHeight: 16, fps: 10, start: 2, frameCount: 8)
                    }
                }
            }
        "#,
        );

        let layer = &project.scenes[0].layers[0];
        match &layer.content {
            vidra_ir::layer::LayerContent::Spritesheet {
                asset_id,
                frame_width,
                frame_height,
                fps,
                start_frame,
                frame_count,
            } => {
                assert_eq!(asset_id.0, "assets/sheet.png");
                assert_eq!(*frame_width, 32);
                assert_eq!(*frame_height, 16);
                assert!((*fps - 10.0).abs() < 1e-6);
                assert_eq!(*start_frame, 2);
                assert_eq!(*frame_count, Some(8));
            }
            other => panic!("expected Spritesheet content, got {:?}", other),
        }
    }
}
