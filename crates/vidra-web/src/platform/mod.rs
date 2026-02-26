// Platform-native webview backends for web scene capture.
//
// Each platform uses the OS-provided web engine:
// - macOS: WKWebView (Safari/WebKit)
// - Windows: WebView2 (Edge/Chromium) — behind cfg(target_os = "windows")
// - Linux: WebKitGTK — behind cfg(target_os = "linux")

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

// Re-export the platform-specific backend under a unified name
#[cfg(target_os = "macos")]
pub use macos::PlatformWebViewBackend;

#[cfg(target_os = "windows")]
pub use windows::PlatformWebViewBackend;

#[cfg(target_os = "linux")]
pub use linux::PlatformWebViewBackend;
