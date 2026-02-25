//! # vidra-render
//!
//! The Vidra rendering engine. Takes a validated IR and produces raw frame buffers.
//! This is the single-threaded, CPU-only prototype renderer (Phase 0).
//! GPU acceleration comes in Phase 1.

pub mod compositor;
pub mod custom_shader;
pub mod effects;
pub mod gpu;
pub mod image_loader;
pub mod pipeline;
pub mod text;
pub mod video_decoder;

pub use gpu::GpuContext;
pub use pipeline::{RenderContext, RenderPipeline, RenderResult};
pub use video_decoder::VideoDecoder;
