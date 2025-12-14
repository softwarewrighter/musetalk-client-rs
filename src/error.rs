//! Error types for the MuseTalk CLI.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the CLI application.
#[derive(Error, Debug)]
pub enum CliError {
    /// Image file not found at the specified path.
    #[error("Image file not found: {0}")]
    ImageNotFound(PathBuf),

    /// Audio file not found at the specified path.
    #[error("Audio file not found: {0}")]
    AudioNotFound(PathBuf),

    /// Unsupported image format.
    #[error("Unsupported image format: {0}. Supported formats: PNG, JPEG")]
    UnsupportedImageFormat(String),

    /// Unsupported audio format.
    #[error("Unsupported audio format: {0}. Supported formats: WAV, MP3, FLAC")]
    UnsupportedAudioFormat(String),

    /// Invalid output path.
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(PathBuf),

    /// Server connection error.
    #[error("Failed to connect to server: {0}")]
    ServerConnection(String),

    /// General I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias using CliError.
pub type Result<T> = std::result::Result<T, CliError>;
