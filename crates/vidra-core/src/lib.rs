//! # vidra-core
//!
//! Core types and primitives for the Vidra video engine.
//! This crate contains foundational types shared across all Vidra crates:
//! frames, colors, transforms, durations, easing functions, and error types.

pub mod color;
pub mod config;
pub mod error;
pub mod frame;
pub mod hash;
pub mod math;
pub mod plugin;
pub mod time;
pub mod types;
pub mod vfx;

pub use config::*;

pub use color::Color;
pub use error::{VidraError, VidraResult};
pub use frame::{Frame, FrameBuffer, PixelFormat};
pub use math::{Point2D, Size2D, Transform2D};
pub use time::{Duration, Timestamp};
pub use types::{BlendMode, LayerEffect, LayerType};
