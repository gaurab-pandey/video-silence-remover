use serde::{Deserialize, Serialize};

/// Represents a clip segment on the timeline
/// After silence deletion, timeline is contiguous but source has gaps.
/// Each clip maps timeline time to source video time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Start time in the playback timeline (contiguous)
    pub timeline_start: f64,
    /// End time in the playback timeline (contiguous)
    pub timeline_end: f64,
    /// Start time in the source video file
    pub source_start: f64,
    /// End time in the source video file
    pub source_end: f64,
    /// Whether this clip represents silence
    pub is_silence: bool,
    /// Whether to include this clip in export (checkbox state)
    pub include: bool,
}

impl Clip {
    /// Creates a new clip with both timeline and source times
    pub fn new(source_start: f64, source_end: f64, is_silence: bool) -> Self {
        // Initially, timeline and source times are the same
        // Content segments are included by default, silence excluded
        Clip {
            timeline_start: source_start,
            timeline_end: source_end,
            source_start,
            source_end,
            is_silence,
            include: !is_silence,
        }
    }

    /// Returns the duration of this clip in seconds
    pub fn duration(&self) -> f64 {
        self.source_end - self.source_start
    }

    /// Validates that the clip has valid time boundaries
    pub fn is_valid(&self) -> bool {
        self.source_start >= 0.0 
            && self.source_end > self.source_start
            && self.timeline_start >= 0.0
            && self.timeline_end > self.timeline_start
    }

    /// Checks if a timeline time falls within this clip
    pub fn contains_timeline_time(&self, time: f64) -> bool {
        time >= self.timeline_start && time < self.timeline_end
    }

    /// Converts a timeline time to source video time
    /// Assumes the time is within this clip's timeline range
    pub fn timeline_to_source(&self, timeline_time: f64) -> f64 {
        let offset = timeline_time - self.timeline_start;
        self.source_start + offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_creation() {
        let clip = Clip::new(0.0, 5.0, false);
        assert_eq!(clip.source_start, 0.0);
        assert_eq!(clip.source_end, 5.0);
        assert_eq!(clip.timeline_start, 0.0);
        assert_eq!(clip.timeline_end, 5.0);
        assert_eq!(clip.is_silence, false);
        assert_eq!(clip.include, true); // Content clips are included by default
        
        let silence_clip = Clip::new(0.0, 5.0, true);
        assert_eq!(silence_clip.is_silence, true);
        assert_eq!(silence_clip.include, false); // Silence clips are excluded by default
    }

    #[test]
    fn test_clip_duration() {
        let clip = Clip::new(2.5, 7.5, false);
        assert_eq!(clip.duration(), 5.0);
    }

    #[test]
    fn test_clip_validation() {
        let valid_clip = Clip::new(0.0, 1.0, false);
        assert!(valid_clip.is_valid());

        let mut invalid_clip = Clip::new(0.0, 1.0, false);
        invalid_clip.source_end = 0.0; // Invalid
        assert!(!invalid_clip.is_valid());
    }

    #[test]
    fn test_timeline_time_check() {
        let mut clip = Clip::new(10.0, 20.0, false);
        // Remap to timeline 0-10
        clip.timeline_start = 0.0;
        clip.timeline_end = 10.0;

        assert!(clip.contains_timeline_time(5.0));
        assert!(!clip.contains_timeline_time(15.0));
    }

    #[test]
    fn test_timeline_to_source_mapping() {
        let mut clip = Clip::new(10.0, 20.0, false);
        // Remap to timeline 0-10 (e.g., after deleting 0-10s of silence)
        clip.timeline_start = 0.0;
        clip.timeline_end = 10.0;

        // Timeline 5.0 should map to source 15.0
        assert_eq!(clip.timeline_to_source(5.0), 15.0);
    }
}



