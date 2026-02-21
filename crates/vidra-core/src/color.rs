use serde::{Deserialize, Serialize};
use std::fmt;

/// RGBA color representation with f32 components in [0.0, 1.0] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new RGBA color.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque RGB color (alpha = 1.0).
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "#FF0000FF").
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ColorError::InvalidHex)?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ColorError::InvalidHex)?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ColorError::InvalidHex)?;
                Ok(Self::rgb(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                ))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ColorError::InvalidHex)?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ColorError::InvalidHex)?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ColorError::InvalidHex)?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| ColorError::InvalidHex)?;
                Ok(Self::rgba(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ))
            }
            _ => Err(ColorError::InvalidHex),
        }
    }

    /// Convert to RGBA u8 tuple.
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r * 255.0).clamp(0.0, 255.0) as u8,
            (self.g * 255.0).clamp(0.0, 255.0) as u8,
            (self.b * 255.0).clamp(0.0, 255.0) as u8,
            (self.a * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    /// Linearly interpolate between two colors.
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    // --- Named constants ---

    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b, a] = self.to_rgba8();
        if a == 255 {
            write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            write!(f, "#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("invalid hex color string")]
    InvalidHex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex_rgb() {
        let c = Color::from_hex("#FF8800").unwrap();
        assert_eq!(c.to_rgba8(), [255, 136, 0, 255]);
    }

    #[test]
    fn test_color_from_hex_rgba() {
        let c = Color::from_hex("#FF880080").unwrap();
        assert_eq!(c.to_rgba8(), [255, 136, 0, 128]);
    }

    #[test]
    fn test_color_from_hex_no_hash() {
        let c = Color::from_hex("00FF00").unwrap();
        assert_eq!(c.to_rgba8(), [0, 255, 0, 255]);
    }

    #[test]
    fn test_color_from_hex_invalid() {
        assert!(Color::from_hex("invalid").is_err());
        assert!(Color::from_hex("#GG0000").is_err());
    }

    #[test]
    fn test_color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(&b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_display() {
        assert_eq!(format!("{}", Color::RED), "#FF0000");
        assert_eq!(format!("{}", Color::rgba(1.0, 0.0, 0.0, 0.5)), "#FF00007F");
    }
}
