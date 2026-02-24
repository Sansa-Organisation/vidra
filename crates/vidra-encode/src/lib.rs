//! # vidra-encode
//!
//! Encoding module — converts raw FrameBuffers to encoded video/image files.
//!
//! ## Encoders
//! - `FfmpegEncoder` — H.264 MP4 via FFmpeg subprocess
//! - `WebmEncoder` — VP9 WebM via FFmpeg subprocess (web-optimized, alpha support)
//! - `GifEncoder` — Native animated GIF (no external dependencies)
//! - `ApngEncoder` — Native animated PNG (lossless, no external dependencies)

pub mod ffmpeg;
pub mod webm;
pub mod gif;
pub mod apng;

pub use ffmpeg::{FfmpegEncoder, AudioTrack};
pub use webm::WebmEncoder;
pub use gif::GifEncoder;
pub use apng::ApngEncoder;
