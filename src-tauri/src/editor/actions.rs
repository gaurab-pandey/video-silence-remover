use crate::media::{extract_audio_to_wav, get_video_duration};
use crate::analysis::{detect_silence, SilenceDetectionConfig};
use crate::timeline::Timeline;
use std::path::PathBuf;
use std::env;



/// Orchestrates video processing and returns both Timeline and WAV path
/// WAV path is needed for waveform visualization
pub fn process_video_pipeline_with_wav(
    video_path: &str,
    config: &SilenceDetectionConfig,
    ffmpeg_path: &PathBuf,
    ffprobe_path: &PathBuf,
) -> Result<(Timeline, PathBuf), String> {
    log::info!("Starting video processing pipeline for: {}", video_path);
    
    // Validate video file

    
    // Get video duration
    let duration = get_video_duration(video_path, ffprobe_path)?;
    log::info!("Video duration: {:.2} seconds", duration);
    
    // Extract audio to temporary directory
    let temp_dir = env::temp_dir().join("video-silence-remover");
    let wav_path = extract_audio_to_wav(video_path, &temp_dir, ffmpeg_path)?;
    
    // Detect silence
    let silence_ranges = detect_silence(&wav_path, config)?;
    
    // Create timeline and split by silence
    let mut timeline = Timeline::new(duration, video_path.to_string());
    timeline.split_by_silence(silence_ranges);
    
    log::info!("Pipeline completed successfully");
    Ok((timeline, wav_path))
}




