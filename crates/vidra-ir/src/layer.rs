use serde::{Deserialize, Serialize};

use crate::animation::Animation;
use crate::asset::AssetId;
use vidra_core::types::ShapeType;
use vidra_core::{BlendMode, Color, Transform2D};

/// Unique identifier for a layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub String);

impl LayerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The content of a layer — what it renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerContent {
    /// A text layer.
    Text {
        text: String,
        font_family: String,
        font_size: f64,
        color: Color,
    },
    /// An image layer referencing an asset.
    Image { asset_id: AssetId },
    /// A video clip layer referencing an asset.
    Video {
        asset_id: AssetId,
        /// Trim start offset within the source video.
        trim_start: vidra_core::Duration,
        /// Trim end offset within the source video.
        trim_end: Option<vidra_core::Duration>,
    },
    /// An audio clip referencing an asset.
    Audio {
        asset_id: AssetId,
        trim_start: vidra_core::Duration,
        trim_end: Option<vidra_core::Duration>,
        volume: f64,
    },
    /// Text to Speech AI node
    TTS {
        text: String,
        voice: String,
        volume: f64,
    },
    /// Auto Caption AI generation
    AutoCaption {
        asset_id: AssetId,
        font_family: String,
        font_size: f64,
        color: Color,
    },
    /// A geometric shape.
    Shape {
        shape: ShapeType,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: f64,
    },
    /// A solid color fill.
    Solid { color: Color },
    /// An empty content block (useful for grouping layers into components).
    Empty,
}

/// A layer in a scene — a visual element with transform, animations, and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Unique layer identifier.
    pub id: LayerId,
    /// The visual content this layer renders.
    pub content: LayerContent,
    /// 2D transform (position, scale, rotation, opacity, anchor).
    pub transform: Transform2D,
    /// Blend mode for compositing.
    pub blend_mode: BlendMode,
    /// Animations applied to this layer.
    pub animations: Vec<Animation>,
    /// Visual effects applied to this layer.
    pub effects: Vec<vidra_core::types::LayerEffect>,
    /// Whether the layer is visible.
    pub visible: bool,
    /// Child layers (for nesting / component hierarchy).
    pub children: Vec<Layer>,
    /// Optional mask layer (alpha channel is used to mask this layer).
    pub mask: Option<LayerId>,
}

impl Layer {
    /// Create a new layer with the given content and default transform.
    pub fn new(id: LayerId, content: LayerContent) -> Self {
        Self {
            id,
            content,
            transform: Transform2D::identity(),
            blend_mode: BlendMode::Normal,
            animations: Vec::new(),
            effects: Vec::new(),
            visible: true,
            children: Vec::new(),
            mask: None,
        }
    }

    /// Builder: set position.
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.transform.position = vidra_core::Point2D::new(x, y);
        self
    }

    /// Builder: set scale.
    pub fn with_scale(mut self, sx: f64, sy: f64) -> Self {
        self.transform.scale = vidra_core::Point2D::new(sx, sy);
        self
    }

    /// Builder: set opacity.
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.transform.opacity = opacity;
        self
    }

    /// Builder: add an animation.
    pub fn with_animation(mut self, animation: Animation) -> Self {
        self.animations.push(animation);
        self
    }

    /// Add a child layer.
    pub fn add_child(&mut self, child: Layer) {
        self.children.push(child);
    }

    /// Get the layer type description.
    pub fn layer_type(&self) -> vidra_core::LayerType {
        match &self.content {
            LayerContent::Text { .. } => vidra_core::LayerType::Text,
            LayerContent::Image { .. } => vidra_core::LayerType::Image,
            LayerContent::Video { .. } => vidra_core::LayerType::Video,
            LayerContent::Audio { .. } => vidra_core::LayerType::Audio,
            LayerContent::Shape { .. } => vidra_core::LayerType::Shape,
            LayerContent::Solid { .. } => vidra_core::LayerType::Solid,
            LayerContent::TTS { .. } => vidra_core::LayerType::TTS,
            LayerContent::AutoCaption { .. } => vidra_core::LayerType::AutoCaption,
            LayerContent::Empty => vidra_core::LayerType::Component,
        }
    }
    /// Builder: add an effect.
    pub fn with_effect(mut self, effect: vidra_core::types::LayerEffect) -> Self {
        self.effects.push(effect);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new(
            LayerId::new("title"),
            LayerContent::Text {
                text: "Hello".into(),
                font_family: "Inter".into(),
                font_size: 48.0,
                color: Color::WHITE,
            },
        );
        assert_eq!(layer.id.0, "title");
        assert_eq!(layer.layer_type(), vidra_core::LayerType::Text);
        assert!(layer.visible);
        assert!(layer.animations.is_empty());
    }

    #[test]
    fn test_layer_builders() {
        let layer = Layer::new(
            LayerId::new("bg"),
            LayerContent::Solid { color: Color::BLUE },
        )
        .with_position(100.0, 200.0)
        .with_opacity(0.8);

        assert!((layer.transform.position.x - 100.0).abs() < 0.001);
        assert!((layer.transform.position.y - 200.0).abs() < 0.001);
        assert!((layer.transform.opacity - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_layer_children() {
        let mut parent = Layer::new(
            LayerId::new("group"),
            LayerContent::Solid {
                color: Color::TRANSPARENT,
            },
        );
        let child = Layer::new(
            LayerId::new("child"),
            LayerContent::Solid { color: Color::RED },
        );
        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
    }
}
