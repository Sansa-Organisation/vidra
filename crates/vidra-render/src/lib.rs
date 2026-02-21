//! # vidra-render
//!
//! The Vidra rendering engine. Takes a validated IR and produces raw frame buffers.
//! This is the single-threaded, CPU-only prototype renderer (Phase 0).
//! GPU acceleration comes in Phase 1.

pub mod compositor;
pub mod image_loader;
pub mod pipeline;
pub mod text;
pub mod video_decoder;
pub mod gpu;
pub mod effects;

pub use pipeline::{RenderContext, RenderPipeline, RenderResult};
pub use gpu::GpuContext;
pub use video_decoder::VideoDecoder;
