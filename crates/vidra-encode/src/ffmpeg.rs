use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use vidra_core::frame::FrameBuffer;
use vidra_core::VidraError;

/// An audio clip to be conceptually mixed into the final video file.
#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub path: std::path::PathBuf,
    pub trim_start: f64,
    pub trim_end: Option<f64>,
    pub volume: f64,
}

/// Encoder that shells out to FFmpeg for H.264 encoding.
pub struct FfmpegEncoder;

impl FfmpegEncoder {
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

    /// Encode a sequence of RGBA frame buffers to an MP4 file using H.264.
    ///
    /// # Arguments
    /// * `frames` - Ordered sequence of frame buffers (all must have the same dimensions)
    /// * `width` - Frame width in pixels
    /// * `height` - Frame height in pixels
    /// * `fps` - Frames per second
    /// * `output_path` - Path for the output MP4 file
    pub fn encode(
        frames: &[FrameBuffer],
        audio_tracks: &[AudioTrack],
        width: u32,
        height: u32,
        fps: f64,
        output_path: &Path,
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
                let input_idx = i + 1; // 0 is the video stdin stream
                let delay_ms = (track.trim_start * 1000.0).round() as i64;
                let filter_name = format!("[a{}]", i);

                filter_complex.push_str(&format!(
                    "[{}:a]adelay={}|{}:all=1,volume={}{};",
                    input_idx, delay_ms, delay_ms, track.volume, filter_name
                ));
                mix_inputs.push_str(&filter_name);
            }

            filter_complex.push_str(&format!(
                "{}amix=inputs={}:duration=first:dropout_transition=3[aout]",
                mix_inputs, audio_tracks.len()
            ));

            cmd.args(["-filter_complex", &filter_complex]);
            cmd.args(["-map", "0:v", "-map", "[aout]"]);
            // Use AAC codec for audio
            cmd.args(["-c:a", "aac", "-b:a", "192k"]);
        }

        // Shared video encoding configuration
        cmd.args([
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-preset", "medium",
            "-crf", "23",
            "-movflags", "+faststart",
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
                // If write fails, wait for the child process to get its stderr instead of just returning broken pipe.
                let output = child.wait_with_output().unwrap();
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VidraError::Encode(format!(
                    "failed to write frame {} to ffmpeg: {}. FFmpeg stderr: {}", i, e, stderr
                )));
            }
        }

        // Close stdin to signal end of input
        drop(stdin);

        // Wait for FFmpeg to finish
        let output = child
            .wait_with_output()
            .map_err(|e| VidraError::Encode(format!("ffmpeg process error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VidraError::Encode(format!(
                "ffmpeg failed with status {}: {}",
                output.status, stderr
            )));
        }

        tracing::info!(
            "Encoded {} frames to {} ({}x{} @ {}fps)",
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
    fn test_ffmpeg_availability() {
        // This test just checks the availability check doesn't panic.
        // It may return true or false depending on the system.
        let _available = FfmpegEncoder::is_available();
    }

    #[test]
    fn test_encode_empty_frames() {
        let result = FfmpegEncoder::encode(&[], &[], 320, 240, 30.0, Path::new("/tmp/test.mp4"));
        assert!(result.is_err());
    }
}
