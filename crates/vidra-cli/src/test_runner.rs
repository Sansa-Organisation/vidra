use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use image::{RgbaImage, ImageBuffer, Rgba};
use vidra_core::frame::FrameBuffer;

use crate::parse_and_resolve_imports;

/// Diff tolerance out of 255 per channel
const TOLERANCE: i16 = 2; 
/// Max allowed percentage of strictly different pixels
const MAX_ERROR_PIXELS_PERCENT: f64 = 0.05;

pub fn run_test(file: PathBuf, update: bool) -> Result<()> {
    let start = Instant::now();
    println!("🧪 Vidra Visual Regression Test");
    println!("   Source: {}", file.display());

    // 1. Parse & Check
    let ast = parse_and_resolve_imports(&file)?;
    let file_name = file.file_name().unwrap().to_string_lossy().to_string();
    let checker = vidra_lang::TypeChecker::new(file_name.clone());
    if let Err(diags) = checker.check(&ast) {
        let msgs: Vec<String> = diags.into_iter()
            .filter(|d| d.severity == vidra_lang::checker::DiagnosticSeverity::Error)
            .map(|e| e.to_string()).collect();
        anyhow::bail!("Type checking failed:\n  {}", msgs.join("\n  "));
    }

    // 2. Compile to IR
    let project = vidra_lang::Compiler::compile(&ast)
        .map_err(|e| anyhow::anyhow!("Compile error: {}", e))?;
        
    vidra_ir::validate::validate_project(&project).map_err(|errors| {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("IR validation errors:\n  {}", msgs.join("\n  "))
    })?;

    // 3. Setup Render Pipeline
    let mut pipeline = vidra_render::RenderPipeline::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    pipeline.load_assets(&project).map_err(|e| anyhow::anyhow!("{}", e))?;

    // We'll save snapshots to `./tests/snapshots/`
    let base_name = file.file_stem().unwrap().to_string_lossy().to_string();
    let snapshots_dir = Path::new("tests").join("snapshots");
    if update {
        std::fs::create_dir_all(&snapshots_dir)?;
    }

    let mut all_passed = true;
    let mut global_frame_start = 0;
    let mut test_results = Vec::new();

    for scene in &project.scenes {
        println!("\n▶ Testing scene: '{}'", scene.id);
        
        // Take a snapshot of the exact middle frame of the scene
        let scene_frames = scene.frame_count(project.settings.fps);
        let mid_local_frame = scene_frames / 2;
        let global_frame = global_frame_start + mid_local_frame;

        let frame = pipeline.render_frame_index(&project, global_frame)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let snapshot_name = format!("{}_{}.png", base_name, scene.id);
        let snapshot_path = snapshots_dir.join(&snapshot_name);
        
        // Setup diff images
        let reference_path = format!("{}_{}.png", base_name, scene.id);
        let failed_path = format!("{}_{}_failed.png", base_name, scene.id);
        let diff_path = format!("{}_{}_diff.png", base_name, scene.id);

        if update {
            save_frame_as_png(&frame, &snapshot_path)?;
            println!("   📸 Updated snapshot: {}", snapshot_path.display());
            test_results.push(HtmlTestResult { scene_id: scene.id.to_string(), status: "UPDATED", error_msg: String::new(), reference_img: reference_path, failed_img: String::new(), diff_img: String::new() });
        } else {
            if !snapshot_path.exists() {
                println!("   ❌ Missing snapshot: {}", snapshot_path.display());
                println!("      Run with --update to generate it.");
                all_passed = false;
                test_results.push(HtmlTestResult { scene_id: scene.id.to_string(), status: "MISSING", error_msg: "Run with --update to generate".to_string(), reference_img: String::new(), failed_img: String::new(), diff_img: String::new() });
                global_frame_start += scene_frames;
                continue;
            }

            let reference = image::open(&snapshot_path)
                .with_context(|| format!("Failed to load snapshot {}", snapshot_path.display()))?
                .into_rgba8();

            match compare_frames(&frame, &reference) {
                Err(e) => {
                    println!("   ❌ Failed: {}", e);
                    
                    let failed_absolute = snapshots_dir.join(&failed_path);
                    save_frame_as_png(&frame, &failed_absolute)?;
                    
                    let diff_absolute = snapshots_dir.join(&diff_path);
                    let diff_img = generate_diff_image(&frame, &reference);
                    diff_img.save(&diff_absolute)?;

                    println!("      Rendered output saved to: {}", failed_absolute.display());
                    
                    all_passed = false;
                    test_results.push(HtmlTestResult { scene_id: scene.id.to_string(), status: "FAILED", error_msg: e.to_string(), reference_img: reference_path, failed_img: failed_path.clone(), diff_img: diff_path.clone() });
                }
                Ok(_) => {
                    println!("   ✅ Passed pixel diff");
                    test_results.push(HtmlTestResult { scene_id: scene.id.to_string(), status: "PASSED", error_msg: String::new(), reference_img: reference_path, failed_img: String::new(), diff_img: String::new() });
                }
            }
        }

        global_frame_start += scene_frames;
    }

    generate_html_report(&base_name, &snapshots_dir, &test_results)?;

    let elapsed = start.elapsed();
    println!("\n✨ Test runner finished in {:.2}s", elapsed.as_secs_f64());

    if !all_passed {
        anyhow::bail!("Some visual tests failed.");
    }

    Ok(())
}

fn save_frame_as_png(frame: &FrameBuffer, path: &Path) -> Result<()> {
    if frame.format != vidra_core::frame::PixelFormat::Rgba8 {
        anyhow::bail!("Unexpected pixel format in frame buffer");
    }

    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.data.clone())
        .ok_or_else(|| anyhow::anyhow!("Failed to convert frame to image buffer (size mismatch)"))?;
        
    img.save(path)?;
    Ok(())
}

fn compare_frames(frame: &FrameBuffer, reference: &RgbaImage) -> Result<()> {
    if frame.width != reference.width() || frame.height != reference.height() {
        anyhow::bail!("Dimensions mismatch: {}x{} vs reference {}x{}", frame.width, frame.height, reference.width(), reference.height());
    }

    let mut error_pixels = 0;
    let total_pixels = frame.width * frame.height;
    
    let frame_data = &frame.data;
    let ref_data = reference.as_raw();

    for i in (0..frame_data.len()).step_by(4) {
        let r1 = frame_data[i] as i16;
        let g1 = frame_data[i+1] as i16;
        let b1 = frame_data[i+2] as i16;
        let a1 = frame_data[i+3] as i16;

        let r2 = ref_data[i] as i16;
        let g2 = ref_data[i+1] as i16;
        let b2 = ref_data[i+2] as i16;
        let a2 = ref_data[i+3] as i16;

        let diff_r = (r1 - r2).abs();
        let diff_g = (g1 - g2).abs();
        let diff_b = (b1 - b2).abs();
        let diff_a = (a1 - a2).abs();


        if diff_r > TOLERANCE || diff_g > TOLERANCE || diff_b > TOLERANCE || diff_a > TOLERANCE {
            error_pixels += 1;
        }
    }

    let error_percent = (error_pixels as f64 / total_pixels as f64) * 100.0;

    if error_percent > MAX_ERROR_PIXELS_PERCENT {
        anyhow::bail!("Exceeded error tolerance: {:.2}% differing pixels (max {:.2}%)", error_percent, MAX_ERROR_PIXELS_PERCENT);
    }

    Ok(())
}

fn generate_diff_image(frame: &FrameBuffer, reference: &RgbaImage) -> RgbaImage {
    let mut diff_img = ImageBuffer::new(frame.width, frame.height);
    let frame_data = &frame.data;
    let ref_data = reference.as_raw();

    for y in 0..frame.height {
        for x in 0..frame.width {
            let i = ((y * frame.width + x) * 4) as usize;
            
            let r1 = frame_data[i] as i16;
            let g1 = frame_data[i+1] as i16;
            let b1 = frame_data[i+2] as i16;
            
            let r2 = ref_data[i] as i16;
            let g2 = ref_data[i+1] as i16;
            let b2 = ref_data[i+2] as i16;
            
            let diff_r = (r1 - r2).abs();
            let diff_g = (g1 - g2).abs();
            let diff_b = (b1 - b2).abs();
            
            if diff_r > TOLERANCE || diff_g > TOLERANCE || diff_b > TOLERANCE {
                diff_img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                let avg = ((r1 + g1 + b1) / 3) as u8;
                diff_img.put_pixel(x, y, Rgba([avg, avg, avg, 128]));
            }
        }
    }
    
    diff_img
}

struct HtmlTestResult {
    scene_id: String,
    status: &'static str,
    error_msg: String,
    reference_img: String,
    failed_img: String,
    diff_img: String,
}

fn generate_html_report(base_name: &str, out_dir: &Path, results: &[HtmlTestResult]) -> Result<()> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><title>Vidra Test Report</title><style>");
    html.push_str("body { font-family: system-ui, sans-serif; background: #fafafa; color: #333; margin: 0; padding: 2rem; }");
    html.push_str("h1 { margin-top: 0; }");
    html.push_str(".card { background: white; border-radius: 8px; padding: 1.5rem; margin-bottom: 2rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }");
    html.push_str(".status-PASS, .status-PASSED, .status-UPDATED { color: #10B981; font-weight: bold; }");
    html.push_str(".status-FAIL, .status-FAILED, .status-MISSING { color: #EF4444; font-weight: bold; }");
    html.push_str(".img-flex { display: flex; gap: 1rem; overflow-x: auto; padding-top: 1rem; }");
    html.push_str(".img-col { display: flex; flex-direction: column; flex: 1; min-width: 300px; }");
    html.push_str("img { max-width: 100%; border: 1px solid #ddd; background: #eee; }");
    html.push_str("</style></head><body>");
    
    html.push_str(&format!("<h1>Test Report: {}</h1>", base_name));

    for res in results {
        html.push_str("<div class='card'>");
        html.push_str(&format!("<h2>Scene: <code>{}</code></h2>", res.scene_id));
        html.push_str(&format!("<p>Status: <span class='status-{}'>{}</span></p>", res.status, res.status));
        
        if !res.error_msg.is_empty() {
            html.push_str(&format!("<p><strong>Error:</strong> {}</p>", res.error_msg));
        }

        if res.status == "FAILED" {
            html.push_str("<div class='img-flex'>");
            
            html.push_str("<div class='img-col'><strong>Reference</strong>");
            html.push_str(&format!("<img src='{}' ></div>", res.reference_img));
            
            html.push_str("<div class='img-col'><strong>Rendered</strong>");
            html.push_str(&format!("<img src='{}' ></div>", res.failed_img));
            
            html.push_str("<div class='img-col'><strong>Diff</strong>");
            html.push_str(&format!("<img src='{}' ></div>", res.diff_img));
            
            html.push_str("</div>");
        } else if res.status == "PASSED" || res.status == "UPDATED" {
            html.push_str("<div class='img-flex'>");
            html.push_str("<div class='img-col'><strong>Reference</strong>");
            html.push_str(&format!("<img src='{}' ></div>", res.reference_img));
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }

    html.push_str("</body></html>");

    let report_path = out_dir.join(format!("{}_report.html", base_name));
    std::fs::write(&report_path, html)?;
    println!("   📄 HTML diff report generated: {}", report_path.display());
    Ok(())
}
