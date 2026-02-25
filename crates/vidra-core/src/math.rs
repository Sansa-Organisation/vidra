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

    /// 2.5D Z translation (pixels-ish). Only meaningful when `perspective > 0`.
    #[serde(default)]
    pub translate_z: f64,
    /// 2.5D rotation around X axis (degrees). Only meaningful when `perspective > 0`.
    #[serde(default)]
    pub rotate_x: f64,
    /// 2.5D rotation around Y axis (degrees). Only meaningful when `perspective > 0`.
    #[serde(default)]
    pub rotate_y: f64,
    /// Perspective distance. When <= 0, perspective is disabled and the transform behaves as 2D.
    ///
    /// Interpreted as a focal-length-like distance in pixels.
    #[serde(default)]
    pub perspective: f64,
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
            translate_z: 0.0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            perspective: 0.0,
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
            translate_z: self.translate_z + (other.translate_z - self.translate_z) * t,
            rotate_x: self.rotate_x + (other.rotate_x - self.rotate_x) * t,
            rotate_y: self.rotate_y + (other.rotate_y - self.rotate_y) * t,
            perspective: self.perspective + (other.perspective - self.perspective) * t,
            anchor: self.anchor.lerp(&other.anchor, t),
            opacity: self.opacity + (other.opacity - self.opacity) * t,
        }
    }

    /// Project the 4 corners of a layer's local rectangle into screen space.
    ///
    /// - `width`/`height` are the layer's pixel dimensions (after any CPU scaling).
    /// - The layer's anchor defines the pivot within that rectangle.
    /// - The returned points are ordered: top-left, top-right, bottom-right, bottom-left.
    pub fn project_corners(&self, width: f64, height: f64) -> [[f64; 2]; 4] {
        // Local corners in pixels, with pivot at anchor.
        let ax = self.anchor.x;
        let ay = self.anchor.y;
        let left = -ax * width;
        let right = (1.0 - ax) * width;
        let top = -ay * height;
        let bottom = (1.0 - ay) * height;

        let corners = [
            [left, top, 0.0],
            [right, top, 0.0],
            [right, bottom, 0.0],
            [left, bottom, 0.0],
        ];

        let rz = self.rotation.to_radians();
        let rx = self.rotate_x.to_radians();
        let ry = self.rotate_y.to_radians();

        let (sz, cz) = rz.sin_cos();
        let (sx, cx) = rx.sin_cos();
        let (sy, cy) = ry.sin_cos();

        let persp = self.perspective;

        let mut out = [[0.0; 2]; 4];
        for (i, c) in corners.iter().enumerate() {
            let mut x = c[0];
            let mut y = c[1];
            let mut z = c[2];

            // Rotate Z (2D rotation).
            let xz = x * cz - y * sz;
            let yz = x * sz + y * cz;
            x = xz;
            y = yz;

            // Rotate X.
            let yx = y * cx - z * sx;
            let zx = y * sx + z * cx;
            y = yx;
            z = zx;

            // Rotate Y.
            let xy = x * cy + z * sy;
            let zy = -x * sy + z * cy;
            x = xy;
            z = zy;

            // Translate.
            x += self.position.x;
            y += self.position.y;
            z += self.translate_z;

            // Perspective projection.
            if persp > 0.0 {
                let denom = (persp + z).max(1e-6);
                let f = persp / denom;
                x = self.position.x + (x - self.position.x) * f;
                y = self.position.y + (y - self.position.y) * f;
            }

            out[i] = [x, y];
        }

        out
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
