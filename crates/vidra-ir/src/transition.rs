use serde::{Deserialize, Serialize};
use vidra_core::types::Easing;
use vidra_core::Duration;

/// Defines the visual effect used to transition into this scene from the previous one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransitionType {
    /// A smooth crossfade from the previous scene to the current scene.
    Crossfade,
    /// The new scene slides in from a specified direction ("up", "down", "left", "right").
    Slide { direction: String },
    /// The new scene pushes the old scene out in a specified direction.
    Push { direction: String },
    /// The new scene wipes in from a specified direction.
    Wipe { direction: String },
}

/// A scene transition definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// The type of transition effect.
    pub effect: TransitionType,
    /// The duration of the transition.
    pub duration: Duration,
    /// The easing curve of the transition.
    pub easing: Easing,
}
