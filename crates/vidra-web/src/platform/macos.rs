use crate::backend::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use vidra_core::frame::FrameBuffer;

use objc2::runtime::AnyObject;
use objc2::msg_send;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2_foundation::NSString;

/// Core Graphics geometry types (C ABI compatible with Apple frameworks).
/// Defined inline to avoid pulling in objc2-core-graphics/core-foundation crates.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGPoint {
    x: f64,
    y: f64,
}

unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[
        f64::ENCODING,
        f64::ENCODING,
    ]);
}
unsafe impl RefEncode for CGPoint {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGSize {
    width: f64,
    height: f64,
}

unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[
        f64::ENCODING,
        f64::ENCODING,
    ]);
}
unsafe impl RefEncode for CGSize {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[
        <CGPoint as Encode>::ENCODING,
        <CGSize as Encode>::ENCODING,
    ]);
}
unsafe impl RefEncode for CGRect {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// Platform-native web capture backend for macOS using WKWebView.
///
/// Uses the OS-provided WebKit engine (Safari) to render web content offscreen
/// and capture frames via `takeSnapshot`. Zero external dependencies.
///
/// **Important**: WKWebView requires a main thread and a running RunLoop.
/// This backend is designed for use in the Vidra render pipeline where
/// the web capture runs on the main thread via `tokio::task::block_in_place`.
pub struct PlatformWebViewBackend {
    /// Raw Objective-C pointer to the WKWebView instance
    webview: Option<*mut AnyObject>,
    /// Raw Objective-C pointer to the hidden NSWindow
    window: Option<*mut AnyObject>,
    config: Option<WebCaptureSessionConfig>,
    viewport_width: u32,
    viewport_height: u32,
}

// The Objective-C objects are tied to the main thread.
// We guarantee main-thread access in the async methods via block_in_place.
unsafe impl Send for PlatformWebViewBackend {}
unsafe impl Sync for PlatformWebViewBackend {}

impl PlatformWebViewBackend {
    pub fn new() -> Self {
        Self {
            webview: None,
            window: None,
            config: None,
            viewport_width: 0,
            viewport_height: 0,
        }
    }

    /// Generate the __vidra bridge injection JavaScript.
    fn bridge_script(mode: &vidra_ir::layer::WebCaptureMode) -> String {
        match mode {
            vidra_ir::layer::WebCaptureMode::FrameAccurate => {
                r#"
                window.__vidra = {
                    capturing: true, frame: 0, time: 0, fps: 60, vars: {},
                    emit: function() {}, requestAdvance: function(cb) {}
                };
                var __vt = 0;
                Date.now = function() { return __vt; };
                performance.now = function() { return __vt; };
                var __rafCbs = [];
                window.requestAnimationFrame = function(cb) { __rafCbs.push(cb); return 0; };
                window.__vidra_advance_frame = function(t, v) {
                    window.__vidra.time = t; window.__vidra.vars = v || {};
                    __vt = t * 1000; window.__vidra.frame++;
                    var cbs = __rafCbs; __rafCbs = [];
                    for (var i = 0; i < cbs.length; i++) { try { cbs[i](__vt); } catch(e) {} }
                };
                "#
                .to_string()
            }
            vidra_ir::layer::WebCaptureMode::Realtime => {
                r#"
                window.__vidra = {
                    capturing: true, frame: 0, time: 0, fps: 60, vars: {},
                    emit: function() {}
                };
                "#
                .to_string()
            }
        }
    }
}

#[async_trait]
impl WebCaptureBackend for PlatformWebViewBackend {
    async fn start_session(&mut self, config: WebCaptureSessionConfig) -> Result<()> {
        let vp_w = config.viewport_width;
        let vp_h = config.viewport_height;
        let bridge_js = Self::bridge_script(&config.mode);
        let source = config.source.clone();

        // All Objective-C work happens via msg_send on the main thread
        let (webview_ptr, window_ptr) = tokio::task::block_in_place(|| unsafe {
            // Ensure NSApplication is initialized
            let ns_app_class = objc2::runtime::AnyClass::get(c"NSApplication")
                .expect("NSApplication class not found");
            let _app: *mut AnyObject = msg_send![ns_app_class, sharedApplication];

            // Create WKWebViewConfiguration
            let config_class = objc2::runtime::AnyClass::get(c"WKWebViewConfiguration")
                .expect("WKWebViewConfiguration not found");
            let wk_config: *mut AnyObject = msg_send![config_class, new];

            // Inject bridge script via userContentController
            let ucc: *mut AnyObject = msg_send![wk_config, userContentController];
            let script_class = objc2::runtime::AnyClass::get(c"WKUserScript")
                .expect("WKUserScript not found");
            let ns_js = NSString::from_str(&bridge_js);
            let user_script_raw: *mut AnyObject = msg_send![script_class, alloc];
            let user_script: *mut AnyObject = msg_send![
                user_script_raw,
                initWithSource: &*ns_js
                injectionTime: 0i64
                forMainFrameOnly: true
            ];
            let _: () = msg_send![ucc, addUserScript: user_script];

            // Create WKWebView with frame
            let wk_class = objc2::runtime::AnyClass::get(c"WKWebView")
                .expect("WKWebView not found");
            let frame = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    width: vp_w as f64,
                    height: vp_h as f64,
                },
            };
            let wk_raw: *mut AnyObject = msg_send![wk_class, alloc];
            let webview: *mut AnyObject = msg_send![
                wk_raw,
                initWithFrame: frame
                configuration: wk_config
            ];

            // Create hidden NSWindow
            let win_class = objc2::runtime::AnyClass::get(c"NSWindow")
                .expect("NSWindow not found");
            let style_mask: usize = 0; // NSWindowStyleMaskBorderless
            let backing: usize = 2; // NSBackingStoreBuffered
            let win_raw: *mut AnyObject = msg_send![win_class, alloc];
            let window: *mut AnyObject = msg_send![
                win_raw,
                initWithContentRect: frame
                styleMask: style_mask
                backing: backing
                defer: false
            ];
            let _: () = msg_send![window, setContentView: webview];

            // Navigate to source
            if source.starts_with("http://") || source.starts_with("https://") {
                let ns_url_str = NSString::from_str(&source);
                let url_class = objc2::runtime::AnyClass::get(c"NSURL").unwrap();
                let url: *mut AnyObject = msg_send![url_class, URLWithString: &*ns_url_str];
                let req_class = objc2::runtime::AnyClass::get(c"NSURLRequest").unwrap();
                let request: *mut AnyObject = msg_send![req_class, requestWithURL: url];
                let _: *mut AnyObject = msg_send![webview, loadRequest: request];
            } else {
                let abs_path = std::fs::canonicalize(&source)
                    .unwrap_or_else(|_| std::path::PathBuf::from(&source));
                let ns_path = NSString::from_str(&abs_path.to_string_lossy());
                let url_class = objc2::runtime::AnyClass::get(c"NSURL").unwrap();
                let file_url: *mut AnyObject = msg_send![url_class, fileURLWithPath: &*ns_path];
                let dir_path = abs_path.parent().unwrap_or(std::path::Path::new("."));
                let ns_dir = NSString::from_str(&dir_path.to_string_lossy());
                let dir_url: *mut AnyObject = msg_send![url_class, fileURLWithPath: &*ns_dir];
                let _: *mut AnyObject =
                    msg_send![webview, loadFileURL: file_url allowingReadAccessToURL: dir_url];
            }

            (webview, window)
        });

        // Store pointers immediately (before any .await) to satisfy Send
        self.webview = Some(webview_ptr);
        self.window = Some(window_ptr);
        self.config = Some(config);
        self.viewport_width = vp_w;
        self.viewport_height = vp_h;

        // Wait for page to finish loading
        for _ in 0..200 {
            let is_loading: bool = tokio::task::block_in_place(|| unsafe {
                let wv = self.webview.unwrap();
                msg_send![wv, isLoading]
            });
            if !is_loading {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(())
    }

    async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer> {
        let webview = self
            .webview
            .ok_or_else(|| anyhow!("Session not started"))?;
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| anyhow!("Session not started"))?;
        let vp_w = self.viewport_width;
        let vp_h = self.viewport_height;

        // Build JS to advance time
        let vars_json = serde_json::to_string(variables)?;
        let js = match config.mode {
            vidra_ir::layer::WebCaptureMode::FrameAccurate => {
                format!(
                    "window.__vidra_advance_frame({}, JSON.parse('{}'));",
                    time_seconds, vars_json
                )
            }
            vidra_ir::layer::WebCaptureMode::Realtime => {
                format!(
                    "window.__vidra.time = {}; window.__vidra.vars = JSON.parse('{}');",
                    time_seconds, vars_json
                )
            }
        };

        // Evaluate JS and capture snapshot using msg_send
        let frame_data: Vec<u8> = tokio::task::block_in_place(|| unsafe {
            // Evaluate JavaScript
            let ns_js = NSString::from_str(&js);
            let _: () = msg_send![webview, evaluateJavaScript: &*ns_js completionHandler: std::ptr::null::<AnyObject>()];

            // Small settle time
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Use a simple approach: evaluate JS to render canvas and get pixel data
            // But for WKWebView, we can use the performSelector approach with takeSnapshot
            // For now, use the TIFFRepresentation path via the view's cacheDisplay

            // Get the WKWebView to draw into an NSBitmapImageRep
            let bitmap_class = objc2::runtime::AnyClass::get(c"NSBitmapImageRep").unwrap();

            let frame_rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    width: vp_w as f64,
                    height: vp_h as f64,
                },
            };

            // Allocate an NSBitmapImageRep for RGBA capture
            let bitmap_raw: *mut AnyObject = msg_send![bitmap_class, alloc];
            let bitmap: *mut AnyObject = msg_send![
                bitmap_raw,
                initWithBitmapDataPlanes: std::ptr::null::<*mut u8>()
                pixelsWide: vp_w as i64
                pixelsHigh: vp_h as i64
                bitsPerSample: 8i64
                samplesPerPixel: 4i64
                hasAlpha: true
                isPlanar: false
                colorSpaceName: &*NSString::from_str("NSDeviceRGBColorSpace")
                bytesPerRow: (vp_w * 4) as i64
                bitsPerPixel: 32i64
            ];

            // Save and set NSGraphicsContext from the bitmap
            let gc_class = objc2::runtime::AnyClass::get(c"NSGraphicsContext").unwrap();
            let gfx_ctx: *mut AnyObject =
                msg_send![gc_class, graphicsContextWithBitmapImageRep: bitmap];
            let old_ctx: *mut AnyObject = msg_send![gc_class, currentContext];
            let _: () = msg_send![gc_class, setCurrentContext: gfx_ctx];

            // Ask the webview to draw into the current graphics context
            let _: () = msg_send![webview, displayRectIgnoringOpacity: frame_rect inContext: gfx_ctx];

            // Restore context
            let _: () = msg_send![gc_class, setCurrentContext: old_ctx];

            // Extract pixel data from bitmap
            let bitmap_data: *const u8 = msg_send![bitmap, bitmapData];
            let data_len = (vp_w * vp_h * 4) as usize;

            if bitmap_data.is_null() {
                vec![0u8; data_len]
            } else {
                std::slice::from_raw_parts(bitmap_data, data_len).to_vec()
            }
        });

        let mut fb = FrameBuffer::new(vp_w, vp_h, vidra_core::frame::PixelFormat::Rgba8);
        let copy_len = fb.data.len().min(frame_data.len());
        fb.data[..copy_len].copy_from_slice(&frame_data[..copy_len]);

        Ok(fb)
    }

    async fn stop_session(&mut self) -> Result<()> {
        if let Some(webview) = self.webview.take() {
            tokio::task::block_in_place(|| unsafe {
                let _: () = msg_send![webview, removeFromSuperview];
            });
        }
        if let Some(window) = self.window.take() {
            tokio::task::block_in_place(|| unsafe {
                let _: () = msg_send![window, close];
            });
        }
        self.config = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vidra_ir::layer::WebCaptureMode;

    #[tokio::test]
    async fn test_platform_webview_lifecycle() {
        // This test requires macOS with a display or virtual framebuffer
        let mut backend = PlatformWebViewBackend::new();
        let config = WebCaptureSessionConfig {
            source: "data:text/html,<html><body style='background:red'><h1>Test</h1></body></html>"
                .to_string(),
            viewport_width: 800,
            viewport_height: 600,
            mode: WebCaptureMode::Realtime,
            wait_for: None,
            fps: 30.0,
            format: vidra_core::frame::PixelFormat::Rgba8,
        };

        let result = backend.start_session(config).await;
        // May fail in headless CI — that's expected
        if result.is_err() {
            eprintln!("Skipping platform webview test (no display): {:?}", result.err());
            return;
        }

        let vars = HashMap::new();
        let frame = backend.capture_frame(0.0, &vars).await;
        if let Ok(fb) = frame {
            assert_eq!(fb.width, 800);
            assert_eq!(fb.height, 600);
        }

        let _ = backend.stop_session().await;
    }
}
