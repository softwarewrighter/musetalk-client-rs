//! Request and response types for the MuseTalk API.

use serde::{Deserialize, Serialize};

/// Server health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub status: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// Inference request payload.
///
/// Either `image` or `video` should be provided, not both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Base64-encoded PNG image (optional, use for static image reference).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Base64-encoded MP4 video (optional, use for video reference).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<String>,
    /// Base64-encoded WAV audio.
    pub audio: String,
    /// Target frames per second.
    pub fps: u32,
}

/// Inference response with generated frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub status: String,
    pub total_frames: usize,
    pub frames: Vec<Frame>,
}

/// A single generated frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub index: usize,
    /// Base64-encoded PNG frame data.
    pub data: String,
}
