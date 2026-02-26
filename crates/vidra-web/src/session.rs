use crate::backend::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::Result;
use std::collections::HashMap;
use vidra_core::frame::FrameBuffer;

pub struct WebCaptureSession {
    config: WebCaptureSessionConfig,
    backend: Box<dyn WebCaptureBackend>,
    is_active: bool,
}

impl WebCaptureSession {
    pub fn new(config: WebCaptureSessionConfig, backend: Box<dyn WebCaptureBackend>) -> Self {
        Self {
            config,
            backend,
            is_active: false,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if !self.is_active {
            self.backend.start_session(self.config.clone()).await?;
            self.is_active = true;
        }
        Ok(())
    }

    pub async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer> {
        if !self.is_active {
            self.start().await?;
        }

        self.backend.capture_frame(time_seconds, variables).await
    }

    pub async fn stop(&mut self) -> Result<()> {
        if self.is_active {
            self.backend.stop_session().await?;
            self.is_active = false;
        }
        Ok(())
    }
}

impl Drop for WebCaptureSession {
    fn drop(&mut self) {
        if self.is_active {
            // Try to cleanly stop the session. If there's no tokio runtime available
            // (e.g., dropped from a plain thread), the backend's Drop impl handles cleanup.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let _ = tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        let _ = self.backend.stop_session().await;
                    })
                });
            }
        }
    }
}
