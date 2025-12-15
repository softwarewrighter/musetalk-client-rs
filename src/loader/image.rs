//! Image loading and preprocessing.

use crate::error::{CliError, Result};
use base64::Engine;
use image::GenericImageView;
use std::path::Path;

/// Loaded image data ready for processing.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Raw RGB bytes.
    pub rgb_data: Vec<u8>,
    /// Base64-encoded PNG for API transmission.
    pub base64_png: String,
}

/// Loads an image from the given path.
///
/// Converts to RGB format and prepares for API transmission.
pub fn load_image(path: &Path) -> Result<ImageData> {
    tracing::debug!("Loading image from: {}", path.display());

    let img = image::open(path).map_err(|e| CliError::ImageLoad(e.to_string()))?;

    let (width, height) = img.dimensions();
    tracing::debug!("Image dimensions: {width}x{height}");

    // Convert to RGB8
    let rgb_img = img.to_rgb8();
    let rgb_data = rgb_img.as_raw().clone();

    // Encode as PNG for transmission
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    rgb_img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| CliError::ImageLoad(format!("Failed to encode PNG: {e}")))?;

    let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    tracing::info!(
        "Loaded image: {}x{}, {} bytes (base64: {} chars)",
        width,
        height,
        rgb_data.len(),
        base64_png.len()
    );

    Ok(ImageData {
        width,
        height,
        rgb_data,
        base64_png,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_png_image() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.png");

        // Create a simple 2x2 red PNG
        let img = image::RgbImage::from_fn(2, 2, |_, _| image::Rgb([255, 0, 0]));
        img.save(&path).unwrap();

        let data = load_image(&path).unwrap();
        assert_eq!(data.width, 2);
        assert_eq!(data.height, 2);
        assert_eq!(data.rgb_data.len(), 2 * 2 * 3); // 2x2 pixels, 3 bytes each
        assert!(!data.base64_png.is_empty());
    }

    #[test]
    fn test_load_jpeg_image() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.jpg");

        // Create a simple 4x4 blue JPEG
        let img = image::RgbImage::from_fn(4, 4, |_, _| image::Rgb([0, 0, 255]));
        img.save(&path).unwrap();

        let data = load_image(&path).unwrap();
        assert_eq!(data.width, 4);
        assert_eq!(data.height, 4);
    }

    #[test]
    fn test_load_nonexistent_image() {
        let result = load_image(Path::new("nonexistent.png"));
        assert!(result.is_err());
    }
}
