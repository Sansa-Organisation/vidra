use std::path::PathBuf;
use std::time::Instant;
use anyhow::{Context, Result};
use super::parse_and_resolve_imports;
use vidra_ir::Project;

pub fn run_benchmark(file: PathBuf, update_baseline: bool) -> Result<()> {
    println!("⚡ Vidra Performance Benchmark");
    println!("   Source: {}", file.display());

    let source = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read file: {}", file.display()))?;

    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    let is_json = file.extension().map_or(false, |e| e == "json");

    let base_project = if is_json {
        serde_json::from_str::<Project>(&source)
            .with_context(|| format!("failed to parse IR JSON: {}", file.display()))?
    } else {
        let ast = parse_and_resolve_imports(&file)?;
        let checker = vidra_lang::TypeChecker::new(file_name.clone());
        let _ = checker.check(&ast);
        vidra_lang::Compiler::compile(&ast).map_err(|e| anyhow::anyhow!("{}", e))?
    };

    // We will benchmark 3 target resolutions at 60fps (if original allows varying, we override just dimensions):
    // 720p (1280x720)
    // 1080p (1920x1080)
    // 4K (3840x2160)

    let targets = vec![
        ("720p", 1280, 720),
        ("1080p", 1920, 1080),
        ("4K", 3840, 2160),
    ];

    println!("\n▶ Running benchmark suite across {} profiles...\n", targets.len());

    let mut results = Vec::new();

    for (name, w, h) in targets {
        let mut proj = base_project.clone();
        proj.settings.width = w;
        proj.settings.height = h;

        print!("   Measuring {} ({}x{}) ... ", name, w, h);
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        
        let _target_frames = proj.total_frames();
        
        // Render 
        let render_start = Instant::now();
        let render_result = vidra_render::RenderPipeline::render(&proj);
        let render_time = render_start.elapsed();

        if let Ok(res) = render_result {
            let fps = res.frame_count as f64 / render_time.as_secs_f64();
            println!("{:.1}ms ({:.0} fps)", render_time.as_secs_f64() * 1000.0, fps);
            
            results.push(BenchResult {
                profile: name.to_string(),
                width: w,
                height: h,
                frames: res.frame_count,
                duration_ms: render_time.as_secs_f64() * 1000.0,
                fps,
            });
        } else {
            println!("ERROR");
        }
    }

    println!("\n📊 Benchmark Report:");
    println!("{:<10} | {:<10} | {:<10} | {:<10} | {}", "Profile", "Resolution", "Render (ms)", "FPS", "Regression");
    println!("{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}", "", "", "", "", "");
    
    let baseline_path = std::path::Path::new("tests/snapshots/benchmarks.json");
    let mut baselines: std::collections::HashMap<String, BenchResult> = std::collections::HashMap::new();
    
    if baseline_path.exists() {
        if let Ok(b) = std::fs::read_to_string(baseline_path) {
            if let Ok(parsed) = serde_json::from_str::<Vec<BenchResult>>(&b) {
                for res in parsed {
                    baselines.insert(format!("{}@{}x{}", res.profile, res.width, res.height), res);
                }
            }
        }
    }

    let mut failed = false;

    for res in &results {
        let key = format!("{}@{}x{}", res.profile, res.width, res.height);
        let mut reg_str = String::from("-");
        
        if let Some(baseline) = baselines.get(&key) {
            let diff = res.duration_ms - baseline.duration_ms;
            let percent = (diff / baseline.duration_ms) * 100.0;
            
            if percent > 5.0 {
                reg_str = format!("❌ +{:.1}%", percent);
                failed = true;
            } else if percent < -5.0 {
                reg_str = format!("✅ {:.1}%", percent);
            } else {
                reg_str = format!("➖ {:.1}%", percent);
            }
        }

        println!("{:<10} | {:>4}x{:<4}  | {:>10.1} | {:>10.0} | {}", 
            res.profile, res.width, res.height, res.duration_ms, res.fps, reg_str);
    }
    
    println!();
    
    if update_baseline {
        if let Some(parent) = baseline_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(baseline_path, json)?;
        println!("📸 Updated baseline at {}", baseline_path.display());
    } else if failed {
        anyhow::bail!("Performance regression detected. See report above.");
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BenchResult {
    profile: String,
    width: u32,
    height: u32,
    frames: u64,
    duration_ms: f64,
    fps: f64,
}
