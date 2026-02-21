use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::asset::AssetRegistry;
use crate::scene::Scene;

/// Top-level project — the root of the Vidra IR tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier.
    pub id: String,
    /// Project settings (resolution, fps, etc.).
    pub settings: ProjectSettings,
    /// Registered assets (images, fonts, audio, video clips).
    pub assets: AssetRegistry,
    /// Ordered list of scenes in the project.
    pub scenes: Vec<Scene>,
}

impl Project {
    /// Create a new project with the given settings.
    pub fn new(settings: ProjectSettings) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            settings,
            assets: AssetRegistry::new(),
            scenes: Vec::new(),
        }
    }

    /// Total duration of the project (sum of all scene durations).
    pub fn total_duration(&self) -> vidra_core::Duration {
        self.scenes
            .iter()
            .fold(vidra_core::Duration::zero(), |acc, s| acc + s.duration)
    }

    /// Total number of frames in the project.
    pub fn total_frames(&self) -> u64 {
        self.total_duration().frame_count(self.settings.fps)
    }

    /// Add a scene to the project.
    pub fn add_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }

    /// Get a scene by its ID.
    pub fn get_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id.0 == id)
    }

    /// Get a mutable reference to a scene by its ID.
    pub fn get_scene_mut(&mut self, id: &str) -> Option<&mut Scene> {
        self.scenes.iter_mut().find(|s| s.id.0 == id)
    }
}

/// Global project settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Frames per second.
    pub fps: f64,
    /// Background color.
    pub background: vidra_core::Color,
}

impl ProjectSettings {
    /// Create settings for 1080p at 30fps.
    pub fn hd_30() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            background: vidra_core::Color::BLACK,
        }
    }

    /// Create settings for 1080p at 60fps.
    pub fn hd_60() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60.0,
            background: vidra_core::Color::BLACK,
        }
    }

    /// Create settings for 4K at 30fps.
    pub fn uhd_30() -> Self {
        Self {
            width: 3840,
            height: 2160,
            fps: 30.0,
            background: vidra_core::Color::BLACK,
        }
    }

    /// Create custom settings.
    pub fn custom(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            background: vidra_core::Color::BLACK,
        }
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self::hd_30()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Scene, SceneId};

    #[test]
    fn test_project_creation() {
        let project = Project::new(ProjectSettings::hd_30());
        assert_eq!(project.settings.width, 1920);
        assert_eq!(project.settings.height, 1080);
        assert!((project.settings.fps - 30.0).abs() < 0.001);
        assert!(project.scenes.is_empty());
    }

    #[test]
    fn test_project_total_duration() {
        let mut project = Project::new(ProjectSettings::hd_30());
        project.add_scene(Scene::new(
            SceneId::new("intro"),
            vidra_core::Duration::from_seconds(5.0),
        ));
        project.add_scene(Scene::new(
            SceneId::new("main"),
            vidra_core::Duration::from_seconds(10.0),
        ));
        assert!((project.total_duration().as_seconds() - 15.0).abs() < 0.001);
        assert_eq!(project.total_frames(), 450); // 15s * 30fps
    }

    #[test]
    fn test_project_get_scene() {
        let mut project = Project::new(ProjectSettings::hd_30());
        project.add_scene(Scene::new(
            SceneId::new("intro"),
            vidra_core::Duration::from_seconds(3.0),
        ));
        assert!(project.get_scene("intro").is_some());
        assert!(project.get_scene("nonexistent").is_none());
    }
}
