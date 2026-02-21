use serde::{Deserialize, Serialize};

/// The kind of content a layer holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    /// Plain text rendered with a font.
    Text,
    /// A static image (PNG, JPEG, etc.).
    Image,
    /// A video clip.
    Video,
    /// An audio clip.
    Audio,
    /// A geometric shape (rect, circle, path).
    Shape,
    /// A solid color fill.
    Solid,
    /// A reusable component instance.
    Component,
    /// AI Text-to-Speech node.
    TTS,
    /// AI Auto-Caption node.
    AutoCaption,
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerType::Text => write!(f, "text"),
            LayerType::Image => write!(f, "image"),
            LayerType::Video => write!(f, "video"),
            LayerType::Audio => write!(f, "audio"),
            LayerType::Shape => write!(f, "shape"),
            LayerType::Solid => write!(f, "solid"),
            LayerType::Component => write!(f, "component"),
            LayerType::TTS => write!(f, "tts"),
            LayerType::AutoCaption => write!(f, "autocaption"),
        }
    }
}

/// Visual effect that can be applied to a rendered layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayerEffect {
    /// Blur effect with a specific radius.
    Blur(f64),
    /// Grayscale effect (0.0 to 1.0 intensity).
    Grayscale(f64),
    /// Invert colors (0.0 to 1.0 intensity).
    Invert(f64),
}

/// Blend mode for layer compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    /// Standard alpha blending (Porter-Duff "over").
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

/// Easing function for animation interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Linear
    }
}

impl Easing {
    /// Apply the easing function to a normalized time value t in [0, 1].
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::CubicIn => t * t * t,
            Easing::CubicOut => {
                let t1 = t - 1.0;
                t1 * t1 * t1 + 1.0
            }
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t1 = 2.0 * t - 2.0;
                    0.5 * t1 * t1 * t1 + 1.0
                }
            }
        }
    }
}

/// Shape variant for shape layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rect {
        width: f64,
        height: f64,
        corner_radius: f64,
    },
    Circle {
        radius: f64,
    },
    Ellipse {
        rx: f64,
        ry: f64,
    },
}

/// A Brand Kit containing predefined styling rules and assets.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BrandKit {
    pub name: String,
    pub colors: std::collections::HashMap<String, String>, // e.g. "primary" -> "FF0000"
    pub fonts: std::collections::HashMap<String, String>,  // e.g. "heading" -> "Inter"
    pub logos: std::collections::HashMap<String, String>,  // e.g. "symbol" -> "logo.png"
    pub numbers: std::collections::HashMap<String, f64>,   // e.g. "border_radius" -> 12.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let e = Easing::Linear;
        assert!((e.apply(0.0)).abs() < 0.001);
        assert!((e.apply(0.5) - 0.5).abs() < 0.001);
        assert!((e.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in() {
        let e = Easing::EaseIn;
        assert!((e.apply(0.0)).abs() < 0.001);
        assert!(e.apply(0.5) < 0.5); // easeIn is slower at start
        assert!((e.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_out() {
        let e = Easing::EaseOut;
        assert!((e.apply(0.0)).abs() < 0.001);
        assert!(e.apply(0.5) > 0.5); // easeOut is faster at start
        assert!((e.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_boundaries() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::CubicIn,
            Easing::CubicOut,
            Easing::CubicInOut,
        ] {
            assert!(
                (easing.apply(0.0)).abs() < 0.001,
                "{:?} should start at 0",
                easing
            );
            assert!(
                (easing.apply(1.0) - 1.0).abs() < 0.001,
                "{:?} should end at 1",
                easing
            );
        }
    }

    #[test]
    fn test_layer_type_display() {
        assert_eq!(format!("{}", LayerType::Text), "text");
        assert_eq!(format!("{}", LayerType::Solid), "solid");
    }
}
