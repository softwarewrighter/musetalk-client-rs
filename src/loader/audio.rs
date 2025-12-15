//! Audio loading and preprocessing.

use crate::error::{CliError, Result};
use base64::Engine;
use hound::WavReader;
use std::path::Path;

/// Loaded audio data ready for processing.
#[derive(Debug, Clone)]
pub struct AudioData {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Duration in seconds.
    pub duration_secs: f32,
    /// Raw audio samples as f32 (normalized -1.0 to 1.0).
    pub samples: Vec<f32>,
    /// Base64-encoded WAV for API transmission.
    pub base64_wav: String,
}

/// Loads a WAV audio file from the given path.
pub fn load_audio(path: &Path) -> Result<AudioData> {
    tracing::debug!("Loading audio from: {}", path.display());

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "wav" => load_wav(path),
        "mp3" | "flac" => Err(CliError::AudioLoad(format!(
            "{} format not yet implemented, please convert to WAV",
            ext.to_uppercase()
        ))),
        _ => Err(CliError::UnsupportedAudioFormat(ext)),
    }
}

fn load_wav(path: &Path) -> Result<AudioData> {
    let reader = WavReader::open(path).map_err(|e| CliError::AudioLoad(e.to_string()))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels;
    let bits_per_sample = spec.bits_per_sample;

    tracing::debug!(
        "WAV spec: {} Hz, {} channels, {} bits",
        sample_rate,
        channels,
        bits_per_sample
    );

    // Read samples based on format
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_val = (1 << (bits_per_sample - 1)) as f32;
            reader
                .into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect()
        }
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .filter_map(|s| s.ok())
            .collect(),
    };

    let num_samples = samples.len();
    let duration_secs = num_samples as f32 / (sample_rate as f32 * channels as f32);

    // Read raw file bytes for base64 encoding
    let wav_bytes = std::fs::read(path).map_err(CliError::Io)?;
    let base64_wav = base64::engine::general_purpose::STANDARD.encode(&wav_bytes);

    tracing::info!(
        "Loaded audio: {:.2}s, {} Hz, {} ch, {} samples (base64: {} chars)",
        duration_secs,
        sample_rate,
        channels,
        num_samples,
        base64_wav.len()
    );

    Ok(AudioData {
        sample_rate,
        channels,
        duration_secs,
        samples,
        base64_wav,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{WavSpec, WavWriter};
    use tempfile::tempdir;

    fn create_test_wav(path: &Path, sample_rate: u32, duration_secs: f32) {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = WavWriter::create(path, spec).unwrap();

        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        for i in 0..num_samples {
            // Generate a simple sine wave
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
            let sample_i16 = (sample * 32767.0) as i16;
            writer.write_sample(sample_i16).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn test_load_wav() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.wav");
        create_test_wav(&path, 16000, 1.0);

        let data = load_audio(&path).unwrap();
        assert_eq!(data.sample_rate, 16000);
        assert_eq!(data.channels, 1);
        assert!((data.duration_secs - 1.0).abs() < 0.1);
        assert!(!data.samples.is_empty());
        assert!(!data.base64_wav.is_empty());
    }

    #[test]
    fn test_load_nonexistent_audio() {
        let result = load_audio(Path::new("nonexistent.wav"));
        assert!(result.is_err());
    }
}
