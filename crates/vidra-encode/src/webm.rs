use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use vidra_core::frame::FrameBuffer;
use vidra_core::VidraError;

use crate::ffmpeg::AudioTrack;

/// Encoder that shells out to FFmpeg for VP9 WebM encoding.
/// Ideal for web-native video that doesn't require H.264 licensing.
pub struct WebmEncoder;

impl WebmEncoder {
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

    /// Encode a sequence of RGBA frame buffers to a WebM file using VP9.
    ///
    /// # Arguments
    /// * `frames` - Ordered sequence of frame buffers (all must have the same dimensions)
    /// * `audio_tracks` - Optional audio tracks to mix in
    /// * `width` - Frame width in pixels
    /// * `height` - Frame height in pixels
    /// * `fps` - Frames per second
    /// * `output_path` - Path for the output .webm file
    /// * `crf` - Constant Rate Factor for VP9 (0-63, lower = better quality, default 31)
    pub fn encode(
        frames: &[FrameBuffer],
        audio_tracks: &[AudioTrack],
        width: u32,
        height: u32,
        fps: f64,
        output_path: &Path,
        crf: Option<u32>,
    ) -> Result<(), VidraError> {
        if frames.is_empty() {
            return Err(VidraError::Encode("no frames to encode".into()));
        }

        if !Self::is_available() {
            return Err(VidraError::Encode(
                "ffmpeg not found in PATH. Install FFmpeg: https://ffmpeg.org/download.html".into(),
            ));
        }

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let fps_str = format!("{}", fps);
        let crf_val = crf.unwrap_or(31).min(63);

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y"); // Overwrite output

        // Input 0: raw video frames from stdin
        cmd.args([
            "-f", "rawvideo",
            "-pixel_format", "rgba",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &fps_str,
            "-i", "-",
        ]);

        // Inputs 1..N: audio files
        for track in audio_tracks {
            cmd.arg("-i").arg(&track.path);
        }

        // Configure audio mixing if any tracks are provided
        if !audio_tracks.is_empty() {
            let mut filter_complex = String::new();
            let mut mix_inputs = String::new();

            for (i, track) in audio_tracks.iter().enumerate() {
                let input_idx = i + 1;
                let delay_ms = (track.trim_start * 1000.0).round() as i64;
                let filter_name = format!("[a{}]", i);

                filter_complex.push_str(&format!(
                    "[{}:a]adelay={}:all=1,volume={}{};",
                    input_idx, delay_ms, track.volume, filter_name
                ));
                mix_inputs.push_str(&filter_name);
            }

            filter_complex.push_str(&format!(
                "{}amix=inputs={}:duration=first:dropout_transition=3[aout]",
                mix_inputs, audio_tracks.len()
            ));

            cmd.args(["-filter_complex", &filter_complex]);
            cmd.args(["-map", "0:v", "-map", "[aout]"]);
            // Use Opus for WebM audio (best practice)
            cmd.args(["-c:a", "libopus", "-b:a", "128k"]);
        }

        // VP9 video encoding settings
        cmd.args([
            "-c:v", "libvpx-vp9",
            "-pix_fmt", "yuva420p",      // Support alpha channel
            "-crf", &crf_val.to_string(),
            "-b:v", "0",                  // Constant quality mode
            "-row-mt", "1",              // Row-based multithreading
        ]);

        cmd.arg(output_path);

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| VidraError::Encode(format!("failed to start ffmpeg: {}", e)))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| VidraError::Encode("failed to open ffmpeg stdin".into()))?;

        // Write raw RGBA frame data to FFmpeg's stdin
        for (i, frame) in frames.iter().enumerate() {
            if frame.width != width || frame.height != height {
                return Err(VidraError::Encode(format!(
                    "frame {} has dimensions {}x{}, expected {}x{}",
                    i, frame.width, frame.height, width, height
                )));
            }
            if let Err(e) = stdin.write_all(&frame.data) {
                let output = child.wait_with_output().unwrap();
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VidraError::Encode(format!(
                    "failed to write frame {} to ffmpeg: {}. FFmpeg stderr: {}", i, e, stderr
                )));
            }
        }

        drop(stdin);

        let output = child
            .wait_with_output()
            .map_err(|e| VidraError::Encode(format!("ffmpeg process error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VidraError::Encode(format!(
                "ffmpeg (VP9) failed with status {}: {}",
                output.status, stderr
            )));
        }

        tracing::info!(
            "Encoded {} frames to WebM (VP9) at {} ({}x{} @ {}fps)",
            frames.len(),
            output_path.display(),
            width,
            height,
            fps
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webm_encode_empty_frames() {
        let result = WebmEncoder::encode(&[], &[], 320, 240, 30.0, Path::new("/tmp/test.webm"), None);
        assert!(result.is_err());
    }
}
