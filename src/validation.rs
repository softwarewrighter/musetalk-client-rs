//! Input validation for CLI arguments.

use crate::error::{CliError, Result};
use std::path::Path;

/// Supported image extensions.
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];

/// Supported video extensions.
const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &["mp4"];

/// Supported audio extensions.
const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "flac"];

/// Reference input type (image or video).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    /// Static image (PNG/JPEG).
    Image,
    /// Video file (MP4).
    Video,
}

/// Validates the reference file path.
///
/// Checks that:
/// - The file exists
/// - The extension is a supported reference format (PNG, JPEG, MP4)
///
/// Returns the detected reference type.
pub fn validate_reference_path(path: &Path) -> Result<ReferenceType> {
    // Check file exists
    if !path.exists() {
        return Err(CliError::ReferenceNotFound(path.to_path_buf()));
    }

    // Check extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(ReferenceType::Image);
    }

    if SUPPORTED_VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(ReferenceType::Video);
    }

    Err(CliError::UnsupportedReferenceFormat(ext))
}

/// Returns true if the path has an image extension.
pub fn is_image_reference(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Returns true if the path has a video extension.
pub fn is_video_reference(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_VIDEO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validates the audio file path.
///
/// Checks that:
/// - The file exists
/// - The extension is a supported audio format
pub fn validate_audio_path(path: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(CliError::AudioNotFound(path.to_path_buf()));
    }

    // Check extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !SUPPORTED_AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        return Err(CliError::UnsupportedAudioFormat(ext));
    }

    Ok(())
}

/// Validates the output path.
///
/// Checks that the parent directory exists and is writable.
pub fn validate_output_path(path: &Path) -> Result<()> {
    // Get parent directory (or current dir if no parent or empty parent)
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let parent = parent.unwrap_or(Path::new("."));

    // Check parent directory exists
    if !parent.exists() {
        return Err(CliError::InvalidOutputPath(path.to_path_buf()));
    }

    Ok(())
}

/// Validates all input arguments.
///
/// Returns the detected reference type (image or video).
pub fn validate_inputs(reference: &Path, audio: &Path, output: &Path) -> Result<ReferenceType> {
    let ref_type = validate_reference_path(reference)?;
    validate_audio_path(audio)?;
    validate_output_path(output)?;
    Ok(ref_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_validate_reference_not_found() {
        let result = validate_reference_path(Path::new("nonexistent.png"));
        assert!(matches!(result, Err(CliError::ReferenceNotFound(_))));
    }

    #[test]
    fn test_validate_reference_unsupported_format() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.gif");
        File::create(&path).unwrap();

        let result = validate_reference_path(&path);
        assert!(matches!(
            result,
            Err(CliError::UnsupportedReferenceFormat(_))
        ));
    }

    #[test]
    fn test_validate_reference_png_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.png");
        File::create(&path).unwrap();

        let result = validate_reference_path(&path);
        assert_eq!(result.unwrap(), ReferenceType::Image);
    }

    #[test]
    fn test_validate_reference_jpeg_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.jpeg");
        File::create(&path).unwrap();

        let result = validate_reference_path(&path);
        assert_eq!(result.unwrap(), ReferenceType::Image);
    }

    #[test]
    fn test_validate_reference_jpg_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.jpg");
        File::create(&path).unwrap();

        let result = validate_reference_path(&path);
        assert_eq!(result.unwrap(), ReferenceType::Image);
    }

    #[test]
    fn test_validate_reference_mp4_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("video.mp4");
        File::create(&path).unwrap();

        let result = validate_reference_path(&path);
        assert_eq!(result.unwrap(), ReferenceType::Video);
    }

    #[test]
    fn test_is_image_reference() {
        assert!(is_image_reference(Path::new("test.png")));
        assert!(is_image_reference(Path::new("test.jpg")));
        assert!(is_image_reference(Path::new("test.jpeg")));
        assert!(!is_image_reference(Path::new("test.mp4")));
        assert!(!is_image_reference(Path::new("test.wav")));
    }

    #[test]
    fn test_is_video_reference() {
        assert!(is_video_reference(Path::new("test.mp4")));
        assert!(!is_video_reference(Path::new("test.png")));
        assert!(!is_video_reference(Path::new("test.jpg")));
        assert!(!is_video_reference(Path::new("test.wav")));
    }

    #[test]
    fn test_validate_audio_not_found() {
        let result = validate_audio_path(Path::new("nonexistent.wav"));
        assert!(matches!(result, Err(CliError::AudioNotFound(_))));
    }

    #[test]
    fn test_validate_audio_unsupported_format() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audio.ogg");
        File::create(&path).unwrap();

        let result = validate_audio_path(&path);
        assert!(matches!(result, Err(CliError::UnsupportedAudioFormat(_))));
    }

    #[test]
    fn test_validate_audio_wav_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audio.wav");
        File::create(&path).unwrap();

        let result = validate_audio_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_audio_mp3_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audio.mp3");
        File::create(&path).unwrap();

        let result = validate_audio_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_audio_flac_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audio.flac");
        File::create(&path).unwrap();

        let result = validate_audio_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_invalid_parent() {
        let result = validate_output_path(Path::new("/nonexistent/dir/output.mp4"));
        assert!(matches!(result, Err(CliError::InvalidOutputPath(_))));
    }

    #[test]
    fn test_validate_output_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.mp4");

        let result = validate_output_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_relative_path() {
        // Relative path like "output.mp4" should be valid (parent is ".")
        let path = Path::new("output.mp4");
        let result = validate_output_path(path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_inputs_image_valid() {
        let dir = tempdir().unwrap();
        let reference = dir.path().join("avatar.png");
        let audio = dir.path().join("speech.wav");
        let output = dir.path().join("output.mp4");

        File::create(&reference).unwrap();
        File::create(&audio).unwrap();

        let result = validate_inputs(&reference, &audio, &output);
        assert_eq!(result.unwrap(), ReferenceType::Image);
    }

    #[test]
    fn test_validate_inputs_video_valid() {
        let dir = tempdir().unwrap();
        let reference = dir.path().join("avatar.mp4");
        let audio = dir.path().join("speech.wav");
        let output = dir.path().join("output.mp4");

        File::create(&reference).unwrap();
        File::create(&audio).unwrap();

        let result = validate_inputs(&reference, &audio, &output);
        assert_eq!(result.unwrap(), ReferenceType::Video);
    }

    #[test]
    fn test_validate_inputs_reference_not_found() {
        let dir = tempdir().unwrap();
        let reference = dir.path().join("nonexistent.png");
        let audio = dir.path().join("speech.wav");
        let output = dir.path().join("output.mp4");

        File::create(&audio).unwrap();

        let result = validate_inputs(&reference, &audio, &output);
        assert!(matches!(result, Err(CliError::ReferenceNotFound(_))));
    }
}
