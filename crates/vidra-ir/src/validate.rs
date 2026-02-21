use crate::project::Project;
use vidra_core::VidraError;

/// Validate a Project IR node for structural correctness.
pub fn validate_project(project: &Project) -> Result<(), Vec<VidraError>> {
    let mut errors = Vec::new();

    // Settings validation
    if project.settings.width == 0 || project.settings.height == 0 {
        errors.push(VidraError::IrValidation(
            "project resolution must be non-zero".into(),
        ));
    }

    if project.settings.fps <= 0.0 {
        errors.push(VidraError::IrValidation(
            "project fps must be positive".into(),
        ));
    }

    // Scene validation
    if project.scenes.is_empty() {
        errors.push(VidraError::IrValidation(
            "project must have at least one scene".into(),
        ));
    }

    // Check for duplicate scene IDs
    let mut scene_ids = std::collections::HashSet::new();
    for scene in &project.scenes {
        if !scene_ids.insert(&scene.id) {
            errors.push(VidraError::IrValidation(format!(
                "duplicate scene id: {}",
                scene.id
            )));
        }

        // Validate each scene
        if scene.duration.as_seconds() <= 0.0 {
            errors.push(VidraError::IrValidation(format!(
                "scene '{}' has non-positive duration",
                scene.id
            )));
        }

        // Check for duplicate layer IDs within a scene
        let mut layer_ids = std::collections::HashSet::new();
        for layer in &scene.layers {
            if !layer_ids.insert(&layer.id) {
                errors.push(VidraError::IrValidation(format!(
                    "duplicate layer id '{}' in scene '{}'",
                    layer.id, scene.id
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectSettings;
    use crate::scene::{Scene, SceneId};

    #[test]
    fn test_validate_empty_project() {
        let project = Project::new(ProjectSettings::hd_30());
        let result = validate_project(&project);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_project() {
        let mut project = Project::new(ProjectSettings::hd_30());
        project.add_scene(Scene::new(
            SceneId::new("intro"),
            vidra_core::Duration::from_seconds(5.0),
        ));
        let result = validate_project(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_resolution() {
        let mut project = Project::new(ProjectSettings::custom(0, 1080, 30.0));
        project.add_scene(Scene::new(
            SceneId::new("s"),
            vidra_core::Duration::from_seconds(1.0),
        ));
        let result = validate_project(&project);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duplicate_scene_ids() {
        let mut project = Project::new(ProjectSettings::hd_30());
        project.add_scene(Scene::new(
            SceneId::new("intro"),
            vidra_core::Duration::from_seconds(5.0),
        ));
        project.add_scene(Scene::new(
            SceneId::new("intro"),
            vidra_core::Duration::from_seconds(3.0),
        ));
        let result = validate_project(&project);
        assert!(result.is_err());
    }
}
