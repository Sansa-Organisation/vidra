use serde::{Deserialize, Serialize};

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Linear interpolation between two points.
    pub fn lerp(&self, other: &Point2D, t: f64) -> Point2D {
        Point2D {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl Default for Point2D {
    fn default() -> Self {
        Self::zero()
    }
}

/// A 2D size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size2D {
    pub width: f64,
    pub height: f64,
}

impl Size2D {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Compute the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0.0 {
            return 0.0;
        }
        self.width / self.height
    }
}

/// A 2D affine transform: position, scale, rotation, and anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// Position offset (translation).
    pub position: Point2D,
    /// Scale factors.
    pub scale: Point2D,
    /// Rotation in degrees.
    pub rotation: f64,
    /// Anchor point (0.0–1.0 normalized, 0.5/0.5 = center).
    pub anchor: Point2D,
    /// Opacity (0.0–1.0).
    pub opacity: f64,
}

impl Transform2D {
    /// Identity transform: no translation, scale 1, no rotation, centered anchor, fully opaque.
    pub fn identity() -> Self {
        Self {
            position: Point2D::zero(),
            scale: Point2D::new(1.0, 1.0),
            rotation: 0.0,
            anchor: Point2D::new(0.5, 0.5),
            opacity: 1.0,
        }
    }

    /// Linear interpolation between two transforms.
    pub fn lerp(&self, other: &Transform2D, t: f64) -> Transform2D {
        let t = t.clamp(0.0, 1.0);
        Transform2D {
            position: self.position.lerp(&other.position, t),
            scale: self.scale.lerp(&other.scale, t),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            anchor: self.anchor.lerp(&other.anchor, t),
            opacity: self.opacity + (other.opacity - self.opacity) * t,
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_lerp() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(10.0, 20.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 0.001);
        assert!((mid.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_size_aspect_ratio() {
        let s = Size2D::new(1920.0, 1080.0);
        assert!((s.aspect_ratio() - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform2D::identity();
        assert_eq!(t.position, Point2D::zero());
        assert_eq!(t.scale, Point2D::new(1.0, 1.0));
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.opacity, 1.0);
    }

    #[test]
    fn test_transform_lerp() {
        let a = Transform2D::identity();
        let mut b = Transform2D::identity();
        b.position = Point2D::new(100.0, 200.0);
        b.opacity = 0.0;
        let mid = a.lerp(&b, 0.5);
        assert!((mid.position.x - 50.0).abs() < 0.001);
        assert!((mid.opacity - 0.5).abs() < 0.001);
    }
}
