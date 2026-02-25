use anyhow::{Context, Result};
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

fn should_skip_dir(name: &str) -> bool {
    matches!(name, ".git" | ".vidra" | "target" | "node_modules" | "dist")
}

fn add_dir_to_zip<W: Write + Seek>(
    zip: &mut zip::ZipWriter<W>,
    base: &Path,
    dir: &Path,
) -> Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("failed to read dir: {}", dir.display()))?
    {
        let path = entry?.path();
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if should_skip_dir(name) {
                continue;
            }
            add_dir_to_zip(zip, base, &path)?;
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let bytes = std::fs::read(&path)
            .with_context(|| format!("failed to read file: {}", path.display()))?;
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(rel, options)
            .with_context(|| format!("failed to start zip entry for {}", path.display()))?;
        zip.write_all(&bytes)
            .with_context(|| format!("failed to write zip entry for {}", path.display()))?;
    }
    Ok(())
}

/// Returns a file path to upload (either the original file, or a created zip for directories)
/// plus whether it is temporary.
pub fn package_for_publish(input: &Path) -> Result<(PathBuf, bool)> {
    if input.is_file() {
        return Ok((input.to_path_buf(), false));
    }
    if !input.is_dir() {
        anyhow::bail!(
            "publish path must be a file or directory: {}",
            input.display()
        );
    }

    let base = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let name = base
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("resource");
    let out =
        std::env::temp_dir().join(format!("vidra_publish_{}_{}.zip", name, std::process::id()));

    let f = std::fs::File::create(&out)
        .with_context(|| format!("failed to create publish zip: {}", out.display()))?;
    let mut zip = zip::ZipWriter::new(f);
    add_dir_to_zip(&mut zip, &base, &base)?;
    zip.finish().context("failed to finalize publish zip")?;

    Ok((out, true))
}

pub fn resource_id_from_sha256_prefixed(sha256_prefixed: &str) -> String {
    let hex = sha256_prefixed.trim_start_matches("sha256:");
    format!("res_{}", &hex[..std::cmp::min(12, hex.len())])
}
