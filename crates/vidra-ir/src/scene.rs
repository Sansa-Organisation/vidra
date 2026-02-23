use serde::{Deserialize, Serialize};

use crate::layer::Layer;

/// Unique identifier for a scene.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub String);

impl SceneId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for SceneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A scene in the video — a segment of time containing layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Unique scene identifier.
    pub id: SceneId,
    /// Duration of this scene.
    pub duration: vidra_core::Duration,
    /// Ordered list of layers (bottom to top for compositing).
    pub layers: Vec<Layer>,
    /// Optional transition to effect when entering this scene from the previous one.
    pub transition: Option<crate::transition::Transition>,
}

impl Scene {
    /// Create a new empty scene.
    pub fn new(id: SceneId, duration: vidra_core::Duration) -> Self {
        Self {
            id,
            duration,
            layers: Vec::new(),
            transition: None,
        }
    }

    /// Add a layer to the scene.
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Get a layer by its ID.
    pub fn get_layer(&self, id: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id.0 == id)
    }

    /// Get a mutable reference to a layer by its ID.
    pub fn get_layer_mut(&mut self, id: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id.0 == id)
    }

    /// Number of frames in this scene at the given fps.
    pub fn frame_count(&self, fps: f64) -> u64 {
        self.duration.frame_count(fps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::{Layer, LayerContent, LayerId};
    use vidra_core::Color;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new(
            SceneId::new("test"),
            vidra_core::Duration::from_seconds(5.0),
        );
        assert_eq!(scene.id.0, "test");
        assert!(scene.layers.is_empty());
        assert_eq!(scene.frame_count(30.0), 150);
    }

    #[test]
    fn test_scene_add_and_get_layer() {
        let mut scene = Scene::new(
            SceneId::new("test"),
            vidra_core::Duration::from_seconds(5.0),
        );
        let layer = Layer::new(
            LayerId::new("bg"),
            LayerContent::Solid { color: Color::RED },
        );
        scene.add_layer(layer);
        assert_eq!(scene.layers.len(), 1);
        assert!(scene.get_layer("bg").is_some());
        assert!(scene.get_layer("nonexistent").is_none());
    }
}
