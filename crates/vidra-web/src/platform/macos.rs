use crate::backend::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use vidra_core::frame::FrameBuffer;

use objc2::runtime::AnyObject;
use objc2::msg_send;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2_foundation::NSString;

// Ensure AppKit and WebKit frameworks are linked into the binary.
#[link(name = "AppKit", kind = "framework")]
extern "C" {}
#[link(name = "WebKit", kind = "framework")]
extern "C" {}

/// Core Graphics geometry types (C ABI compatible with Apple frameworks).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGPoint { x: f64, y: f64 }

unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}
unsafe impl RefEncode for CGPoint {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGSize { width: f64, height: f64 }

unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}
unsafe impl RefEncode for CGSize {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGRect { origin: CGPoint, size: CGSize }

unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[
        <CGPoint as Encode>::ENCODING, <CGSize as Encode>::ENCODING,
    ]);
}
unsafe impl RefEncode for CGRect {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// Run the CFRunLoop briefly to process pending events.
/// This is needed for WKWebView to process navigation and rendering.
unsafe fn pump_runloop(seconds: f64) {
    extern "C" {
        fn CFRunLoopRunInMode(
            mode: *const std::ffi::c_void,
            seconds: f64,
            returnAfterSourceHandled: u8,
        ) -> i32;
        // kCFRunLoopDefaultMode is a global CFStringRef constant
        static kCFRunLoopDefaultMode: *const std::ffi::c_void;
    }
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, 0);
}

/// Platform-native web capture backend for macOS using WKWebView.
///
/// Since WKWebView operations in a CLI app can run on whatever thread tokio
/// assigns (via block_in_place), we use NSObject's performSelectorOnMainThread
/// approach — but for a CLI app without a running RunLoop, the simplest reliable
/// approach is to create and use the WKWebView directly (it works from any thread
/// as long as we pump the RunLoop manually).
pub struct PlatformWebViewBackend {
    webview: Option<*mut AnyObject>,
    window: Option<*mut AnyObject>,
    config: Option<WebCaptureSessionConfig>,
    viewport_width: u32,
    viewport_height: u32,
}

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
                "#.to_string()
            }
            vidra_ir::layer::WebCaptureMode::Realtime => {
                r#"
                window.__vidra = {
                    capturing: true, frame: 0, time: 0, fps: 60, vars: {},
                    emit: function() {}
                };
                "#.to_string()
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

        let (webview_ptr, window_ptr) = tokio::task::block_in_place(|| unsafe {
            // Ensure NSApplication is initialized
            let ns_app_class = objc2::runtime::AnyClass::get(c"NSApplication")
                .expect("NSApplication class not found");
            let _app: *mut AnyObject = msg_send![ns_app_class, sharedApplication];

            // Create WKWebViewConfiguration
            let config_class = objc2::runtime::AnyClass::get(c"WKWebViewConfiguration")
                .expect("WKWebViewConfiguration not found");
            let wk_config: *mut AnyObject = msg_send![config_class, new];

            // Inject bridge script
            let ucc: *mut AnyObject = msg_send![wk_config, userContentController];
            let script_class = objc2::runtime::AnyClass::get(c"WKUserScript")
                .expect("WKUserScript not found");
            let ns_js = NSString::from_str(&bridge_js);
            let script_raw: *mut AnyObject = msg_send![script_class, alloc];
            let user_script: *mut AnyObject = msg_send![
                script_raw,
                initWithSource: &*ns_js
                injectionTime: 0i64
                forMainFrameOnly: true
            ];
            let _: () = msg_send![ucc, addUserScript: user_script];

            // Create WKWebView
            let wk_class = objc2::runtime::AnyClass::get(c"WKWebView")
                .expect("WKWebView not found");
            let frame = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width: vp_w as f64, height: vp_h as f64 },
            };
            let wk_raw: *mut AnyObject = msg_send![wk_class, alloc];
            let webview: *mut AnyObject = msg_send![
                wk_raw,
                initWithFrame: frame
                configuration: wk_config
            ];

            // Create hidden NSWindow to host the webview
            let win_class = objc2::runtime::AnyClass::get(c"NSWindow")
                .expect("NSWindow not found");
            let win_raw: *mut AnyObject = msg_send![win_class, alloc];
            let window: *mut AnyObject = msg_send![
                win_raw,
                initWithContentRect: frame
                styleMask: 0usize
                backing: 2usize
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
                let _: *mut AnyObject = msg_send![
                    webview, loadFileURL: file_url allowingReadAccessToURL: dir_url
                ];
            }

            // Pump the RunLoop to let WebKit start loading
            pump_runloop(0.1);

            // Wait for page to finish loading
            for _ in 0..200 {
                let is_loading: bool = msg_send![webview, isLoading];
                if !is_loading {
                    break;
                }
                pump_runloop(0.05);
            }

            // Extra settle for rendering
            pump_runloop(0.2);

            (webview, window)
        });

        self.webview = Some(webview_ptr);
        self.window = Some(window_ptr);
        self.config = Some(config);
        self.viewport_width = vp_w;
        self.viewport_height = vp_h;

        Ok(())
    }

    async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer> {
        let webview = self.webview.ok_or_else(|| anyhow!("Session not started"))?;
        let config = self.config.as_ref().ok_or_else(|| anyhow!("Session not started"))?;
        let vp_w = self.viewport_width;
        let vp_h = self.viewport_height;

        let vars_json = serde_json::to_string(variables)?;
        let js = match config.mode {
            vidra_ir::layer::WebCaptureMode::FrameAccurate => {
                format!("window.__vidra_advance_frame({}, JSON.parse('{}'));", time_seconds, vars_json)
            }
            vidra_ir::layer::WebCaptureMode::Realtime => {
                format!("window.__vidra.time = {}; window.__vidra.vars = JSON.parse('{}');", time_seconds, vars_json)
            }
        };

        let frame_data: Vec<u8> = tokio::task::block_in_place(|| unsafe {
            // Evaluate JavaScript
            let ns_js = NSString::from_str(&js);
            let _: () = msg_send![
                webview,
                evaluateJavaScript: &*ns_js
                completionHandler: std::ptr::null::<AnyObject>()
            ];

            // Pump RunLoop to let JS execute and render
            pump_runloop(0.02);

            // Capture via displayRectIgnoringOpacity into NSBitmapImageRep
            let bitmap_class = objc2::runtime::AnyClass::get(c"NSBitmapImageRep").unwrap();
            let frame_rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width: vp_w as f64, height: vp_h as f64 },
            };

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

            let gc_class = objc2::runtime::AnyClass::get(c"NSGraphicsContext").unwrap();
            let gfx_ctx: *mut AnyObject =
                msg_send![gc_class, graphicsContextWithBitmapImageRep: bitmap];
            let old_ctx: *mut AnyObject = msg_send![gc_class, currentContext];
            let _: () = msg_send![gc_class, setCurrentContext: gfx_ctx];

            // Draw the webview into the bitmap context
            let _: () = msg_send![
                webview,
                displayRectIgnoringOpacity: frame_rect
                inContext: gfx_ctx
            ];

            let _: () = msg_send![gc_class, setCurrentContext: old_ctx];

            // Extract pixel data
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
        if let (Some(webview), Some(window)) = (self.webview.take(), self.window.take()) {
            tokio::task::block_in_place(|| unsafe {
                let _: () = msg_send![webview, removeFromSuperview];
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_platform_webview_lifecycle() {
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
        if result.is_err() {
            eprintln!("Skipping test (no display): {:?}", result.err());
            return;
        }

        let vars = HashMap::new();
        if let Ok(fb) = backend.capture_frame(0.0, &vars).await {
            assert_eq!(fb.width, 800);
            assert_eq!(fb.height, 600);
        }

        let _ = backend.stop_session().await;
    }
}
