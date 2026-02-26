// Linux WebKitGTK backend — placeholder for future implementation.
//
// Will use webkit2gtk crate for offscreen rendering,
// capturing frames via cairo::ImageSurface.

use crate::backend::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use vidra_core::frame::FrameBuffer;

pub struct PlatformWebViewBackend {
    _config: Option<WebCaptureSessionConfig>,
}

impl PlatformWebViewBackend {
    pub fn new() -> Self {
        Self { _config: None }
    }
}

#[async_trait]
impl WebCaptureBackend for PlatformWebViewBackend {
    async fn start_session(&mut self, _config: WebCaptureSessionConfig) -> Result<()> {
        Err(anyhow!(
            "Linux WebKitGTK backend not yet implemented — use --web-backend=playwright"
        ))
    }

    async fn capture_frame(
        &mut self,
        _time_seconds: f64,
        _variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer> {
        Err(anyhow!("Linux WebKitGTK backend not yet implemented"))
    }

    async fn stop_session(&mut self) -> Result<()> {
        Ok(())
    }
}
