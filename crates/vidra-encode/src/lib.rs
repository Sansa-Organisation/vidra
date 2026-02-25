//! # vidra-encode
//!
//! Encoding module — converts raw FrameBuffers to encoded video/image files.
//!
//! ## Encoders
//! - `FfmpegEncoder` — H.264 MP4 via FFmpeg subprocess
//! - `WebmEncoder` — VP9 WebM via FFmpeg subprocess (web-optimized, alpha support)
//! - `GifEncoder` — Native animated GIF (no external dependencies)
//! - `ApngEncoder` — Native animated PNG (lossless, no external dependencies)

pub mod apng;
pub mod ffmpeg;
pub mod gif;
pub mod webm;

pub use apng::ApngEncoder;
pub use ffmpeg::{AudioTrack, FfmpegEncoder};
pub use gif::GifEncoder;
pub use webm::WebmEncoder;
