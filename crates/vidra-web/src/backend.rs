use async_trait::async_trait;
use std::collections::HashMap;
use vidra_core::frame::FrameBuffer;
use vidra_ir::layer::WebCaptureMode;

#[derive(Debug, Clone)]
pub struct WebCaptureSessionConfig {
    pub source: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub mode: WebCaptureMode,
    pub wait_for: Option<String>,
    pub fps: f64,
    pub format: vidra_core::frame::PixelFormat,
}

#[async_trait]
pub trait WebCaptureBackend: Send + Sync {
    async fn start_session(&mut self, config: WebCaptureSessionConfig)
        -> Result<(), anyhow::Error>;

    async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer, anyhow::Error>;

    async fn stop_session(&mut self) -> Result<(), anyhow::Error>;
}
