//! Error types for the MuseTalk CLI.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the CLI application.
#[derive(Error, Debug)]
pub enum CliError {
    /// Reference file not found at the specified path.
    #[error("Reference file not found: {0}")]
    ReferenceNotFound(PathBuf),

    /// Audio file not found at the specified path.
    #[error("Audio file not found: {0}")]
    AudioNotFound(PathBuf),

    /// Unsupported reference format.
    #[error("Unsupported reference format: {0}. Supported formats: PNG, JPEG, MP4")]
    UnsupportedReferenceFormat(String),

    /// Unsupported audio format.
    #[error("Unsupported audio format: {0}. Supported formats: WAV, MP3, FLAC")]
    UnsupportedAudioFormat(String),

    /// Invalid output path.
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(PathBuf),

    /// Server connection error.
    #[error("Failed to connect to server: {0}")]
    ServerConnection(String),

    /// Image loading/processing error.
    #[error("Image loading error: {0}")]
    ImageLoad(String),

    /// Video loading/processing error.
    #[error("Video loading error: {0}")]
    VideoLoad(String),

    /// Audio loading/processing error.
    #[error("Audio loading error: {0}")]
    AudioLoad(String),

    /// Video encoding error.
    #[error("Video encoding error: {0}")]
    Video(String),

    /// General I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias using CliError.
pub type Result<T> = std::result::Result<T, CliError>;
