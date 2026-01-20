import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./globals.css";

// ============= TYPES =============
interface Clip {
  timeline_start: number;
  timeline_end: number;
  source_start: number;
  source_end: number;
  is_silence: boolean;
  include: boolean;
}

interface Timeline {
  clips: Clip[];
  total_duration: number;
  video_path: string;
  audio_path?: string;
}

interface WaveformData {
  peaks: number[];
  duration: number;
  bucket_ms: number;
}

// ============= STATE =============
// ============= STATE =============
let currentTimeline: Timeline | null = null;
let waveformData: WaveformData | null = null;
let currentSegmentIndex: number | null = null;
let isPlaying = false;
let segmentEndTime: number | null = null;
// Zoom: pixels per second. Start at a reasonable default.
let zoomLevel = 50;
const MIN_ZOOM = 5;
const MAX_ZOOM = 200;

// ============= DOM ELEMENTS =============
let importBtn: HTMLButtonElement;
let exportBtn: HTMLButtonElement;
let rerunAnalysisBtn: HTMLButtonElement;
let thresholdInput: HTMLInputElement;
let thresholdValue: HTMLElement;
let durationInput: HTMLInputElement;
let durationValue: HTMLElement;
let softnessInput: HTMLInputElement;
let softnessValue: HTMLElement;

let audioPlayer: HTMLAudioElement;
let playPauseBtn: HTMLButtonElement;
let prevSegmentBtn: HTMLButtonElement;
let nextSegmentBtn: HTMLButtonElement;
let currentTimeDisplay: HTMLElement;
let totalTimeDisplay: HTMLElement;
let currentSegmentLabel: HTMLElement;
let waveformCanvas: HTMLCanvasElement;
let waveformEmpty: HTMLElement;
let segmentList: HTMLElement;
let segmentCountDisplay: HTMLElement;
let statusArea: HTMLElement;
let videoInfo: HTMLElement;
let videoPathDisplay: HTMLElement;
let videoDurationDisplay: HTMLElement;
let clipsCountDisplay: HTMLElement;
let includedDurationDisplay: HTMLElement;
let exportInfo: HTMLElement;
let zoomInBtn: HTMLButtonElement;
let zoomOutBtn: HTMLButtonElement;
let timelineTooltip: HTMLElement;
let waveformContainer: HTMLElement;

let waveformPanel: HTMLElement;

let resizer2: HTMLElement;
let isResizing = false;
let currentResizerId: string | null = null;
let startY = 0;
let startHeights = { waveform: 0 };

// Boundary Dragging State
let isDraggingBoundary = false;
let draggedBoundaryIndex: number | null = null; // Index of the clip *before* the boundary
let hoverBoundaryIndex: number | null = null;
const BOUNDARY_CLICK_THRESHOLD_PX = 15;

// ============= INITIALIZATION =============
window.addEventListener("DOMContentLoaded", async () => {
  // Helper to safely get element
  const getEl = <T extends HTMLElement>(id: string): T | null => {
    const el = document.getElementById(id) as T;
    // Don't warn for optional elements if you expect them to be missing sometimes
    // if (!el) console.warn(`DOM Element not found: #${id}`);
    return el;
  };

  // Get DOM elements
  importBtn = getEl("import-btn") as HTMLButtonElement;
  exportBtn = getEl("export-btn") as HTMLButtonElement;
  rerunAnalysisBtn = getEl("rerun-analysis-btn") as HTMLButtonElement;
  thresholdInput = getEl("threshold-input") as HTMLInputElement;
  thresholdValue = getEl("threshold-value") as HTMLElement;
  durationInput = getEl("duration-input") as HTMLInputElement;
  durationValue = getEl("duration-value") as HTMLElement;
  softnessInput = getEl("softness-input") as HTMLInputElement;
  softnessValue = getEl("softness-value") as HTMLElement;

  playPauseBtn = getEl("play-pause-btn") as HTMLButtonElement;
  prevSegmentBtn = getEl("prev-segment-btn") as HTMLButtonElement;
  nextSegmentBtn = getEl("next-segment-btn") as HTMLButtonElement;
  currentTimeDisplay = getEl("current-time") as HTMLElement;
  totalTimeDisplay = getEl("total-time") as HTMLElement;
  currentSegmentLabel = getEl("current-segment-label") as HTMLElement;
  waveformCanvas = getEl("waveform-canvas") as HTMLCanvasElement;
  if (waveformCanvas) {
    waveformCanvas.addEventListener("mousedown", onWaveformMouseDown);
  }
  waveformEmpty = getEl("waveform-empty") as HTMLElement;
  segmentList = getEl("segment-list") as HTMLElement;
  segmentCountDisplay = getEl("segment-count") as HTMLElement;
  statusArea = getEl("status-area") as HTMLElement;
  videoInfo = getEl("video-info") as HTMLElement;
  videoPathDisplay = getEl("video-path-display") as HTMLElement;
  videoDurationDisplay = getEl("video-duration-display") as HTMLElement;
  clipsCountDisplay = getEl("video-count-display") as HTMLElement;
  includedDurationDisplay = getEl("included-duration") as HTMLElement;
  exportInfo = getEl("export-info") as HTMLElement;
  zoomInBtn = getEl("zoom-in-btn") as HTMLButtonElement;
  zoomOutBtn = getEl("zoom-out-btn") as HTMLButtonElement;
  timelineTooltip = getEl("timeline-tooltip") as HTMLElement;
  waveformContainer = waveformCanvas?.parentElement as HTMLElement;

  waveformPanel = document.querySelector(".waveform-panel") as HTMLElement;

  resizer2 = getEl("resizer-2") as HTMLElement;

  // Initialize Audio Player
  audioPlayer = new Audio();
  audioPlayer.preload = "auto";

  // Audio Events (Logic Driver)
  audioPlayer.addEventListener("timeupdate", onAudioTimeUpdate);
  audioPlayer.addEventListener("ended", onAudioEnded);
  audioPlayer.addEventListener("play", () => {
    playPauseBtn.textContent = "⏸";
    isPlaying = true;
  });
  audioPlayer.addEventListener("pause", () => {
    playPauseBtn.textContent = "▶";
    isPlaying = false;
  });

  // Resizer listeners
  // if (resizer1) resizer1.addEventListener("mousedown", (e) => startResize(e, "resizer-1"));
  if (resizer2) resizer2.addEventListener("mousedown", (e) => startResize(e, "resizer-2"));

  window.addEventListener("mousemove", (e) => {
    resize(e);
    onBoundaryDrag(e);
  });
  window.addEventListener("mouseup", () => {
    stopResize();
    stopBoundaryDrag();
  });

  // Event listeners (with null checks)
  if (importBtn) importBtn.addEventListener("click", importVideo);
  if (exportBtn) exportBtn.addEventListener("click", exportVideo);
  if (rerunAnalysisBtn) rerunAnalysisBtn.addEventListener("click", rerunAnalysis);
  if (playPauseBtn) playPauseBtn.addEventListener("click", togglePlayPause);
  if (prevSegmentBtn) prevSegmentBtn.addEventListener("click", () => jumpToSegment(-1));
  if (nextSegmentBtn) nextSegmentBtn.addEventListener("click", () => jumpToSegment(1));

  // Zoom listeners
  if (zoomInBtn) zoomInBtn.addEventListener("click", () => updateZoom(1.5));
  if (zoomOutBtn) zoomOutBtn.addEventListener("click", () => updateZoom(1 / 1.5));

  // Slider event listeners
  if (thresholdInput && thresholdValue) {
    thresholdInput.addEventListener("input", () => {
      thresholdValue.textContent = `${thresholdInput.value} dB`;
    });
  }
  if (durationInput && durationValue) {
    durationInput.addEventListener("input", () => {
      durationValue.textContent = `${durationInput.value} s`;
    });
  }

  // Softness Slider
  if (softnessInput && softnessValue) {
    softnessInput.addEventListener("input", async () => {
      const val = softnessInput.value;
      softnessValue.textContent = `${val}%`;

      if (currentTimeline) {
        try {
          const updatedTimeline = await invoke<Timeline>("set_cut_softness", { percent: parseInt(val) });
          currentTimeline = updatedTimeline;
          renderWaveform();
          renderSegmentList();
          currentTimeline = updatedTimeline;
          renderWaveform();
          renderSegmentList();
          updateExportInfo();
        } catch (err) {
          console.error("Cut softness error:", err);
        }
      }
    });
  }

  // Waveform handling
  if (waveformCanvas) {
    waveformCanvas.addEventListener("click", onWaveformClick);
    waveformCanvas.addEventListener("mousemove", onWaveformMouseMove);
    waveformCanvas.addEventListener("mouseleave", () => {
      if (timelineTooltip) timelineTooltip.style.opacity = "0";
    });
  }

  // Segment list event delegation
  if (segmentList) {
    segmentList.addEventListener("click", onSegmentAction);
    segmentList.addEventListener("change", onSegmentAction);
  }

  // Load settings
  await loadSettings();
  updateStatus("Ready", "");
});

// ============= IMPORT & PROCESS =============
async function importVideo() {
  try {
    const selected = await open({
      title: "Select Video File",
      multiple: false,
      filters: [
        {
          name: "Video",
          extensions: ["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm"],
        },
      ],
    });

    if (!selected) return;

    const videoPath = selected as string;
    updateStatus("Processing video...", "processing");
    if (importBtn) importBtn.disabled = true;

    // Process video
    const timeline = await invoke<Timeline>("process_video", { videoPath });
    currentTimeline = timeline;

    // Auto-fit zoom to initially see the whole video
    if (currentTimeline && currentTimeline.total_duration > 0 && waveformContainer) {
      // Calculate needed zoom to fit width
      const containerWidth = waveformContainer.clientWidth;
      zoomLevel = containerWidth / currentTimeline.total_duration;
      // Clamp it
      zoomLevel = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel));
    }

    // Load audio preview
    await loadAudioPreview();

    // Get waveform data
    try {
      waveformData = await invoke<WaveformData>("get_waveform_data");
      renderWaveform();
    } catch (e) {
      console.error("Failed to get waveform:", e);
    }

    // Update UI
    renderSegmentList();
    displayVideoInfo();
    enableControls();

    updateStatus("Video processed successfully", "success");
  } catch (error) {
    updateStatus(`Error: ${error}`, "error");
  } finally {
    if (importBtn) importBtn.disabled = false;
  }
}

// ============= SETTINGS & ANALYSIS =============
async function loadSettings() {
  try {
    const config = await invoke<{ threshold_db: number; min_silence_duration: number }>("get_silence_config");
    thresholdInput.value = String(config.threshold_db);
    thresholdValue.textContent = `${config.threshold_db} dB`;
    durationInput.value = String(config.min_silence_duration);
    durationValue.textContent = `${config.min_silence_duration} s`;
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

async function rerunAnalysis() {
  if (!currentTimeline) return;

  try {
    updateStatus("Re-running analysis...", "processing");
    if (rerunAnalysisBtn) rerunAnalysisBtn.disabled = true;

    // Update config first
    await invoke("update_silence_config", {
      thresholdDb: parseFloat(thresholdInput.value),
      minSilenceDuration: parseFloat(durationInput.value),
    });

    // Re-run analysis
    const timeline = await invoke<Timeline>("rerun_analysis");
    currentTimeline = timeline;

    // Reload waveform
    try {
      waveformData = await invoke<WaveformData>("get_waveform_data");
      renderWaveform();
    } catch (e) {
      console.error("Failed to get waveform:", e);
    }

    // Update UI
    renderSegmentList();
    displayVideoInfo();
    currentSegmentIndex = null;
    updateSegmentLabel();

    updateStatus("Analysis complete", "success");
  } catch (error) {
    updateStatus(`Error: ${error}`, "error");
  } finally {
    if (rerunAnalysisBtn) rerunAnalysisBtn.disabled = false;
  }
} // End rerunAnalysis

// ============= AUDIO PREVIEW (Was Video) =============
async function loadAudioPreview() {
  try {
    // We don't load video anymore. Just audio.


    // Load Audio if available
    if (currentTimeline && currentTimeline.audio_path) {
      const { convertFileSrc } = await import("@tauri-apps/api/core");
      const audioUrl = convertFileSrc(currentTimeline.audio_path);
      audioPlayer.src = audioUrl;
      audioPlayer.load();
      console.log("Loaded Audio Source:", currentTimeline.audio_path);
    } else {
      console.warn("No audio path available in timeline");
    }

    totalTimeDisplay.textContent = formatTime(currentTimeline?.total_duration || 0);

    audioPlayer.currentTime = 0;
  } catch (error) {
    console.error("Failed to load audio:", error);
  }
}

function onAudioTimeUpdate() {
  const currentTime = audioPlayer.currentTime;
  currentTimeDisplay.textContent = formatTime(currentTime);

  // Stop at segment end if playing a segment preview
  if (segmentEndTime !== null && currentTime >= segmentEndTime) {
    pause();
    segmentEndTime = null;
    return;
  }

  // Live Playback Skipping Logic (Skip excluded/silence clips)
  if (currentTimeline && isPlaying && segmentEndTime === null) {
    // Find current clip
    let clipIndex = -1;
    for (let i = 0; i < currentTimeline.clips.length; i++) {
      const clip = currentTimeline.clips[i];
      if (currentTime >= clip.source_start && currentTime < clip.source_end) {
        clipIndex = i;
        break;
      }
    }

    if (clipIndex !== -1) {
      const currentClip = currentTimeline.clips[clipIndex];
      // If we are in a silence/excluded clip, jump to next content
      if (!currentClip.include) {
        // Find next included clip
        let nextContentTime = -1;
        for (let j = clipIndex + 1; j < currentTimeline.clips.length; j++) {
          if (currentTimeline.clips[j].include) {
            nextContentTime = currentTimeline.clips[j].source_start;
            break;
          }
        }

        if (nextContentTime !== -1) {
          audioPlayer.currentTime = nextContentTime;
        } else {
          // No more content? Stop?
        }
      }
    }
  }

  // Update waveform playhead
  renderWaveform();

  // Auto-scroll logic depending on playhead
  if (currentTimeline && zoomLevel > 20) {
    const px = Math.floor(audioPlayer.currentTime * zoomLevel);
    const containerWidth = waveformContainer.clientWidth;
    const scrollPos = waveformContainer.scrollLeft;

    // If playhead goes out of right side of view
    if (px > scrollPos + containerWidth * 0.9) {
      waveformContainer.scrollTo({ left: px - containerWidth * 0.1, behavior: 'auto' });
    }
  }
}

function onAudioEnded() {
  pause();
}


function togglePlayPause() {
  if (isPlaying) {
    pause();
  } else {
    play();
  }
}

function play() {
  if (!currentTimeline) return;
  isPlaying = true;
  playPauseBtn.textContent = "⏸";

  audioPlayer.play().catch(e => console.error("Play failed:", e));
}

function pause() {
  isPlaying = false;
  playPauseBtn.textContent = "▶";

  audioPlayer.pause();
  segmentEndTime = null;
}

function jumpToSegment(direction: number) {
  if (!currentTimeline || currentTimeline.clips.length === 0) return;

  if (currentSegmentIndex === null) {
    currentSegmentIndex = direction > 0 ? 0 : currentTimeline.clips.length - 1;
  } else {
    currentSegmentIndex = Math.max(0, Math.min(currentTimeline.clips.length - 1, currentSegmentIndex + direction));
  }

  seekToSegment(currentSegmentIndex);
}

function seekToSegment(index: number) {
  if (!currentTimeline || index < 0 || index >= currentTimeline.clips.length) return;

  currentSegmentIndex = index;
  const clip = currentTimeline.clips[index];

  audioPlayer.currentTime = clip.source_start;

  updateSegmentLabel();
  highlightSegmentRow(index);
}

function previewSegment(index: number) {
  if (!currentTimeline || index < 0 || index >= currentTimeline.clips.length) return;

  const clip = currentTimeline.clips[index];
  currentSegmentIndex = index;
  // previewVideo.currentTime = clip.source_start; // REMOVED
  audioPlayer.currentTime = clip.source_start;
  segmentEndTime = clip.source_end;

  updateSegmentLabel();
  highlightSegmentRow(index);
  play();
}



// ============= RESIZING LOGIC =============
function startResize(e: MouseEvent, resizerId: string) {
  isResizing = true;
  currentResizerId = resizerId;
  startY = e.clientY;
  document.body.style.cursor = "row-resize";

  // Capture start heights

  startHeights.waveform = waveformPanel.getBoundingClientRect().height;
}

function resize(e: MouseEvent) {
  if (!isResizing || !currentResizerId) return;

  const dy = e.clientY - startY;

  if (currentResizerId === "resizer-2") {
    // Resizer 2: Waveform vs Segment List (flex)
    // Drag down: Waveform grows, Segment List shrinks
    const newHeight = Math.max(100, startHeights.waveform + dy);
    waveformPanel.style.height = `${newHeight}px`;

    // Defer waveform redraw slightly to avoid thrashing, or just do it
    requestAnimationFrame(renderWaveform);
  }
}

function stopResize() {
  if (isResizing) {
    isResizing = false;
    currentResizerId = null;
    document.body.style.cursor = "";
    // Final render
    renderWaveform();
  }
}

// ============= ZOOM & WAVEFORM RENDERING =============
function updateZoom(factor: number) {
  if (!currentTimeline) return;

  // Calculate new zoom
  let newZoom = zoomLevel * factor;
  newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newZoom));

  if (newZoom !== zoomLevel) {
    zoomLevel = newZoom;
    renderWaveform();
  }
}

function renderWaveform() {
  if (!waveformData || !currentTimeline) {
    if (waveformEmpty) waveformEmpty.style.display = "flex";
    return;
  }

  if (waveformEmpty) waveformEmpty.style.display = "none";

  const ctx = waveformCanvas.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;

  // Calculate width based on zoom
  const totalDuration = currentTimeline.total_duration;
  const rawWidth = Math.max(waveformContainer.clientWidth, totalDuration * zoomLevel);
  // Dynamic height from container
  const containerHeight = waveformContainer.clientHeight;
  const height = containerHeight;

  waveformCanvas.style.height = `${height}px`; // ensure style matches
  waveformCanvas.height = height * dpr;

  waveformCanvas.width = rawWidth * dpr;
  waveformCanvas.style.width = `${rawWidth}px`;

  ctx.scale(dpr, dpr);

  // Clear
  ctx.fillStyle = "#1B1B1B";
  ctx.fillRect(0, 0, rawWidth, height);

  // Draw Regions (Bottom Layer)
  // Continuous strip - no gaps between clips
  for (const clip of currentTimeline.clips) {
    const x1 = clip.source_start * zoomLevel;
    const x2 = clip.source_end * zoomLevel;
    const w = Math.max(1, x2 - x1);

    // Silence = Dimmed Grey (#2a2a2a)
    // Content = Bright Blue (#007AFF) -- reduced opacity for background? 
    // Wait, requirement: "Content regions = bright blue". 
    // And "Thicker waveform (height ^ 40%)".

    ctx.fillStyle = clip.is_silence ? "#2a2a2a" : "#003366"; // Dark blue for content background
    ctx.fillRect(x1, 0, w, height);

    // Optional: Draw separator lines
    ctx.fillStyle = "#111";
    ctx.fillRect(x2, 0, 1, height);
  }

  // Draw Waveform (Middle Layer)
  const peaks = waveformData.peaks;
  const duration = currentTimeline.total_duration;
  const samplesPerSec = peaks.length / duration;

  // Create Gradient
  const gradient = ctx.createLinearGradient(0, 0, 0, height);
  gradient.addColorStop(0, "rgba(100, 200, 255, 0.8)"); // Light Blue Top
  gradient.addColorStop(0.5, "#3A7AFE");             // Primary Blue Middle
  gradient.addColorStop(1, "rgba(0, 51, 102, 0.8)");   // Dark Blue Bottom

  ctx.beginPath();
  ctx.strokeStyle = gradient;
  ctx.lineWidth = 1; // Keep thin for detail, or 1.5 for bolder
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  const centerY = height / 2;
  const ampScale = (height / 2) * 0.95; // 95% of half scale

  // We need to map pixels to peaks
  // x goes from 0 to rawWidth
  // time = x / zoomLevel
  // peakIndex = time * samplesPerSec

  // Optimization: Don't draw every pixel if zoom is low (skip)
  // Or don't draw every peak if zoom is high (interpolate) -- canvas lineTo handles interpolation

  ctx.beginPath();
  for (let x = 0; x < rawWidth; x += 2) { // Step 2 for perf
    const time = x / zoomLevel;
    const peakIdx = Math.floor(time * samplesPerSec);
    if (peakIdx >= 0 && peakIdx < peaks.length) {
      const val = peaks[peakIdx];
      // Increase height by ~80% (scale by 1.8) for better visibility
      const h = Math.min(1.0, val * 1.8) * ampScale;
      ctx.moveTo(x, centerY - h);
      ctx.lineTo(x, centerY + h);
    }
  }
  ctx.stroke();

  // Draw Regions overlay for color coding waveform? 
  // Requirement: "Content regions = bright blue".
  // Maybe they meant the waveform itself should be bright blue in content regions.
  // Let's iterate clips again and overlay colors with composite operation?
  // Simpler: Just rely on background color distinction I did above.

  // Draw Playhead
  // Draw Playhead
  const playheadX = audioPlayer.currentTime * zoomLevel;
  ctx.strokeStyle = "#ff0000"; // Red playhead
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(playheadX, 0);
  ctx.lineTo(playheadX, height);
  ctx.stroke();

  // Draw Ruler (Top Layer)
  drawRuler(ctx, rawWidth);
}

function drawRuler(ctx: CanvasRenderingContext2D, width: number) {
  ctx.fillStyle = "rgba(255, 255, 255, 0.8)";
  ctx.font = "10px monospace";
  ctx.textAlign = "left";

  // Ruler background strip
  ctx.fillStyle = "rgba(0,0,0,0.3)";
  ctx.fillRect(0, 0, width, 20);

  ctx.strokeStyle = "rgba(255, 255, 255, 0.3)";
  ctx.lineWidth = 1;

  // Smart tick calculation
  // We want a label roughly every 80-100 pixels
  const minLabelPx = 80;
  const minStep = minLabelPx / zoomLevel;

  // Available nice steps
  const niceSteps = [0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
  let tickStep = niceSteps.find(s => s >= minStep) || 600;

  // Smaller ticks in between?
  const subTickStep = tickStep / 5;

  // Start from 0 to total duration
  const duration = currentTimeline?.total_duration || 0;

  ctx.beginPath();
  for (let t = 0; t <= duration; t += subTickStep) {
    const x = t * zoomLevel;
    if (x > width) break;

    // Is this a major tick?
    // Using epsilon for float comparison
    const isMajor = Math.abs(t % tickStep) < 0.001 || Math.abs((t % tickStep) - tickStep) < 0.001;

    if (isMajor) {
      // Major tick
      ctx.moveTo(x, 0);
      ctx.lineTo(x, 12);
      ctx.fillStyle = "#fff";
      ctx.fillText(formatTimeShort(t), x + 4, 14);
    } else {
      // Minor tick
      // Only draw if enough space (zoom level high enough)
      if (zoomLevel > 10) {
        ctx.moveTo(x, 0);
        ctx.lineTo(x, 6);
      }
    }
  }
  ctx.stroke();
}

function formatTimeShort(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

// ============= BOUNDARY DRAG LOGIC =============

function onWaveformMouseDown(e: MouseEvent) {
  if (hoverBoundaryIndex !== null) {
    isDraggingBoundary = true;
    draggedBoundaryIndex = hoverBoundaryIndex;
    document.body.style.userSelect = 'none';
    e.preventDefault();
    e.stopPropagation();
  }
}

function onBoundaryDrag(e: MouseEvent) {
  if (!isDraggingBoundary || draggedBoundaryIndex === null || !currentTimeline) return;

  const rect = waveformCanvas.getBoundingClientRect();
  let x = e.clientX - rect.left;

  // Convert x to time
  let newTime = x / zoomLevel;

  const prevClip = currentTimeline.clips[draggedBoundaryIndex];
  const nextClip = currentTimeline.clips[draggedBoundaryIndex + 1];

  if (prevClip && nextClip) {
    // Constrain locally
    const minTime = prevClip.source_start + 0.1;
    const maxTime = nextClip.source_end - 0.1;
    newTime = Math.max(minTime, Math.min(newTime, maxTime));

    // Update local state visually
    prevClip.source_end = newTime;
    nextClip.source_start = newTime;

    requestAnimationFrame(renderWaveform);
  }
}

async function stopBoundaryDrag() {
  if (isDraggingBoundary && draggedBoundaryIndex !== null && currentTimeline) {
    isDraggingBoundary = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';

    const index = draggedBoundaryIndex;
    const clip = currentTimeline.clips[index];
    const newTime = clip.source_end;

    draggedBoundaryIndex = null;

    try {
      const updatedTimeline = await invoke<Timeline>("adjust_segment_boundary", {
        index: index,
        newTime: newTime
      });
      currentTimeline = updatedTimeline;
      renderWaveform();
    } catch (e) {
      console.error("Failed to update boundary:", e);
      // Revert state if needed, or re-fetch timeline
      try {
        const timeline = await invoke<Timeline>("get_timeline");
        currentTimeline = timeline;
        renderWaveform();
      } catch (err) { console.error("Failed to refresh timeline", err); }
    }
  }
}

function onWaveformMouseMove(e: MouseEvent) {
  if (!currentTimeline) return;
  if (isDraggingBoundary) return;

  const rect = waveformCanvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const time = x / zoomLevel;

  // Boundary Detection
  let foundBoundary = null;

  for (let i = 0; i < currentTimeline.clips.length - 1; i++) {
    const clip = currentTimeline.clips[i];
    const boundaryTime = clip.source_end;
    const boundaryX = boundaryTime * zoomLevel;

    const diff = Math.abs(x - boundaryX);
    if (diff < BOUNDARY_CLICK_THRESHOLD_PX) {
      foundBoundary = i;
      break;
    }
  }

  if (foundBoundary !== null) {
    waveformCanvas.style.cursor = "col-resize";
    hoverBoundaryIndex = foundBoundary;
    timelineTooltip.style.display = 'none';
  } else {
    waveformCanvas.style.cursor = "crosshair";
    hoverBoundaryIndex = null;

    // Tooltip logic
    if (timelineTooltip) {
      timelineTooltip.style.opacity = "1";
      timelineTooltip.textContent = formatTimeShort(time);
      const containerRect = waveformContainer.getBoundingClientRect();
      const relativeX = e.clientX - containerRect.left;
      timelineTooltip.style.left = `${relativeX + 10}px`;
    }
  }
}

function onWaveformClick(e: MouseEvent) {
  if (!currentTimeline) return;

  const rect = waveformCanvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  // Calculate time based on zoom
  const clickTime = x / zoomLevel;

  // Use the exact time for seeking
  audioPlayer.currentTime = clickTime;

  // Find segment to highlight
  for (let i = 0; i < currentTimeline.clips.length; i++) {
    const clip = currentTimeline.clips[i];
    if (clickTime >= clip.source_start && clickTime < clip.source_end) {
      currentSegmentIndex = i;
      updateSegmentLabel();
      highlightSegmentRow(i);
      break;
    }
  }
}

// ============= SEGMENT LIST =============
function renderSegmentList() {
  if (!segmentList) return;

  if (!currentTimeline || currentTimeline.clips.length === 0) {
    segmentList.innerHTML = `
      <div class="segment-empty">
        <svg class="w-8 h-8 text-[#444] mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
        </svg>
        <p class="text-xs text-[#666]">No segments</p>
      </div>
    `;
    if (segmentCountDisplay) segmentCountDisplay.textContent = "0 segments";
    return;
  }

  if (segmentCountDisplay) segmentCountDisplay.textContent = `${currentTimeline.clips.length} segments`;

  let html = "";
  for (let i = 0; i < currentTimeline.clips.length; i++) {
    const clip = currentTimeline.clips[i];
    const type = clip.is_silence ? "silence" : "content";
    const duration = clip.source_end - clip.source_start;
    const isLast = i === currentTimeline.clips.length - 1;

    html += `
      <div class="segment-row" data-index="${i}">
        <input type="checkbox" class="segment-checkbox" ${clip.include ? "checked" : ""} data-action="toggle" data-index="${i}" />
        <span class="segment-type ${type}">${type}</span>
        <span class="segment-times">
          ${formatTime(clip.source_start)} → ${formatTime(clip.source_end)}
        </span>
        <span class="segment-duration">${duration.toFixed(2)}s</span>
        <div class="segment-actions">
          <button class="segment-action-btn preview" data-action="preview" data-index="${i}" title="Preview">▶</button>
          <button class="segment-action-btn remove" data-action="remove" data-index="${i}" title="Remove">✕</button>
          ${!isLast ? `<button class="segment-action-btn merge" data-action="merge" data-index="${i}" title="Merge with next">⊕</button>` : ""}
        </div>
      </div>
    `;
  }

  segmentList.innerHTML = html;
  updateIncludedDuration();
}

async function onSegmentAction(e: Event) {
  const target = e.target as HTMLElement;
  const action = target.dataset.action;
  const index = parseInt(target.dataset.index || "-1");

  if (index < 0 || !action) return;

  try {
    switch (action) {
      case "toggle":
        await invoke<Timeline>("toggle_segment", { index });
        // Refetch timeline
        currentTimeline = await invoke<Timeline>("get_timeline");
        updateIncludedDuration();
        break;

      case "preview":
        previewSegment(index);
        break;

      case "remove":
        currentTimeline = await invoke<Timeline>("remove_segment", { index });
        renderSegmentList();
        displayVideoInfo();
        renderWaveform();
        break;

      case "merge":
        currentTimeline = await invoke<Timeline>("merge_segments", { index });
        renderSegmentList();
        displayVideoInfo();
        renderWaveform();
        break;
    }
  } catch (error) {
    updateStatus(`Error: ${error}`, "error");
  }
}

function highlightSegmentRow(index: number) {
  // Remove existing highlights
  segmentList.querySelectorAll(".segment-row").forEach((row) => {
    row.classList.remove("playing");
  });

  // Add highlight to current
  const row = segmentList.querySelector(`[data-index="${index}"]`);
  if (row) {
    row.classList.add("playing");
    row.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }
}

function updateIncludedDuration() {
  if (!currentTimeline) return;

  const includedClips = currentTimeline.clips.filter(c => c.include);
  const totalDuration = includedClips.reduce((sum, c) => sum + (c.source_end - c.source_start), 0);

  if (includedDurationDisplay) {
    includedDurationDisplay.textContent = `${includedClips.length} segments · ${totalDuration.toFixed(2)}s`;
  }
  if (exportInfo) {
    exportInfo.style.display = "block";
  }
}

function updateSegmentLabel() {
  if (currentSegmentIndex === null || !currentTimeline) {
    currentSegmentLabel.textContent = "No segment";
    return;
  }

  const clip = currentTimeline.clips[currentSegmentIndex];
  const type = clip.is_silence ? "Silence" : "Content";
  currentSegmentLabel.textContent = `${type} ${currentSegmentIndex + 1}/${currentTimeline.clips.length}`;
}

// ============= VIDEO INFO =============
function displayVideoInfo() {
  if (!currentTimeline || !videoInfo) {
    if (videoInfo) videoInfo.style.display = "none";
    return;
  }

  const contentClips = currentTimeline.clips.filter((c) => !c.is_silence).length;
  const silenceClips = currentTimeline.clips.filter((c) => c.is_silence).length;
  // Included duration is calculated in updateIncludedDuration() called below

  videoPathDisplay.textContent = `File: ${currentTimeline.video_path.split("\\").pop()}`;
  videoDurationDisplay.textContent = `Duration: ${currentTimeline.total_duration.toFixed(2)}s`;
  clipsCountDisplay.textContent = `${contentClips} content, ${silenceClips} silence`;

  videoInfo.style.display = "block";
  updateIncludedDuration();
}

function enableControls() {
  if (exportBtn) exportBtn.disabled = false;
  if (rerunAnalysisBtn) rerunAnalysisBtn.disabled = false;
  if (playPauseBtn) playPauseBtn.disabled = false;
  // if (prevSegmentBtn) prevSegmentBtn.disabled = false; // Disabled by user request
  // if (nextSegmentBtn) nextSegmentBtn.disabled = false; // Disabled by user request
}

// ============= EXPORT =============
async function exportVideo() {
  try {
    const savePath = await save({
      title: "Save Edited Video",
      filters: [{ name: "Video", extensions: ["mp4"] }],
      defaultPath: "edited_video.mp4",
    });

    if (!savePath) return;

    const outputPath = savePath as string;
    updateStatus("Exporting video...", "processing");
    if (exportBtn) exportBtn.disabled = true;

    const progressContainer = document.getElementById("progress-container") as HTMLElement;
    const progressPercentage = document.getElementById("progress-percentage") as HTMLElement;
    const progressBarFill = document.getElementById("progress-bar-fill") as HTMLElement;

    progressContainer.style.display = "flex";
    progressPercentage.textContent = "0%";
    progressBarFill.style.width = "0%";

    // Listen for progress
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<{ percentage: number; current_time: number; total_duration: number }>(
      "export-progress",
      (event) => {
        const progress = event.payload;
        const percentage = Math.round(progress.percentage);
        progressPercentage.textContent = `${percentage}%`;
        progressBarFill.style.width = `${percentage}%`;
      }
    );

    try {
      await invoke("export_video", { outputPath });
      progressPercentage.textContent = "100%";
      progressBarFill.style.width = "100%";
      updateStatus(`Exported to ${outputPath}`, "success");
    } finally {
      unlisten();
      setTimeout(() => {
        progressContainer.style.display = "none";
      }, 2000);
    }
  } catch (error) {
    updateStatus(`Export failed: ${error}`, "error");
    const progressContainer = document.getElementById("progress-container");
    if (progressContainer) progressContainer.style.display = "none";
  } finally {
    if (exportBtn) exportBtn.disabled = false;
  }
}

// ============= HELPER FUNCTIONS =============

function updateExportInfo() {
  if (!currentTimeline || !includedDurationDisplay || !exportInfo) return;

  const includedClips = currentTimeline.clips.filter(c => c.include);
  // Calculate total duration using simple arithmetic
  const duration = includedClips.reduce((acc, c) => acc + (c.source_end - c.source_start), 0.0);

  includedDurationDisplay.textContent = `${includedClips.length} segments · ${duration.toFixed(2)}s`;

  // Show the panel
  exportInfo.style.display = "block";
}

// ============= UTILITIES =============
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  const ms = Math.floor((seconds % 1) * 100);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
}

function updateStatus(message: string, type = "") {
  const statusText = statusArea.querySelector(".status-text");
  if (statusText) {
    statusText.textContent = message;
    statusText.className = `status-text ${type}`;
  }
}

// Handle window resize for waveform
window.addEventListener("resize", () => {
  // Re-render when resized
  if (waveformData && currentTimeline) {
    renderWaveform();
  }
});
