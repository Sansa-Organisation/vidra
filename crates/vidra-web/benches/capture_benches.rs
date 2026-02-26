use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use vidra_ir::layer::WebCaptureMode;
use vidra_web::{create_backend, WebCaptureSessionConfig};

fn setup_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vidra_web_capture");
    group.sample_size(10); // Capture is slow, so 10 samples is enough
    
    // Fallback to current directory for finding the test HTML file
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let html_path = current_dir.join("crates").join("vidra-web").join("benches").join("bench.html");
    let source = if html_path.exists() {
        format!("file://{}", html_path.display())
    } else {
        // Fallback for isolated target modes
        format!("file://{}", current_dir.join("benches").join("bench.html").display())
    };

    let rt = Runtime::new().unwrap();

    group.bench_function("frame_accurate_10_frames_native", |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let source_clone = source.clone();
            async move {
                let mut total_duration = std::time::Duration::from_nanos(0);

                for _ in 0..iters {
                    // Using platform backend for fastest native captures (WKWebView on macOS)
                    let mut backend = create_backend(Some("platform"));
                    
                    let config = WebCaptureSessionConfig {
                        source: source_clone.clone(),
                        viewport_width: 1920,
                        viewport_height: 1080,
                        mode: WebCaptureMode::FrameAccurate,
                        wait_for: None,
                        fps: 30.0,
                        format: vidra_core::frame::PixelFormat::Rgba8,
                    };
                    
                    backend.start_session(config).await.unwrap();
                    
                    // We only want to measure the capture phase
                    let vars = HashMap::new();
                    let start = std::time::Instant::now();
                    for i in 0..10 {
                        let time = (i as f64) / 30.0;
                        let _frame = backend.capture_frame(time, &vars).await.unwrap();
                    }
                    total_duration += start.elapsed();
                    
                    let _ = backend.stop_session().await;
                }
                
                total_duration
            }
        });
    });

    group.bench_function("realtime_120_frames_native", |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let source_clone = source.clone();
            async move {
                let mut total_duration = std::time::Duration::from_nanos(0);

                for _ in 0..iters {
                    // Using platform backend
                    let mut backend = create_backend(Some("platform"));
                    
                    let config = WebCaptureSessionConfig {
                        source: source_clone.clone(),
                        viewport_width: 1920,
                        viewport_height: 1080,
                        mode: WebCaptureMode::Realtime,
                        wait_for: None,
                        fps: 30.0,
                        format: vidra_core::frame::PixelFormat::Rgba8,
                    };
                    
                    backend.start_session(config).await.unwrap();
                    
                    // We measure the capture phase for 120 frames
                    let vars = HashMap::new();
                    let start = std::time::Instant::now();
                    for i in 0..120 {
                        let time = (i as f64) / 30.0;
                        let _frame = backend.capture_frame(time, &vars).await.unwrap();
                    }
                    total_duration += start.elapsed();
                    
                    let _ = backend.stop_session().await;
                }
                
                total_duration
            }
        });
    });

    group.finish();
}

criterion_group!(benches, setup_benchmark);
criterion_main!(benches);
