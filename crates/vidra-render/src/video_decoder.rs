//! Video decoding module.
//! Uses FFmpeg subprocess to extract individual frames from video files.
//! Phase 0: single-threaded, frame-at-a-time extraction.
//! Phase 1: streaming decode, seek caching, GPU-accelerated decode.

use dashmap::DashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use vidra_core::frame::FrameBuffer;
use vidra_core::{Color, PixelFormat, VidraError};

/// Metadata about a video file.
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Frame rate (fps).
    pub fps: f64,
    /// Total number of frames.
    pub frame_count: u64,
}

/// A video decoder backed by FFmpeg.
/// Extracts frames from video files by shelling out to `ffmpeg`.
pub struct VideoDecoder {
    /// Cache of decoded frames keyed by (path, frame_index).
    frame_cache: DashMap<(String, u64), FrameBuffer>,
    /// Cache of probed video info keyed by path.
    info_cache: DashMap<String, VideoInfo>,
}

impl VideoDecoder {
    pub fn new() -> Self {
        Self {
            frame_cache: DashMap::new(),
            info_cache: DashMap::new(),
        }
    }

    /// Check if FFmpeg is available on the system.
    pub fn is_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Probe a video file to get its metadata (width, height, duration, fps).
    pub fn probe(&self, path: &Path) -> Result<VideoInfo, VidraError> {
        let key = path.to_string_lossy().to_string();
        if let Some(info) = self.info_cache.get(&key) {
            return Ok(info.clone());
        }

        if !Self::is_available() {
            return Err(VidraError::Encode(
                "ffmpeg/ffprobe not found in PATH. Install FFmpeg: https://ffmpeg.org/download.html"
                    .into(),
            ));
        }

        if !path.exists() {
            return Err(VidraError::asset(
                format!("video file not found: {}", path.display()),
                path,
            ));
        }

        // Use ffprobe to get video info
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_streams",
                "-show_format",
            ])
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| VidraError::Encode(format!("failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VidraError::Encode(format!("ffprobe failed: {}", stderr)));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| VidraError::Encode(format!("failed to parse ffprobe output: {}", e)))?;

        // Find the video stream
        let streams = json["streams"]
            .as_array()
            .ok_or_else(|| VidraError::Encode("no streams found in video".into()))?;

        let video_stream = streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .ok_or_else(|| VidraError::Encode("no video stream found".into()))?;

        let width = video_stream["width"]
            .as_u64()
            .ok_or_else(|| VidraError::Encode("missing width in video stream".into()))?
            as u32;
        let height = video_stream["height"]
            .as_u64()
            .ok_or_else(|| VidraError::Encode("missing height in video stream".into()))?
            as u32;

        // Parse frame rate from r_frame_rate (e.g., "30/1")
        let fps = parse_frame_rate(video_stream["r_frame_rate"].as_str().unwrap_or("30/1"));

        // Parse duration from format or stream
        let duration_secs = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| {
                video_stream["duration"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        let frame_count = if duration_secs > 0.0 {
            (duration_secs * fps).round() as u64
        } else {
            0
        };

        let info = VideoInfo {
            width,
            height,
            duration_secs,
            fps,
            frame_count,
        };

        self.info_cache.insert(key, info.clone());
        Ok(info)
    }

    /// Extract a single frame from a video file at a given timestamp.
    ///
    /// Returns an RGBA FrameBuffer of the video's native resolution.
    pub fn extract_frame(
        &self,
        path: &Path,
        timestamp_secs: f64,
        target_width: u32,
        target_height: u32,
    ) -> Result<FrameBuffer, VidraError> {
        let cache_key = (
            path.to_string_lossy().to_string(),
            (timestamp_secs * 1000.0) as u64, // millisecond precision for cache key
        );

        if let Some(cached) = self.frame_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        if !Self::is_available() {
            return Err(VidraError::Encode("ffmpeg not found in PATH".into()));
        }

        if !path.exists() {
            return Err(VidraError::asset(
                format!("video file not found: {}", path.display()),
                path,
            ));
        }

        let ts_str = format!("{:.3}", timestamp_secs);

        // Use FFmpeg to extract a single frame at the given timestamp,
        // outputting raw RGBA pixels to stdout.
        let output = Command::new("ffmpeg")
            .args([
                "-ss", &ts_str, // Seek position (before -i for fast seeking)
                "-i",
            ])
            .arg(path)
            .args([
                "-vframes",
                "1", // Extract single frame
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "-s",
                &format!("{}x{}", target_width, target_height),
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| VidraError::Encode(format!("failed to extract video frame: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("FFmpeg frame extraction warning: {}", stderr);
            // Return a fallback frame instead of failing hard
            return Ok(FrameBuffer::solid(
                target_width,
                target_height,
                &Color::rgba(0.2, 0.2, 0.2, 1.0),
            ));
        }

        let expected_size = (target_width as usize) * (target_height as usize) * 4;
        if output.stdout.len() < expected_size {
            tracing::warn!(
                "FFmpeg output size mismatch: expected {} bytes, got {}",
                expected_size,
                output.stdout.len()
            );
            return Ok(FrameBuffer::solid(
                target_width,
                target_height,
                &Color::rgba(0.2, 0.2, 0.2, 1.0),
            ));
        }

        let mut fb = FrameBuffer::new(target_width, target_height, PixelFormat::Rgba8);
        fb.data = output.stdout[..expected_size].to_vec();

        self.frame_cache.insert(cache_key, fb.clone());
        Ok(fb)
    }

    /// Clear the frame cache to free memory.
    pub fn clear_cache(&self) {
        self.frame_cache.clear();
        self.info_cache.clear();
    }

    /// Get the number of cached frames.
    pub fn cache_size(&self) -> usize {
        self.frame_cache.len()
    }
}

impl Default for VideoDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a frame rate string like "30/1" or "24000/1001" into a float.
fn parse_frame_rate(rate_str: &str) -> f64 {
    if let Some((num_str, den_str)) = rate_str.split_once('/') {
        let num: f64 = num_str.parse().unwrap_or(30.0);
        let den: f64 = den_str.parse().unwrap_or(1.0);
        if den > 0.0 {
            num / den
        } else {
            30.0
        }
    } else {
        rate_str.parse::<f64>().unwrap_or(30.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame_rate_fraction() {
        assert!((parse_frame_rate("30/1") - 30.0).abs() < 0.001);
        assert!((parse_frame_rate("24000/1001") - 23.976).abs() < 0.01);
        assert!((parse_frame_rate("60/1") - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_frame_rate_plain() {
        assert!((parse_frame_rate("25") - 25.0).abs() < 0.001);
        assert!((parse_frame_rate("29.97") - 29.97).abs() < 0.01);
    }

    #[test]
    fn test_parse_frame_rate_invalid() {
        // Should fall back to 30.0
        assert!((parse_frame_rate("invalid") - 30.0).abs() < 0.001);
        assert!((parse_frame_rate("30/0") - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_video_decoder_new() {
        let decoder = VideoDecoder::new();
        assert_eq!(decoder.cache_size(), 0);
    }

    #[test]
    fn test_video_decoder_probe_missing_file() {
        let decoder = VideoDecoder::new();
        let result = decoder.probe(Path::new("/nonexistent/video.mp4"));
        assert!(result.is_err());
    }

    #[test]
    fn test_video_decoder_extract_missing_file() {
        let decoder = VideoDecoder::new();
        let result = decoder.extract_frame(Path::new("/nonexistent/video.mp4"), 0.0, 320, 240);
        assert!(result.is_err());
    }

    #[test]
    fn test_video_decoder_clear_cache() {
        let decoder = VideoDecoder::new();
        decoder.clear_cache();
        assert_eq!(decoder.cache_size(), 0);
    }
}
