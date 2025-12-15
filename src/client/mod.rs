//! HTTP client for MuseTalk server communication.

pub mod types;

use crate::error::{CliError, Result};
use crate::loader::{AudioData, ImageData, VideoData};
use std::error::Error as StdError;
pub use types::{InferenceRequest, InferenceResponse, ServerHealth};

/// Reference input for inference (image or video).
pub enum ReferenceInput<'a> {
    /// Static image reference.
    Image(&'a ImageData),
    /// Video reference.
    Video(&'a VideoData),
}

/// Client for communicating with the MuseTalk inference server.
pub struct MuseTalkClient {
    base_url: String,
    client: reqwest::Client,
}

impl MuseTalkClient {
    /// Creates a new client for the given server URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Checks if the server is healthy and returns version info.
    pub async fn health_check(&self) -> Result<ServerHealth> {
        let url = format!("{}/health", self.base_url);
        tracing::debug!("Health check: {url}");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| CliError::ServerConnection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CliError::ServerConnection(format!(
                "Health check failed: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| CliError::ServerConnection(format!("Invalid health response: {e}")))
    }

    /// Sends an inference request with image reference and returns generated frames.
    pub async fn infer_with_image(
        &self,
        image: &ImageData,
        audio: &AudioData,
        fps: u32,
    ) -> Result<InferenceResponse> {
        let request = InferenceRequest {
            image: Some(image.base64_png.clone()),
            video: None,
            audio: audio.base64_wav.clone(),
            fps,
        };
        self.send_inference_request(request).await
    }

    /// Sends an inference request with video reference and returns generated frames.
    pub async fn infer_with_video(
        &self,
        video: &VideoData,
        audio: &AudioData,
        fps: u32,
    ) -> Result<InferenceResponse> {
        let request = InferenceRequest {
            image: None,
            video: Some(video.base64_mp4.clone()),
            audio: audio.base64_wav.clone(),
            fps,
        };
        self.send_inference_request(request).await
    }

    /// Sends an inference request with a reference input (image or video).
    pub async fn infer(
        &self,
        reference: ReferenceInput<'_>,
        audio: &AudioData,
        fps: u32,
    ) -> Result<InferenceResponse> {
        match reference {
            ReferenceInput::Image(image) => self.infer_with_image(image, audio, fps).await,
            ReferenceInput::Video(video) => self.infer_with_video(video, audio, fps).await,
        }
    }

    /// Internal helper to send inference request.
    async fn send_inference_request(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let url = format!("{}/infer", self.base_url);
        tracing::debug!("Inference request: {url}");

        // Log request size for debugging
        let request_size = request.image.as_ref().map(|s| s.len()).unwrap_or(0)
            + request.video.as_ref().map(|s| s.len()).unwrap_or(0)
            + request.audio.len();
        tracing::info!(
            "Sending inference request: {} MB total",
            request_size as f64 / 1_000_000.0
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(900)) // 15 minutes for video processing
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Request failed: {e:?}");
                let source_msg = StdError::source(&e)
                    .map(|s| format!(": {s}"))
                    .unwrap_or_default();
                CliError::ServerConnection(format!("{e}{source_msg}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::ServerConnection(format!(
                "Inference failed: {status} - {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| CliError::ServerConnection(format!("Invalid inference response: {e}")))
    }
}
