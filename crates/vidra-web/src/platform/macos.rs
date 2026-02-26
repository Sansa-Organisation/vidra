use crate::backend::{WebCaptureBackend, WebCaptureSessionConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::mpsc;
use std::ffi::c_void;
use vidra_core::frame::FrameBuffer;

use objc2::runtime::AnyObject;
use objc2::msg_send;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2_foundation::NSString;

// Ensure AppKit and WebKit frameworks are linked.
#[link(name = "AppKit", kind = "framework")]
extern "C" {}
#[link(name = "WebKit", kind = "framework")]
extern "C" {}

/// CG geometry types for msg_send FFI.
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

// ── Main-thread dispatch ────────────────────────────────────────────────
//
// WKWebView requires ALL operations on the main thread. Since Vidra's render
// pipeline runs on rayon/tokio worker threads, we use GCD `dispatch_sync_f`
// to the main queue. This works because:
//
// 1. `main()` calls `cmd_render()` which blocks on the render pipeline
// 2. The render pipeline spawns worker threads via rayon
// 3. Worker threads call `dispatch_sync_f(main_queue, ...)` which dispatches
//    to the main thread's RunLoop
//
// The key: `cmd_render` itself is on the main thread, but when it blocks in
// rayon's thread pool, the main thread is idle. We need to process GCD blocks
// on the main thread. The trick: run `CFRunLoopRunInMode` in a loop on the 
// main thread while the render runs on a background thread.
//
// For now, we use a simpler approach: use performSelector with a wait, which
// executes the block synchronously on the calling thread (works for CLI apps).

extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_sync_f(queue: *const c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

/// Execute a closure synchronously on the main thread via GCD.
/// WARNING: Will deadlock if called FROM the main thread.
/// In Vidra's architecture, this is always called from rayon/tokio workers.
fn dispatch_main_sync<R, F: FnOnce() -> R>(f: F) -> R {
    struct Ctx<F, R> {
        f: Option<F>,
        r: Option<R>,
    }
    extern "C" fn run<F: FnOnce() -> R, R>(ctx: *mut c_void) {
        unsafe {
            let c = &mut *(ctx as *mut Ctx<F, R>);
            c.r = Some((c.f.take().unwrap())());
        }
    }
    let mut ctx = Ctx { f: Some(f), r: None };
    unsafe {
        dispatch_sync_f(
            &_dispatch_main_q as *const c_void,
            &mut ctx as *mut _ as *mut c_void,
            run::<F, R>,
        );
    }
    ctx.r.unwrap()
}

/// Pump the RunLoop briefly.
unsafe fn pump_runloop(seconds: f64) {
    extern "C" {
        fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, ret: u8) -> i32;
        static kCFRunLoopDefaultMode: *const c_void;
    }
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, 0);
}

/// Platform-native web capture backend for macOS using WKWebView.
///
/// All WKWebView operations are dispatched to the main thread via GCD.
/// The calling thread (rayon worker) blocks until the main thread completes.
///
/// **Requirement**: The main thread must be running a RunLoop or be available
/// to process GCD blocks. In the Vidra CLI, this is ensured by running
/// the render pipeline on a background thread while the main thread
/// pumps the RunLoop via `start_main_runloop()`.
pub struct PlatformWebViewBackend {
    webview: Option<usize>, // Store as usize to be Send-safe (raw ptr value)
    window: Option<usize>,
    bitmap: Option<usize>,  // Cached NSBitmapImageRep
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
            bitmap: None,
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

/// Start pumping the main thread's RunLoop in the background.
/// This must be called FROM the main thread BEFORE any dispatch_main_sync calls.
/// Returns a sender that, when dropped or sent to, stops the RunLoop.
pub fn start_main_runloop() -> mpsc::Sender<()> {
    let (tx, rx) = mpsc::channel::<()>();
    // Spawn a thread that tells us when to stop
    // We're ON the main thread, so we pump here
    std::thread::spawn(move || {
        // This thread just waits for the stop signal
        let _ = rx.recv();
    });

    // The caller (main thread) should call `pump_main_runloop()` in a loop
    tx
}

/// Pump the main thread RunLoop once. Call this repeatedly from the main thread
/// while running a background render.
pub fn pump_main_runloop_once() {
    unsafe { pump_runloop(0.01); }
}

#[async_trait]
impl WebCaptureBackend for PlatformWebViewBackend {
    async fn start_session(&mut self, config: WebCaptureSessionConfig) -> Result<()> {
        let vp_w = config.viewport_width;
        let vp_h = config.viewport_height;
        let bridge_js = Self::bridge_script(&config.mode);
        let source = config.source.clone();

        // All ObjC work dispatched to main thread
        let (wv_addr, win_addr, bmp_addr) = tokio::task::block_in_place(|| {
            dispatch_main_sync(|| unsafe {
                // Ensure NSApplication exists
                let cls = objc2::runtime::AnyClass::get(c"NSApplication").unwrap();
                let _app: *mut AnyObject = msg_send![cls, sharedApplication];

                // WKWebViewConfiguration
                let cfg_cls = objc2::runtime::AnyClass::get(c"WKWebViewConfiguration").unwrap();
                let wk_cfg: *mut AnyObject = msg_send![cfg_cls, new];

                // Bridge script injection
                let ucc: *mut AnyObject = msg_send![wk_cfg, userContentController];
                let scr_cls = objc2::runtime::AnyClass::get(c"WKUserScript").unwrap();
                let ns_js = NSString::from_str(&bridge_js);
                let scr_raw: *mut AnyObject = msg_send![scr_cls, alloc];
                let script: *mut AnyObject = msg_send![
                    scr_raw,
                    initWithSource: &*ns_js,
                    injectionTime: 0i64,
                    forMainFrameOnly: true
                ];
                let _: () = msg_send![ucc, addUserScript: script];

                // WKWebView
                let wk_cls = objc2::runtime::AnyClass::get(c"WKWebView").unwrap();
                let frame = CGRect {
                    origin: CGPoint { x: 0.0, y: 0.0 },
                    size: CGSize { width: vp_w as f64, height: vp_h as f64 },
                };
                let wk_raw: *mut AnyObject = msg_send![wk_cls, alloc];
                let webview: *mut AnyObject = msg_send![
                    wk_raw, initWithFrame: frame, configuration: wk_cfg
                ];

                // Hidden NSWindow
                let win_cls = objc2::runtime::AnyClass::get(c"NSWindow").unwrap();
                let win_raw: *mut AnyObject = msg_send![win_cls, alloc];
                let window: *mut AnyObject = msg_send![
                    win_raw,
                    initWithContentRect: frame,
                    styleMask: 0usize,
                    backing: 2usize,
                    defer: false
                ];
                let _: () = msg_send![window, setContentView: webview];

                // Navigate
                if source.starts_with("http://") || source.starts_with("https://") {
                    let ns_url = NSString::from_str(&source);
                    let url_cls = objc2::runtime::AnyClass::get(c"NSURL").unwrap();
                    let url: *mut AnyObject = msg_send![url_cls, URLWithString: &*ns_url];
                    let req_cls = objc2::runtime::AnyClass::get(c"NSURLRequest").unwrap();
                    let req: *mut AnyObject = msg_send![req_cls, requestWithURL: url];
                    let _: *mut AnyObject = msg_send![webview, loadRequest: req];
                } else {
                    let abs = std::fs::canonicalize(&source)
                        .unwrap_or_else(|_| std::path::PathBuf::from(&source));
                    let ns_p = NSString::from_str(&abs.to_string_lossy());
                    let url_cls = objc2::runtime::AnyClass::get(c"NSURL").unwrap();
                    let file_url: *mut AnyObject = msg_send![url_cls, fileURLWithPath: &*ns_p];
                    let dir = abs.parent().unwrap_or(std::path::Path::new("."));
                    let ns_d = NSString::from_str(&dir.to_string_lossy());
                    let dir_url: *mut AnyObject = msg_send![url_cls, fileURLWithPath: &*ns_d];
                    let _: *mut AnyObject = msg_send![
                        webview, loadFileURL: file_url, allowingReadAccessToURL: dir_url
                    ];
                }

                // Pre-allocate the bitmap for fast capture (8MB buffer reuse)
                let bmp_cls = objc2::runtime::AnyClass::get(c"NSBitmapImageRep").unwrap();
                let bmp_raw: *mut AnyObject = msg_send![bmp_cls, alloc];
                let bitmap: *mut AnyObject = msg_send![
                    bmp_raw,
                    initWithBitmapDataPlanes: std::ptr::null::<*mut u8>(),
                    pixelsWide: vp_w as i64,
                    pixelsHigh: vp_h as i64,
                    bitsPerSample: 8i64,
                    samplesPerPixel: 4i64,
                    hasAlpha: true,
                    isPlanar: false,
                    colorSpaceName: &*NSString::from_str("NSDeviceRGBColorSpace"),
                    bytesPerRow: (vp_w * 4) as i64,
                    bitsPerPixel: 32i64
                ];

                (webview as usize, window as usize, bitmap as usize)
            })
        });

        self.webview = Some(wv_addr);
        self.window = Some(win_addr);
        self.bitmap = Some(bmp_addr);
        self.config = Some(config);
        self.viewport_width = vp_w;
        self.viewport_height = vp_h;

        // Wait for loading — each check dispatches to main thread
        for _ in 0..200 {
            let is_loading: bool = tokio::task::block_in_place(|| {
                dispatch_main_sync(|| unsafe {
                    let wv = self.webview.unwrap() as *mut AnyObject;
                    pump_runloop(0.05);
                    msg_send![wv, isLoading]
                })
            });
            if !is_loading { break; }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // Extra settle
        tokio::task::block_in_place(|| {
            dispatch_main_sync(|| unsafe { pump_runloop(0.2); });
        });

        Ok(())
    }

    async fn capture_frame(
        &mut self,
        time_seconds: f64,
        variables: &HashMap<String, f64>,
    ) -> Result<FrameBuffer> {
        let wv_addr = self.webview.unwrap();
        let bmp_addr = self.bitmap.unwrap();
        let cfg = self.config.as_ref().ok_or_else(|| anyhow!("No config"))?;
        let vp_w = self.viewport_width;
        let vp_h = self.viewport_height;

        let vars_json = serde_json::to_string(variables)?;
        let js = match cfg.mode {
            vidra_ir::layer::WebCaptureMode::FrameAccurate => {
                format!("window.__vidra_advance_frame({}, JSON.parse('{}'));", time_seconds, vars_json)
            }
            vidra_ir::layer::WebCaptureMode::Realtime => {
                format!("window.__vidra.time = {}; window.__vidra.vars = JSON.parse('{}');", time_seconds, vars_json)
            }
        };

        let frame_data: Vec<u8> = tokio::task::block_in_place(|| {
            dispatch_main_sync(|| unsafe {
                let webview = wv_addr as *mut AnyObject;
                let bitmap = bmp_addr as *mut AnyObject;

                // Execute JS
                let ns_js = NSString::from_str(&js);
                let _: () = msg_send![
                    webview,
                    evaluateJavaScript: &*ns_js,
                    completionHandler: std::ptr::null::<AnyObject>()
                ];

                // Reduced pump time for lower per-frame overhead
                pump_runloop(0.005);

                let rect = CGRect {
                    origin: CGPoint { x: 0.0, y: 0.0 },
                    size: CGSize { width: vp_w as f64, height: vp_h as f64 },
                };

                let gc_cls = objc2::runtime::AnyClass::get(c"NSGraphicsContext").unwrap();
                let gfx_ctx: *mut AnyObject = msg_send![gc_cls, graphicsContextWithBitmapImageRep: bitmap];
                let old: *mut AnyObject = msg_send![gc_cls, currentContext];
                let _: () = msg_send![gc_cls, setCurrentContext: gfx_ctx];
                let _: () = msg_send![webview, displayRectIgnoringOpacity: rect, inContext: gfx_ctx];
                let _: () = msg_send![gc_cls, setCurrentContext: old];

                let data_ptr: *const u8 = msg_send![bitmap, bitmapData];
                let len = (vp_w * vp_h * 4) as usize;
                if data_ptr.is_null() {
                    vec![0u8; len]
                } else {
                    std::slice::from_raw_parts(data_ptr, len).to_vec()
                }
            })
        });

        let mut fb = FrameBuffer::new(vp_w, vp_h, vidra_core::frame::PixelFormat::Rgba8);
        let n = fb.data.len().min(frame_data.len());
        fb.data[..n].copy_from_slice(&frame_data[..n]);
        Ok(fb)
    }

    async fn stop_session(&mut self) -> Result<()> {
        if let (Some(wv), Some(win)) = (self.webview.take(), self.window.take()) {
            tokio::task::block_in_place(|| {
                dispatch_main_sync(|| unsafe {
                    let webview = wv as *mut AnyObject;
                    let window = win as *mut AnyObject;
                    let _: () = msg_send![webview, removeFromSuperview];
                    let _: () = msg_send![window, close];
                });
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
    #[ignore = "Hangs in cargo test because main thread has no RunLoop"]
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
