use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::timeline::Timeline;
use tauri::{Window, Emitter};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct ExportProgress {
    percentage: f64,
    current_time: f64,
    total_duration: f64,
}

/// Exports the edited video based on the final timeline
/// Uses FFmpeg's filter_complex to trim and concatenate clips
/// Emits progress events to the frontend
/// Only includes clips where include=true
pub fn export_video(
    timeline: &Timeline,
    output_path: &str,
    ffmpeg_path: &std::path::PathBuf,
    window: Window,
) -> Result<String, String> {
    log::info!("Starting video export to: {}", output_path);
    
    // Filter to only include clips that are marked include=true
    let included_clips: Vec<_> = timeline.clips.iter()
        .filter(|clip| clip.include)
        .cloned()
        .collect();
    
    log::info!("Exporting {} of {} clips (include=true)", 
               included_clips.len(), timeline.clips.len());
    
    if included_clips.is_empty() {
        return Err("Cannot export: no clips are included".to_string());
    }
    
    // Verify input file exists
    if !Path::new(&timeline.video_path).exists() {
        return Err(format!("Source video file not found: {}", timeline.video_path));
    }
    
    // Build FFmpeg filter_complex command using included clips only
    let filter = build_filter_complex_from_clips(&included_clips)?;
    
    log::info!("FFmpeg filter: {}", filter);
    
    // Calculate total output duration for progress tracking
    let total_duration: f64 = included_clips.iter()
        .map(|clip| clip.source_end - clip.source_start)
        .sum();
    
    // Execute FFmpeg export with progress monitoring
    let mut child = Command::new(ffmpeg_path)
        .args(&[
            "-i", &timeline.video_path,
            "-filter_complex", &filter,
            "-map", "[outv]",
            "-map", "[outa]",
            "-c:v", "libx264", // H.264 video codec
            "-preset", "medium", // Encoding speed preset
            "-crf", "23", // Quality (lower = better, 18-28 is reasonable)
            "-c:a", "aac", // AAC audio codec
            "-b:a", "192k", // Audio bitrate
            "-progress", "pipe:2", // Send progress to stderr
            "-y", // Overwrite output file
            output_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    // Read progress from stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        
        for line in reader.lines() {
            if let Ok(line) = line {
                // FFmpeg progress format: "out_time_ms=12345678"
                if line.starts_with("out_time_us=") {
                    if let Some(time_str) = line.strip_prefix("out_time_us=") {
                        if let Ok(time_us) = time_str.trim().parse::<i64>() {
                            let current_time = time_us as f64 / 1_000_000.0; // Convert to seconds
                            let percentage = ((current_time / total_duration) * 100.0).min(100.0);
                            
                            // Emit progress event
                            let progress = ExportProgress {
                                percentage,
                                current_time,
                                total_duration,
                            };
                            
                            let _ = window.emit("export-progress", progress);
                        }
                    }
                }
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait()
        .map_err(|e| format!("Failed to wait for FFmpeg: {}", e))?;
    
    if !status.success() {
        return Err("FFmpeg export failed".to_string());
    }
    
    log::info!("Video export completed successfully");
    Ok(output_path.to_string())
}



/// Builds the FFmpeg filter_complex string from a slice of clips
fn build_filter_complex_from_clips(clips: &[crate::timeline::Clip]) -> Result<String, String> {
    let mut video_filters = Vec::new();
    let mut audio_filters = Vec::new();
    let mut concat_inputs = Vec::new();
    
    for (i, clip) in clips.iter().enumerate() {
        // Video trim filter
        video_filters.push(format!(
            "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS[v{}]",
            clip.source_start, clip.source_end, i
        ));
        
        // Audio trim filter
        audio_filters.push(format!(
            "[0:a]atrim=start={}:end={},asetpts=PTS-STARTPTS[a{}]",
            clip.source_start, clip.source_end, i
        ));
        
        concat_inputs.push(format!("[v{}][a{}]", i, i));
    }
    
    // Combine all filters
    let mut filter = String::new();
    
    // Add video filters
    for vf in video_filters {
        filter.push_str(&vf);
        filter.push_str(";");
    }
    
    // Add audio filters
    for af in audio_filters {
        filter.push_str(&af);
        filter.push_str(";");
    }
    
    // Add concat filter
    filter.push_str(&concat_inputs.join(""));
    filter.push_str(&format!(
        "concat=n={}:v=1:a=1[outv][outa]",
        clips.len()
    ));
    
    Ok(filter)
}


