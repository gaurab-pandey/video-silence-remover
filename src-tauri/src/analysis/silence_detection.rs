use hound::{WavReader, WavSpec};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Configuration for silence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceDetectionConfig {
    /// Silence threshold in dB (e.g., -35.0)
    pub threshold_db: f64,
    /// Minimum silence duration in seconds (e.g., 0.3 for 300ms)
    pub min_silence_duration: f64,
}

impl Default for SilenceDetectionConfig {
    fn default() -> Self {
        SilenceDetectionConfig {
            threshold_db: -35.0,
            min_silence_duration: 0.3,
        }
    }
}

/// Detects silence ranges in a WAV audio file
/// Returns a vector of (start_time, end_time) tuples representing silent segments
pub fn detect_silence(
    wav_path: &Path,
    config: &SilenceDetectionConfig,
) -> Result<Vec<(f64, f64)>, String> {
    log::info!("Starting silence detection on: {:?}", wav_path);
    log::info!("Threshold: {} dB, Min duration: {} s", 
               config.threshold_db, config.min_silence_duration);
    
    // Open WAV file
    let mut reader = WavReader::open(wav_path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;
    
    let spec = reader.spec();
    log::info!("WAV spec: {:?}", spec);
    
    // Read all samples
    let samples: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read samples: {}", e))?;
    
    if samples.is_empty() {
        return Err("WAV file contains no samples".to_string());
    }
    
    log::info!("Read {} samples", samples.len());
    
    // Calculate silence using RMS analysis
    let silence_ranges = analyze_silence_rms(&samples, &spec, config);
    
    log::info!("Detected {} silence ranges", silence_ranges.len());
    for (i, (start, end)) in silence_ranges.iter().enumerate() {
        log::info!("  Silence {}: {:.2}s - {:.2}s ({:.2}s)", i + 1, start, end, end - start);
    }
    
    Ok(silence_ranges)
}

/// Analyzes audio samples using RMS (Root Mean Square) to detect silence
fn analyze_silence_rms(
    samples: &[i16],
    spec: &WavSpec,
    config: &SilenceDetectionConfig,
) -> Vec<(f64, f64)> {
    let sample_rate = spec.sample_rate as f64;
    
    // Use 10ms windows for analysis (typical for audio analysis)
    let window_size = (sample_rate * 0.01) as usize; // 10ms
    
    if window_size == 0 {
        log::error!("Window size is 0, sample rate might be too low");
        return Vec::new();
    }
    
    let mut silence_ranges = Vec::new();
    let mut silence_start: Option<f64> = None;
    
    // Convert threshold from dB to amplitude
    // dB = 20 * log10(amplitude / max_amplitude)
    // For 16-bit audio, max amplitude is 32768
    let threshold_amplitude = 32768.0 * 10f64.powf(config.threshold_db / 20.0);
    
    log::debug!("Threshold amplitude: {:.2}", threshold_amplitude);
    
    // Analyze each window
    for (i, window) in samples.chunks(window_size).enumerate() {
        let rms = calculate_rms(window);
        let time = (i * window_size) as f64 / sample_rate;
        
        let is_silent = rms < threshold_amplitude;
        
        match (is_silent, silence_start) {
            (true, None) => {
                // Start of new silence region
                silence_start = Some(time);
            }
            (false, Some(start)) => {
                // End of silence region
                let duration = time - start;
                if duration >= config.min_silence_duration {
                    silence_ranges.push((start, time));
                }
                silence_start = None;
            }
            _ => {}
        }
    }
    
    // Handle silence that extends to the end of the file
    if let Some(start) = silence_start {
        let end_time = samples.len() as f64 / sample_rate;
        let duration = end_time - start;
        if duration >= config.min_silence_duration {
            silence_ranges.push((start, end_time));
        }
    }
    
    silence_ranges
}

/// Calculates the Root Mean Square (RMS) of a set of audio samples
fn calculate_rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    
    let sum_of_squares: f64 = samples
        .iter()
        .map(|&s| {
            let s_f64 = s as f64;
            s_f64 * s_f64
        })
        .sum();
    
    (sum_of_squares / samples.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_calculation() {
        let samples = vec![100, -100, 100, -100];
        let rms = calculate_rms(&samples);
        assert_eq!(rms, 100.0);
    }

    #[test]
    fn test_rms_silence() {
        let samples = vec![0, 0, 0, 0];
        let rms = calculate_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_default_config() {
        let config = SilenceDetectionConfig::default();
        assert_eq!(config.threshold_db, -35.0);
        assert_eq!(config.min_silence_duration, 0.3);
    }
}
