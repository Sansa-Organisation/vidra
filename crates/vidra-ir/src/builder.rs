use crate::project::{Project, ProjectSettings};
use crate::scene::{Scene, SceneId};
use crate::layer::{Layer, LayerId, LayerContent};
use crate::asset::{AssetRegistry, Asset, AssetId, AssetType};
use crate::animation::{Animation, Keyframe, AnimatableProperty};

use vidra_core::types::Easing;
use vidra_core::{Color, Duration, BlendMode, Transform2D, Point2D};


/// A builder for constructing a Vidra IR Project programmatically.
/// Useful for SDKs, programmatic generation, and unit testing.
pub struct ProjectBuilder {
    project: Project,
}

impl ProjectBuilder {
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            project: Project {
                id: "generated_id".to_string(),
                settings: ProjectSettings { width, height, fps, background: Color::BLACK },
                scenes: Vec::new(),
                assets: AssetRegistry::new(),
            },
        }
    }

    /// Add an asset to the global project registry.
    pub fn add_asset(&mut self, asset_type: AssetType, id: impl Into<String>, path: std::path::PathBuf) -> &mut Self {
        let id_str = id.into();
        self.project.assets.register(Asset {
            name: Some(id_str.clone()),
            asset_type,
            id: AssetId(id_str),
            path,
        });
        self
    }

    /// Add a scene to the project.
    pub fn add_scene(&mut self, scene: Scene) -> &mut Self {
        self.project.scenes.push(scene);
        self
    }

    /// Build and return the project.
    pub fn build(self) -> Project {
        self.project
    }
}

/// A builder for constructing a Scene within a Project.
pub struct SceneBuilder {
    scene: Scene,
}

impl SceneBuilder {
    pub fn new(id: impl Into<String>, duration: f64) -> Self {
        Self {
            scene: Scene {
                id: SceneId(id.into()),
                duration: Duration::from_seconds(duration),
                layers: Vec::new(),
                transition: None,
            },
        }
    }

    /// Add a layer to the scene. First added is rendered first (back).
    pub fn add_layer(&mut self, layer: Layer) -> &mut Self {
        self.scene.layers.push(layer);
        self
    }

    /// Build and return the scene.
    pub fn build(self) -> Scene {
        self.scene
    }
}

/// A builder for constructing a Layer.
pub struct LayerBuilder {
    layer: Layer,
}

impl LayerBuilder {
    pub fn new(id: impl Into<String>, content: LayerContent) -> Self {
        Self {
            layer: Layer {
                id: LayerId(id.into()),
                content,
                transform: Transform2D::identity(),
                blend_mode: BlendMode::Normal,
                animations: Vec::new(),
                effects: Vec::new(),
                visible: true,
                children: Vec::new(),
                mask: None,
            },
        }
    }

    pub fn position(&mut self, x: f64, y: f64) -> &mut Self {
        self.layer.transform.position = Point2D::new(x, y);
        self
    }

    pub fn scale(&mut self, filter: f64) -> &mut Self {
        self.layer.transform.scale = Point2D::new(filter, filter);
        self
    }

    pub fn scale_xy(&mut self, x: f64, y: f64) -> &mut Self {
        self.layer.transform.scale = Point2D::new(x, y);
        self
    }

    pub fn rotation(&mut self, angle: f64) -> &mut Self {
        self.layer.transform.rotation = angle;
        self
    }

    pub fn opacity(&mut self, val: f64) -> &mut Self {
        self.layer.transform.opacity = val;
        self
    }

    /// Add a child layer.
    pub fn add_child(&mut self, child: Layer) -> &mut Self {
        self.layer.children.push(child);
        self
    }

    /// Add an animation block.
    pub fn add_animation(&mut self, anim: Animation) -> &mut Self {
        self.layer.animations.push(anim);
        self
    }

    /// Build and return the layer.
    pub fn build(self) -> Layer {
        self.layer
    }
}

/// Helper builder for Animation blocks to easily attach keyframes.
pub struct AnimationBuilder {
    animation: Animation,
}

impl AnimationBuilder {
    pub fn new(property: AnimatableProperty) -> Self {
        Self {
            animation: Animation { 
                property,
                keyframes: Vec::new(),
                delay: Duration::zero(),
            },
        }
    }

    pub fn add_keyframe(&mut self, time: f64, value: f64, easing: Easing) -> &mut Self {
        self.animation.keyframes.push(Keyframe { time: Duration::from_seconds(time), value, easing });
        self
    }

    pub fn build(self) -> Animation {
        self.animation
    }
}
