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
    pub role: Option<String>,
    pub duck: Option<f64>,
}

/// Detected hardware encoder with its required arguments.
#[derive(Debug, Clone)]
enum HwEncoder {
    /// Apple VideoToolbox (macOS)
    VideoToolbox,
    /// NVIDIA NVENC
    #[allow(dead_code)]
    Nvenc,
    /// Software libx264 fallback
    Libx264,
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

    /// Detect the best available H.264 encoder.
    /// Tries hardware encoders first, falls back to libx264.
    fn detect_best_encoder() -> HwEncoder {
        let output = Command::new("ffmpeg")
            .args(["-encoders", "-hide_banner"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        if let Ok(out) = output {
            let encoders = String::from_utf8_lossy(&out.stdout);
            // Prefer VideoToolbox on macOS
            if encoders.contains("h264_videotoolbox") {
                tracing::info!("Using VideoToolbox hardware encoder");
                return HwEncoder::VideoToolbox;
            }
            // NVIDIA NVENC
            if encoders.contains("h264_nvenc") {
                tracing::info!("Using NVENC hardware encoder");
                return HwEncoder::Nvenc;
            }
        }

        tracing::info!("Using libx264 software encoder");
        HwEncoder::Libx264
    }

    /// Apply encoder-specific arguments to the FFmpeg command.
    fn apply_encoder_args(cmd: &mut Command, encoder: &HwEncoder) {
        match encoder {
            HwEncoder::VideoToolbox => {
                cmd.args([
                    "-c:v",
                    "h264_videotoolbox",
                    "-pix_fmt",
                    "yuv420p",
                    // quality: 1-100, lower = better. 55 ≈ CRF 23 quality.
                    "-q:v",
                    "55",
                    "-profile:v",
                    "high",
                    "-allow_sw",
                    "1", // fallback to software if HW session limit hit
                    "-movflags",
                    "+faststart",
                ]);
            }
            HwEncoder::Nvenc => {
                cmd.args([
                    "-c:v",
                    "h264_nvenc",
                    "-pix_fmt",
                    "yuv420p",
                    "-cq",
                    "23",
                    "-preset",
                    "p4", // balanced speed/quality
                    "-movflags",
                    "+faststart",
                ]);
            }
            HwEncoder::Libx264 => {
                cmd.args([
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                    "-preset",
                    "fast", // was "medium"; fast is ~2x quicker, minimal quality loss
                    "-crf",
                    "23",
                    "-threads",
                    "0", // use all available CPU threads
                    "-movflags",
                    "+faststart",
                ]);
            }
        }
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
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &fps_str,
            "-i",
            "-",
        ]);

        // Inputs 1..N: audio files
        for track in audio_tracks {
            cmd.arg("-i").arg(&track.path);
        }

        // Configure audio mixing if any tracks are provided
        if !audio_tracks.is_empty() {
            let mut filter_complex = String::new();
            let mut mix_inputs = String::new();
            let mut narr_inputs = Vec::new();
            let mut music_inputs = Vec::new();
            let mut music_duck = 1.0_f64;

            for (i, track) in audio_tracks.iter().enumerate() {
                let input_idx = i + 1; // 0 is the video stdin stream
                let delay_ms = (track.trim_start * 1000.0).round() as i64;
                let filter_name = format!("[a{}]", i);

                filter_complex.push_str(&format!(
                    "[{}:a]adelay={}:all=1,volume={}{};",
                    input_idx, delay_ms, track.volume, filter_name
                ));
                mix_inputs.push_str(&filter_name);

                let role = track.role.as_deref().unwrap_or("narration");
                if role == "music" || track.duck.is_some() {
                    music_inputs.push(filter_name);
                    if let Some(d) = track.duck {
                        music_duck = music_duck.min(d.max(0.0).min(1.0));
                    }
                } else {
                    narr_inputs.push(filter_name);
                }
            }

            let should_duck =
                !music_inputs.is_empty() && !narr_inputs.is_empty() && music_duck < 1.0;
            if should_duck {
                filter_complex.push_str(&format!(
                    "{}amix=inputs={}:duration=first:dropout_transition=3:normalize=0[narr];",
                    narr_inputs.join(""),
                    narr_inputs.len()
                ));
                filter_complex.push_str(&format!(
                    "{}amix=inputs={}:duration=first:dropout_transition=3:normalize=0[music];",
                    music_inputs.join(""),
                    music_inputs.len()
                ));
                filter_complex.push_str(
                    "[music][narr]sidechaincompress=threshold=0.02:ratio=8:attack=20:release=250[ducked];",
                );
                filter_complex.push_str(&format!("[ducked]volume={}[duckv];", music_duck));
                filter_complex.push_str("[duckv][narr]amix=inputs=2:duration=first:dropout_transition=3:normalize=0[aout]");
            } else if audio_tracks.len() == 1 {
                // Single track: skip amix entirely to avoid any normalization artefacts
                filter_complex.push_str(&format!("{}anull[aout]", mix_inputs,));
            } else {
                filter_complex.push_str(&format!(
                    "{}amix=inputs={}:duration=first:dropout_transition=3:normalize=0[aout]",
                    mix_inputs,
                    audio_tracks.len()
                ));
            }

            cmd.args(["-filter_complex", &filter_complex]);
            cmd.args(["-map", "0:v", "-map", "[aout]"]);
            // Use AAC codec for audio
            cmd.args(["-c:a", "aac", "-b:a", "192k"]);
        }

        // Detect and apply the best available encoder
        let encoder = Self::detect_best_encoder();
        Self::apply_encoder_args(&mut cmd, &encoder);

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
                    "failed to write frame {} to ffmpeg: {}. FFmpeg stderr: {}",
                    i, e, stderr
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

    use vidra_core::frame::PixelFormat;

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

    #[test]
    fn test_detect_best_encoder() {
        // Should not panic on any platform and always return a valid variant
        let encoder = FfmpegEncoder::detect_best_encoder();
        match encoder {
            HwEncoder::VideoToolbox | HwEncoder::Nvenc | HwEncoder::Libx264 => {}
        }
    }

    #[test]
    fn test_encode_dimension_mismatch() {
        if !FfmpegEncoder::is_available() {
            return;
        }
        let frame1 = FrameBuffer::new(320, 240, PixelFormat::Rgba8);
        let frame2 = FrameBuffer::new(640, 480, PixelFormat::Rgba8); // wrong dimensions
        let result = FfmpegEncoder::encode(
            &[frame1, frame2],
            &[],
            320,
            240,
            30.0,
            Path::new("/tmp/vidra_test_mismatch.mp4"),
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("dimensions"),
            "Error should mention dimensions: {}",
            err
        );
    }

    #[test]
    fn test_encode_small_video() {
        if !FfmpegEncoder::is_available() {
            return;
        }
        // Generate 10 frames of solid red 64x64
        let frames: Vec<FrameBuffer> = (0..10)
            .map(|_| {
                let mut fb = FrameBuffer::new(64, 64, PixelFormat::Rgba8);
                for pixel in fb.data.chunks_mut(4) {
                    pixel[0] = 255; // R
                    pixel[1] = 0; // G
                    pixel[2] = 0; // B
                    pixel[3] = 255; // A
                }
                fb
            })
            .collect();

        let output_path = Path::new("/tmp/vidra_test_small_encode.mp4");
        let result = FfmpegEncoder::encode(&frames, &[], 64, 64, 30.0, output_path);
        assert!(result.is_ok(), "Encoding failed: {:?}", result.err());
        assert!(output_path.exists(), "Output file should exist");
        let meta = std::fs::metadata(output_path).unwrap();
        assert!(meta.len() > 0, "Output file should not be empty");
        // Cleanup
        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn test_encode_creates_output_directory() {
        if !FfmpegEncoder::is_available() {
            return;
        }
        let frames = vec![FrameBuffer::new(64, 64, PixelFormat::Rgba8)];
        let output_path = Path::new("/tmp/vidra_test_subdir/nested/output.mp4");
        let _ = std::fs::remove_dir_all("/tmp/vidra_test_subdir");

        let result = FfmpegEncoder::encode(&frames, &[], 64, 64, 30.0, output_path);
        assert!(result.is_ok(), "Encoding failed: {:?}", result.err());
        assert!(
            output_path.exists(),
            "Output file should exist in nested dir"
        );
        // Cleanup
        let _ = std::fs::remove_dir_all("/tmp/vidra_test_subdir");
    }

    #[test]
    fn test_apply_encoder_args_libx264() {
        let mut cmd = Command::new("echo");
        FfmpegEncoder::apply_encoder_args(&mut cmd, &HwEncoder::Libx264);
        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("libx264")));
        assert!(args.contains(&std::ffi::OsStr::new("fast")));
        assert!(args.contains(&std::ffi::OsStr::new("yuv420p")));
    }

    #[test]
    fn test_apply_encoder_args_videotoolbox() {
        let mut cmd = Command::new("echo");
        FfmpegEncoder::apply_encoder_args(&mut cmd, &HwEncoder::VideoToolbox);
        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("h264_videotoolbox")));
        assert!(args.contains(&std::ffi::OsStr::new("yuv420p")));
        assert!(args.contains(&std::ffi::OsStr::new("+faststart")));
    }

    #[test]
    fn test_apply_encoder_args_nvenc() {
        let mut cmd = Command::new("echo");
        FfmpegEncoder::apply_encoder_args(&mut cmd, &HwEncoder::Nvenc);
        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("h264_nvenc")));
        assert!(args.contains(&std::ffi::OsStr::new("p4")));
    }

    #[test]
    fn test_audio_track_default() {
        let track = AudioTrack {
            path: std::path::PathBuf::from("test.mp3"),
            trim_start: 0.0,
            trim_end: None,
            volume: 1.0,
            role: None,
            duck: None,
        };
        assert_eq!(track.volume, 1.0);
        assert!(track.role.is_none());
    }
}
