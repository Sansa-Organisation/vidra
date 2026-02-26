use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use vidra_core::{Color, Duration, types::ShapeType};
use vidra_ir::{
    layer::{Layer, LayerContent, LayerId, WebCaptureMode},
    project::{Project, ProjectSettings},
    scene::{Scene, SceneId},
};
use vidra_render::pipeline::RenderPipeline;

fn create_native_project() -> Project {
    let mut project = Project::new(ProjectSettings::hd_30());
    let mut scene = Scene::new(SceneId::new("bench_scene_native"), Duration::from_seconds(4.0)); // 120 frames

    // Native layer 1: Solid background
    let bg = Layer::new(LayerId::new("bg"), LayerContent::Solid { color: Color::BLACK });
    
    // Native layer 2: Shape
    let shape = Layer::new(LayerId::new("shape"), LayerContent::Shape {
        shape: ShapeType::Rect { corner_radius: 10.0, width: 400.0, height: 400.0 },
        fill: Some(Color::RED),
        stroke: None,
        stroke_width: 0.0,
    }).with_position(960.0, 540.0);

    // Native layer 3: Text
    let text = Layer::new(LayerId::new("title"), LayerContent::Text {
        text: "Benchmarks".to_string(),
        font_family: "Inter".to_string(), // Requires actual font rendering
        font_size: 100.0,
        color: Color::WHITE,
    }).with_position(960.0, 200.0);

    scene.add_layer(bg);
    scene.add_layer(shape);
    scene.add_layer(text);
    project.add_scene(scene);

    project
}

fn create_web_project() -> Project {
    let mut project = create_native_project();
    
    // Fallback to current directory for finding the test HTML file
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let html_path = current_dir.join("crates").join("vidra-web").join("benches").join("bench.html");
    let source = if html_path.exists() {
        format!("file://{}", html_path.display())
    } else {
        format!("file://{}", current_dir.join("benches").join("bench.html").display())
    };

    let web = Layer::new(LayerId::new("web_layer"), LayerContent::Web {
        source,
        viewport_width: 1920,
        viewport_height: 1080,
        mode: WebCaptureMode::FrameAccurate,
        wait_for: None,
        variables: HashMap::new(),
    });

    project.scenes[0].add_layer(web); 
    project
}

fn bench_render_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("vidra_render_pipeline");
    group.sample_size(10); // Rendering 120 frames takes a bit of time

    group.bench_function("native_only_120_frames", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::from_nanos(0);

            for _ in 0..iters {
                let project = create_native_project();
                let mut pipeline = RenderPipeline::new().unwrap();
                let _ = pipeline.load_assets(&project);
                
                let start = std::time::Instant::now();
                for frame_idx in 0..120 {
                    let _frame = pipeline.render_frame_index(&project, frame_idx).unwrap();
                }
                total_duration += start.elapsed();
            }
            
            total_duration
        });
    });

    group.bench_function("native_plus_web_120_frames", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::from_nanos(0);

            for _ in 0..iters {
                let project = create_web_project();
                let mut pipeline = RenderPipeline::new().unwrap();
                let _ = pipeline.load_assets(&project);
                
                let start = std::time::Instant::now();
                for frame_idx in 0..120 {
                    let _frame = pipeline.render_frame_index(&project, frame_idx).unwrap();
                }
                total_duration += start.elapsed();
            }
            
            total_duration
        });
    });

    group.finish();
}

criterion_group!(benches, bench_render_pipeline);
criterion_main!(benches);
