// Declare modules
mod media;
mod analysis;
mod timeline;
mod editor;
mod export;

use std::sync::Mutex;
use std::path::PathBuf;
use tauri::State;
use analysis::{SilenceDetectionConfig, WaveformData};
use timeline::Timeline;

/// Application state to store the current timeline
struct AppState {
    timeline: Mutex<Option<Timeline>>,
    silence_config: Mutex<SilenceDetectionConfig>,
    /// Path to extracted WAV file for waveform generation
    wav_path: Mutex<Option<PathBuf>>,
}

/// Paths to bundled sidecar binaries
struct SidecarPaths {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

fn resolve_sidecar_path(program: &str) -> PathBuf {
    // Target triple for Windows (adjust if needed for cross-compilation)
    let target = "x86_64-pc-windows-msvc";
    let filename = format!("{}-{}.exe", program, target);
    
    // 1. Check next to executable (Production / Release)
    let current_exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = current_exe.parent().unwrap_or(std::path::Path::new("."));
    let path_prod = exe_dir.join(&filename);
    
    if path_prod.exists() {
        return path_prod;
    }
    
    // 2. Check in src-tauri/binaries (Development)
    // Assuming cargo run from src-tauri or similar depth
    // Need to find project root relative to target/debug/...
    // Usually target/debug/build/... is deep.
    // Let's try traversing up until we see src-tauri or direct absolute check if possible?
    // Simpler: Just rely on current working directory if run via npm run tauri dev?
    // In dev, CWD is src-tauri.
    let path_dev = std::env::current_dir().unwrap_or_default().join("binaries").join(&filename);
    if path_dev.exists() {
        return path_dev;
    }
    
    // Fallback: assume in PATH if not found (or return what we found and let it fail later)
    log::warn!("Could not find bundled sidecar '{}'. Falling back to system path 'ffmpeg'/'ffprobe'.", filename);
    PathBuf::from(format!("{}.exe", program))
}

/// Import and process a video file
#[tauri::command]

fn process_video(
    video_path: String,
    state: State<AppState>,
    sidecars: State<SidecarPaths>,
) -> Result<Timeline, String> {
    log::info!("Processing video: {}", video_path);
    
    // Get current silence detection config
    let config = state.silence_config.lock().unwrap().clone();
    
    // Process the video
    let (mut timeline, wav_path) = editor::process_video_pipeline_with_wav(
        &video_path, 
        &config,
        &sidecars.ffmpeg,
        &sidecars.ffprobe
    )?;
    
    // Store audio path in timeline for frontend playback
    timeline.audio_path = Some(wav_path.to_string_lossy().to_string());

    // Store in state
    *state.timeline.lock().unwrap() = Some(timeline.clone());
    *state.wav_path.lock().unwrap() = Some(wav_path);
    
    Ok(timeline)
}

/// Get the current timeline
#[tauri::command]
fn get_timeline(state: State<AppState>) -> Result<Timeline, String> {
    let timeline = state.timeline.lock().unwrap();
    timeline.clone().ok_or("No timeline loaded".to_string())
}

/// Delete all silence clips from the timeline
#[tauri::command]
fn delete_silence_clips(state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.delete_silence_clips();
        timeline.recalculate_timeline_times();
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Get video file path
#[tauri::command]
fn get_video_path(state: State<AppState>) -> Result<String, String> {
    log::info!("Getting video path");
    
    let timeline = state.timeline.lock().unwrap();
    
    if let Some(ref timeline) = *timeline {
        Ok(timeline.video_path.clone())
    } else {
        Err("No video loaded".to_string())
    }
}

#[tauri::command]
fn export_video(
    output_path: String,
    state: State<AppState>,
    sidecars: State<SidecarPaths>,
    window: tauri::Window,
) -> Result<String, String> {
    log::info!("Exporting video to: {}", output_path);
    
    let timeline = state.timeline.lock().unwrap();
    
    if let Some(ref timeline) = *timeline {
        export::export_video(timeline, &output_path, &sidecars.ffmpeg, window)
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Update silence detection configuration
#[tauri::command]
fn update_silence_config(
    threshold_db: f64,
    min_silence_duration: f64,
    state: State<AppState>,
) -> Result<(), String> {
    log::info!("Updating silence config: threshold={} dB, min_duration={} s", 
               threshold_db, min_silence_duration);
    
    let mut config = state.silence_config.lock().unwrap();
    config.threshold_db = threshold_db;
    config.min_silence_duration = min_silence_duration;
    
    Ok(())
}

/// Get current silence detection configuration
#[tauri::command]
fn get_silence_config(state: State<AppState>) -> SilenceDetectionConfig {
    state.silence_config.lock().unwrap().clone()
}

// ============= NEW SEGMENT REVIEW COMMANDS =============

/// Toggle include state for a segment
#[tauri::command]
fn toggle_segment(index: usize, state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.toggle_segment_include(index)?;
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Remove a segment from the timeline
#[tauri::command]
fn remove_segment(index: usize, state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.remove_segment(index)?;
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Adjust the boundary time between two segments
#[tauri::command]
fn adjust_segment_boundary(index: usize, new_time: f64, state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.adjust_segment_boundary(index, new_time)?;
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Merge segment with the next segment
#[tauri::command]
fn merge_segments(index: usize, state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.merge_segments(index)?;
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Re-run silence detection with current config
#[tauri::command]
fn rerun_analysis(state: State<AppState>, sidecars: State<SidecarPaths>) -> Result<Timeline, String> {
    let timeline_opt = state.timeline.lock().unwrap();
    let video_path = timeline_opt.as_ref()
        .map(|t| t.video_path.clone())
        .ok_or("No video loaded")?;
    drop(timeline_opt);
    
    let config = state.silence_config.lock().unwrap().clone();
    
    log::info!("Re-running analysis with threshold={} dB, min_duration={} s",
               config.threshold_db, config.min_silence_duration);
    
    let (timeline, wav_path) = editor::process_video_pipeline_with_wav(
        &video_path, 
        &config,
        &sidecars.ffmpeg,
        &sidecars.ffprobe
    )?;
    
    *state.timeline.lock().unwrap() = Some(timeline.clone());
    *state.wav_path.lock().unwrap() = Some(wav_path);
    
    Ok(timeline)
}

/// Get waveform data for visualization
#[tauri::command]
fn get_waveform_data(state: State<AppState>) -> Result<WaveformData, String> {
    let wav_path = state.wav_path.lock().unwrap();
    
    if let Some(ref path) = *wav_path {
        // 10ms buckets for smooth waveform
        analysis::extract_waveform(path, 10)
    } else {
        Err("No audio data available".to_string())
    }
}

/// Get segment at a specific source time (for waveform click)
#[tauri::command]
fn get_segment_at_time(source_time: f64, state: State<AppState>) -> Result<usize, String> {
    let timeline = state.timeline.lock().unwrap();
    
    if let Some(ref timeline) = *timeline {
        for (i, clip) in timeline.clips.iter().enumerate() {
            if source_time >= clip.source_start && source_time < clip.source_end {
                return Ok(i);
            }
        }
        Err("Time not within any segment".to_string())
    } else {
        Err("No timeline loaded".to_string())
    }
}

/// Sets the cut softness percentage and updates the timeline
#[tauri::command]
fn set_cut_softness(percent: u8, state: State<AppState>) -> Result<Timeline, String> {
    let mut timeline_opt = state.timeline.lock().unwrap();
    
    if let Some(ref mut timeline) = *timeline_opt {
        timeline.apply_softness(percent);
        Ok(timeline.clone())
    } else {
        Err("No timeline loaded".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::init();
    
    log::info!("Starting Segment Review Tool");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            timeline: Mutex::new(None),
            silence_config: Mutex::new(SilenceDetectionConfig::default()),
            wav_path: Mutex::new(None),
        })
        .manage(SidecarPaths {
            ffmpeg: resolve_sidecar_path("ffmpeg"),
            ffprobe: resolve_sidecar_path("ffprobe"),
        })
        .invoke_handler(tauri::generate_handler![
            process_video,
            get_timeline,
            delete_silence_clips,
            export_video,
            update_silence_config,
            get_silence_config,
            get_video_path,
            // New segment review commands
            toggle_segment,
            remove_segment,
            merge_segments,
            adjust_segment_boundary,
            rerun_analysis,
            get_waveform_data,
            get_segment_at_time,
            set_cut_softness,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

