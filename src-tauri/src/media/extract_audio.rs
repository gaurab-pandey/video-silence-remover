use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

/// Extracts audio from a video file to WAV format using FFmpeg
/// Returns the path to the extracted WAV file
pub fn extract_audio_to_wav(video_path: &str, output_dir: &Path, ffmpeg_path: &Path) -> Result<PathBuf, String> {
    log::info!("Extracting audio from: {}", video_path);
    
    // Verify video file exists
    if !Path::new(video_path).exists() {
        return Err(format!("Video file not found: {}", video_path));
    }
    
    // Create output filename
    let video_filename = Path::new(video_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid video filename")?;
    
    let output_path = output_dir.join(format!("{}_audio.wav", video_filename));
    
    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }
    
    log::info!("Output WAV path: {:?}", output_path);
    
    // Run FFmpeg command to extract audio
    // -i: input file
    // -vn: no video
    // -acodec pcm_s16le: 16-bit PCM audio codec
    // -ar 44100: 44.1kHz sample rate
    // -ac 1: mono audio
    let output = Command::new(ffmpeg_path)
        .args(&[
            "-i", video_path,
            "-vn",
            "-acodec", "pcm_s16le",
            "-ar", "44100",
            "-ac", "1",
            "-y", // Overwrite output file if exists
            output_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "FFmpeg not found. Please ensure FFmpeg is installed and in your PATH.".to_string()
            } else {
                format!("Failed to execute FFmpeg: {}", e)
            }
        })?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg failed: {}", stderr));
    }
    
    log::info!("Audio extraction completed successfully");
    Ok(output_path)
}

/// Gets the duration of a video file in seconds using FFprobe
pub fn get_video_duration(video_path: &str, ffprobe_path: &Path) -> Result<f64, String> {
    log::info!("Getting duration for: {}", video_path);
    
    // Use FFprobe to get video duration
    let output = Command::new(ffprobe_path)
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "FFprobe not found. Please ensure FFmpeg (with FFprobe) is installed.".to_string()
            } else {
                format!("Failed to execute FFprobe: {}", e)
            }
        })?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFprobe failed: {}", stderr));
    }
    
    let duration_str = String::from_utf8_lossy(&output.stdout);
    let duration = duration_str.trim()
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse duration: {}", e))?;
    
    log::info!("Video duration: {} seconds", duration);
    Ok(duration)
}
