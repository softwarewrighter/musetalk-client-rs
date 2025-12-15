//! Video assembly from frames and audio.

use crate::error::{CliError, Result};
use crate::loader::{AudioData, ImageData};
use base64::Engine;
use std::path::Path;
use std::process::Command;

/// Assembles frames into a video with audio.
///
/// Uses FFmpeg command line for encoding.
pub struct VideoAssembler {
    fps: u32,
    temp_dir: tempfile::TempDir,
}

impl VideoAssembler {
    /// Creates a new video assembler.
    pub fn new(fps: u32) -> Result<Self> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| CliError::Video(format!("Failed to create temp dir: {e}")))?;
        Ok(Self { fps, temp_dir })
    }

    /// Assembles a video from base64-encoded PNG frames and audio.
    pub fn assemble_from_frames(
        &self,
        frames: &[String],
        audio_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        tracing::info!("Assembling {} frames into video", frames.len());

        // Write frames to temp directory
        for (i, frame_b64) in frames.iter().enumerate() {
            let frame_path = self.temp_dir.path().join(format!("frame_{i:05}.png"));
            let frame_bytes = base64::engine::general_purpose::STANDARD
                .decode(frame_b64)
                .map_err(|e| CliError::Video(format!("Failed to decode frame {i}: {e}")))?;
            std::fs::write(&frame_path, frame_bytes)
                .map_err(|e| CliError::Video(format!("Failed to write frame {i}: {e}")))?;
        }

        // Run FFmpeg to combine frames and audio
        self.run_ffmpeg_frames(audio_path, output_path)
    }

    /// Creates a video from a static image and audio (passthrough mode).
    ///
    /// This is used when no server is available - creates a simple video
    /// of the static image with the audio track.
    pub fn assemble_static(
        &self,
        _image: &ImageData,
        audio: &AudioData,
        image_path: &Path,
        audio_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        tracing::info!(
            "Creating static video: {:.2}s at {} fps",
            audio.duration_secs,
            self.fps
        );

        self.run_ffmpeg_static(image_path, audio_path, audio.duration_secs, output_path)
    }

    fn run_ffmpeg_frames(&self, audio_path: &Path, output_path: &Path) -> Result<()> {
        let frame_pattern = self.temp_dir.path().join("frame_%05d.png");

        let status = Command::new("ffmpeg")
            .args([
                "-y", // Overwrite output
                "-framerate",
                &self.fps.to_string(),
                "-i",
                frame_pattern.to_str().unwrap(),
                "-i",
                audio_path.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "23",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-pix_fmt",
                "yuv420p",
                "-shortest",
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CliError::Video(format!("Failed to run ffmpeg: {e}")))?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            return Err(CliError::Video(format!("FFmpeg failed: {stderr}")));
        }

        tracing::info!("Video created: {}", output_path.display());
        Ok(())
    }

    fn run_ffmpeg_static(
        &self,
        image_path: &Path,
        audio_path: &Path,
        duration: f32,
        output_path: &Path,
    ) -> Result<()> {
        let status = Command::new("ffmpeg")
            .args([
                "-y", // Overwrite output
                "-loop",
                "1",
                "-i",
                image_path.to_str().unwrap(),
                "-i",
                audio_path.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "23",
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-pix_fmt",
                "yuv420p",
                "-t",
                &format!("{:.2}", duration),
                "-shortest",
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CliError::Video(format!("Failed to run ffmpeg: {e}")))?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            return Err(CliError::Video(format!("FFmpeg failed: {stderr}")));
        }

        tracing::info!("Static video created: {}", output_path.display());
        Ok(())
    }
}

/// Checks if FFmpeg is available on the system.
pub fn check_ffmpeg() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| CliError::Video("FFmpeg not found. Please install FFmpeg.".to_string()))?;

    if !output.status.success() {
        return Err(CliError::Video("FFmpeg check failed".to_string()));
    }

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("unknown");
    tracing::debug!("FFmpeg: {first_line}");

    Ok(())
}
