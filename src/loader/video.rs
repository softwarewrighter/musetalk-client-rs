//! Video loading for reference videos.

use crate::error::{CliError, Result};
use base64::Engine;
use std::path::Path;

/// Loaded video data ready for API transmission.
#[derive(Debug, Clone)]
pub struct VideoData {
    /// Base64-encoded MP4 for API transmission.
    pub base64_mp4: String,
    /// File size in bytes.
    pub file_size: u64,
}

/// Loads a video from the given path.
///
/// Reads the video file and encodes it as base64 for API transmission.
pub fn load_video(path: &Path) -> Result<VideoData> {
    tracing::debug!("Loading video from: {}", path.display());

    let bytes = std::fs::read(path)
        .map_err(|e| CliError::VideoLoad(format!("Failed to read video file: {e}")))?;

    let file_size = bytes.len() as u64;
    let base64_mp4 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    tracing::info!(
        "Loaded video: {} bytes (base64: {} chars)",
        file_size,
        base64_mp4.len()
    );

    Ok(VideoData {
        base64_mp4,
        file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_video_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.mp4");

        // Create a dummy MP4 file (just bytes, not a real video)
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"fake mp4 content").unwrap();

        let data = load_video(&path).unwrap();
        assert!(!data.base64_mp4.is_empty());
        assert_eq!(data.file_size, 16); // "fake mp4 content" is 16 bytes
    }

    #[test]
    fn test_load_nonexistent_video() {
        let result = load_video(Path::new("nonexistent.mp4"));
        assert!(result.is_err());
    }
}
