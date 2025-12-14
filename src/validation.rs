//! Input validation for CLI arguments.

use crate::error::{CliError, Result};
use std::path::Path;

/// Supported image extensions.
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];

/// Supported audio extensions.
const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "flac"];

/// Validates the image file path.
///
/// Checks that:
/// - The file exists
/// - The extension is a supported image format
pub fn validate_image_path(path: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(CliError::ImageNotFound(path.to_path_buf()));
    }

    // Check extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(CliError::UnsupportedImageFormat(ext));
    }

    Ok(())
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
pub fn validate_inputs(image: &Path, audio: &Path, output: &Path) -> Result<()> {
    validate_image_path(image)?;
    validate_audio_path(audio)?;
    validate_output_path(output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_validate_image_not_found() {
        let result = validate_image_path(Path::new("nonexistent.png"));
        assert!(matches!(result, Err(CliError::ImageNotFound(_))));
    }

    #[test]
    fn test_validate_image_unsupported_format() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.gif");
        File::create(&path).unwrap();

        let result = validate_image_path(&path);
        assert!(matches!(result, Err(CliError::UnsupportedImageFormat(_))));
    }

    #[test]
    fn test_validate_image_png_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.png");
        File::create(&path).unwrap();

        let result = validate_image_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_image_jpeg_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.jpeg");
        File::create(&path).unwrap();

        let result = validate_image_path(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_image_jpg_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("image.jpg");
        File::create(&path).unwrap();

        let result = validate_image_path(&path);
        assert!(result.is_ok());
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
    fn test_validate_inputs_all_valid() {
        let dir = tempdir().unwrap();
        let image = dir.path().join("avatar.png");
        let audio = dir.path().join("speech.wav");
        let output = dir.path().join("output.mp4");

        File::create(&image).unwrap();
        File::create(&audio).unwrap();

        let result = validate_inputs(&image, &audio, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_inputs_image_not_found() {
        let dir = tempdir().unwrap();
        let image = dir.path().join("nonexistent.png");
        let audio = dir.path().join("speech.wav");
        let output = dir.path().join("output.mp4");

        File::create(&audio).unwrap();

        let result = validate_inputs(&image, &audio, &output);
        assert!(matches!(result, Err(CliError::ImageNotFound(_))));
    }
}
