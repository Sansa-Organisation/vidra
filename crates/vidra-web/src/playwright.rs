use super::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use image::ImageFormat;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use vidra_core::frame::FrameBuffer;

pub struct PlaywrightBackend {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl PlaywrightBackend {
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            stdout: None,
        }
    }

    fn read_response(&mut self) -> anyhow::Result<serde_json::Value> {
        let stdout = self
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("Process stdout not available"))?;
        let mut line = String::new();
        stdout.read_line(&mut line)?;
        if line.is_empty() {
            return Err(anyhow!("Browser process closed unexpectedly"));
        }
        let res: serde_json::Value = serde_json::from_str(&line)?;
        if res["type"] == "error" {
            return Err(anyhow!("Browser error: {}", res["error"]));
        }
        Ok(res)
    }

    fn send_request(&mut self, req: serde_json::Value) -> anyhow::Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Process stdin not available"))?;
        let msg = req.to_string() + "\n";
        stdin.write_all(msg.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }
}

#[async_trait]
impl WebCaptureBackend for PlaywrightBackend {
    async fn start_session(&mut self, config: WebCaptureSessionConfig) -> anyhow::Result<()> {
        // In tests, the current dir can be inside 'crates/vidra-web' already,
        // so we just check what's actually there.
        let mut script_path = std::env::current_dir()?
            .join("crates")
            .join("vidra-web")
            .join("scripts")
            .join("capture.js");
        if !script_path.exists() {
            // Probably inside the crate
            script_path = std::env::current_dir()?.join("scripts").join("capture.js");
        }

        let mut child = Command::new("node")
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn node process for Playwright capture script")?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);

        let req = json!({
            "type": "start",
            "config": {
                "source": config.source,
                "viewport_width": config.viewport_width,
                "viewport_height": config.viewport_height,
                "mode": match config.mode {
                    vidra_ir::layer::WebCaptureMode::FrameAccurate => "frame-accurate",
                    vidra_ir::layer::WebCaptureMode::Realtime => "realtime",
                }
            }
        });

        self.send_request(req)?;

        let res = tokio::task::block_in_place(|| self.read_response())?;
        if res["type"] != "ready" {
            return Err(anyhow!("Browser did not return 'ready'"));
        }

        Ok(())
    }

    async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> anyhow::Result<FrameBuffer> {
        let vars_json = serde_json::to_value(variables)?;
        let req = json!({
            "type": "capture",
            "time": time_seconds,
            "vars": vars_json
        });

        self.send_request(req)?;

        let res = tokio::task::block_in_place(|| self.read_response())?;
        if res["type"] != "frame" {
            return Err(anyhow!("Expected 'frame' response, got something else"));
        }

        let b64_str = res["data"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing frame data"))?;

        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(b64_str)?;
        let img = image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg)?;
        let rgba = img.to_rgba8();

        let width = rgba.width();
        let height = rgba.height();

        let mut fb = FrameBuffer::new(width, height, vidra_core::frame::PixelFormat::Rgba8);
        fb.data.copy_from_slice(rgba.as_raw());

        Ok(fb)
    }

    async fn stop_session(&mut self) -> anyhow::Result<()> {
        if self.process.is_some() {
            let req = json!({ "type": "stop" });
            let _ = self.send_request(req);

            // Wait for subprocess
            if let Some(mut child) = self.process.take() {
                let _ = tokio::task::block_in_place(|| child.wait());
            }
            self.stdin = None;
            self.stdout = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod playwright_tests;
