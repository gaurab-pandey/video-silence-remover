use hound::WavReader;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Waveform data for UI visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    /// Peak amplitude values per bucket (normalized to 0.0-1.0)
    pub peaks: Vec<f32>,
    /// Total audio duration in seconds
    pub duration: f64,
    /// Milliseconds per bucket
    pub bucket_ms: u32,
}

/// Extracts waveform peak data from a WAV file
/// Returns normalized peak values for UI rendering
pub fn extract_waveform(wav_path: &Path, bucket_ms: u32) -> Result<WaveformData, String> {
    log::info!("Extracting waveform from: {:?}, bucket_ms: {}", wav_path, bucket_ms);
    
    let mut reader = WavReader::open(wav_path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;
    
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f64;
    let samples_per_bucket = (sample_rate * bucket_ms as f64 / 1000.0) as usize;
    
    if samples_per_bucket == 0 {
        return Err("Bucket size too small for sample rate".to_string());
    }
    
    // Read all samples
    let samples: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read samples: {}", e))?;
    
    if samples.is_empty() {
        return Err("WAV file contains no samples".to_string());
    }
    
    let duration = samples.len() as f64 / sample_rate / spec.channels as f64;
    
    // Calculate peak for each bucket
    let mono_samples: Vec<i16> = if spec.channels > 1 {
        // Average channels to mono
        samples.chunks(spec.channels as usize)
            .map(|chunk| {
                let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
                (sum / chunk.len() as i32) as i16
            })
            .collect()
    } else {
        samples
    };
    
    let peaks: Vec<f32> = mono_samples
        .chunks(samples_per_bucket)
        .map(|bucket| {
            let max_val = bucket.iter()
                .map(|&s| s.abs() as f32)
                .fold(0.0f32, |a, b| a.max(b));
            // Normalize to 0.0-1.0
            max_val / 32768.0
        })
        .collect();
    
    log::info!("Extracted {} waveform peaks for {:.2}s audio", peaks.len(), duration);
    
    Ok(WaveformData {
        peaks,
        duration,
        bucket_ms,
    })
}
