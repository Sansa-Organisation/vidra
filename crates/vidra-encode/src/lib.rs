//! # vidra-encode
//!
//! Encoding module — converts raw FrameBuffers to encoded video files.
//! Phase 0: shells out to FFmpeg for H.264 encoding.
//! Future phases: native AV1 encoder, ProRes, streaming output.

pub mod ffmpeg;

pub use ffmpeg::FfmpegEncoder;
