use super::*;
use std::collections::HashMap;
use vidra_core::frame::PixelFormat;
use vidra_ir::layer::WebCaptureMode;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_playwright_backend_start_stop() {
    let mut backend = PlaywrightBackend::new();
    let config = WebCaptureSessionConfig {
        source: "data:text/html,<html><body><h1>Test</h1></body></html>".to_string(),
        viewport_width: 800,
        viewport_height: 600,
        mode: WebCaptureMode::Realtime,
        wait_for: None,
        fps: 30.0,
        format: PixelFormat::Rgba8,
    };

    // Should return Ok
    let res = backend.start_session(config).await;
    assert!(res.is_ok(), "Failed to start session: {:?}", res);

    let stop_res = backend.stop_session().await;
    assert!(stop_res.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_playwright_backend_capture() {
    let mut backend = PlaywrightBackend::new();
    let config = WebCaptureSessionConfig {
        source: "data:text/html,<html style='background:red'><body>Test</body></html>".to_string(),
        viewport_width: 100,
        viewport_height: 100,
        mode: WebCaptureMode::Realtime,
        wait_for: None,
        fps: 30.0,
        format: PixelFormat::Rgba8,
    };

    backend
        .start_session(config)
        .await
        .expect("Failed to start");

    let vars = HashMap::new();
    let frame = backend
        .capture_frame(0.0, &vars)
        .await
        .expect("Failed to capture frame");
    assert_eq!(frame.width, 100);
    assert_eq!(frame.height, 100);

    backend.stop_session().await.expect("Failed to stop");
}
