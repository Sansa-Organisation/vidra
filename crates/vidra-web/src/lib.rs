pub mod backend;
pub mod playwright;
pub mod platform;
pub mod session;

pub use backend::{WebCaptureBackend, WebCaptureSessionConfig};
pub use playwright::PlaywrightBackend;
pub use session::WebCaptureSession;

/// Create a web capture backend based on preference.
///
/// - `"platform"` — Use OS-native webview (WKWebView / WebView2 / WebKitGTK)
/// - `"playwright"` — Use Playwright Node.js subprocess (legacy)
/// - `"auto"` or `None` — Try platform webview first, fall back to Playwright
pub fn create_backend(preference: Option<&str>) -> Box<dyn WebCaptureBackend> {
    match preference.unwrap_or("auto") {
        "playwright" => Box::new(PlaywrightBackend::new()),
        "platform" | "auto" => {
            let wants_platform = preference.unwrap_or("auto") == "platform";
            
            #[cfg(target_os = "macos")]
            {
                Box::new(platform::PlatformWebViewBackend::new())
            }
            #[cfg(not(target_os = "macos"))]
            {
                if wants_platform {
                    // Forced platform mode, return stub which will error
                    Box::new(platform::PlatformWebViewBackend::new())
                } else {
                    // Auto mode — fallback to playwright until Windows/Linux backends are done
                    tracing::info!("Platform webview not available on this OS, falling back to Playwright");
                    Box::new(PlaywrightBackend::new())
                }
            }
        }
        other => {
            tracing::warn!("Unknown backend '{}', using auto-detection", other);
            Box::new(PlaywrightBackend::new())
        }
    }
}
