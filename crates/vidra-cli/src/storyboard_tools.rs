use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use sha2::{Digest, Sha256};
use std::path::Path;

fn colors_from_prompt(prompt: &str) -> [Rgba<u8>; 6] {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    let digest = hasher.finalize();

    let mut out = [Rgba([0, 0, 0, 255]); 6];
    for i in 0..6 {
        let base = (i * 5) % digest.len();
        let r = digest[base];
        let g = digest[(base + 1) % digest.len()];
        let b = digest[(base + 2) % digest.len()];
        out[i] = Rgba([r, g, b, 255]);
    }
    out
}

pub fn generate_storyboard_png(prompt: &str, out_path: &Path) -> Result<()> {
    // 3x2 grid of 512px tiles -> 1536x1024
    let tile_w = 512u32;
    let tile_h = 512u32;
    let cols = 3u32;
    let rows = 2u32;

    let width = tile_w * cols;
    let height = tile_h * rows;

    let colors = colors_from_prompt(prompt);
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // Fill tiles with solid colors; draw thin white grid lines.
    for y in 0..height {
        for x in 0..width {
            let col = x / tile_w;
            let row = y / tile_h;
            let idx = (row * cols + col) as usize;
            let mut px = colors[idx];

            // grid line
            if x % tile_w == 0 || y % tile_h == 0 {
                px = Rgba([245, 245, 245, 255]);
            }
            img.put_pixel(x, y, px);
        }
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create storyboard output dir: {}", parent.display()))?;
    }

    img.save(out_path)
        .with_context(|| format!("failed to write storyboard: {}", out_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storyboard_png_written() {
        let root = std::env::temp_dir().join(format!("vidra_storyboard_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let out = root.join("storyboard.png");
        generate_storyboard_png("hello world", &out).unwrap();
        assert!(out.exists());

        let img = image::open(&out).unwrap();
        assert_eq!(img.width(), 1536);
        assert_eq!(img.height(), 1024);

        let _ = std::fs::remove_dir_all(&root);
    }
}
