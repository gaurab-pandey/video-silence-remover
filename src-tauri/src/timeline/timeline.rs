use super::clip::Clip;
use serde::{Deserialize, Serialize};

/// Manages the timeline of video clips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub clips: Vec<Clip>,
    pub total_duration: f64,
    pub video_path: String,
    pub audio_path: Option<String>,
    pub raw_silence_ranges: Vec<(f64, f64)>,
}

impl Timeline {
    /// Creates a new timeline with a single clip spanning the entire video
    pub fn new(video_duration: f64, video_path: String) -> Self {
        let initial_clip = Clip::new(0.0, video_duration, false);
        Timeline {
            clips: vec![initial_clip],
            total_duration: video_duration,
            video_path,
            audio_path: None,
            raw_silence_ranges: Vec::new(),
        }
    }

    /// Splits the timeline based on detected silence ranges
    pub fn split_by_silence(&mut self, silence_ranges: Vec<(f64, f64)>) {
        self.raw_silence_ranges = silence_ranges.clone();
        self.apply_silence_splitting(silence_ranges);
    }

    /// Internal method to apply splitting logic
    fn apply_silence_splitting(&mut self, silence_ranges: Vec<(f64, f64)>) {
        log::info!("Splitting timeline with {} silence ranges", silence_ranges.len());
        
        // Sort silence ranges by start time
        let mut sorted_silence = silence_ranges.clone();
        sorted_silence.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Start with the original clips (or reset logic needed if re-applying?)
        // Assuming this is called on a fresh or properly prepared clip list.
        // But for `set_cut_softness` we will need to reset clips first.
        let mut current_clips = self.clips.clone();
        
        // For each silence range, split any overlapping clips
        for (silence_start, silence_end) in sorted_silence {
            let mut temp_clips = Vec::new();
            
            for clip in current_clips.iter() {
                if clip.is_silence {
                    // Already a silence clip, keep it
                    temp_clips.push(clip.clone());
                    continue;
                }
                
                // Check if this clip overlaps with the silence range
                if silence_end <= clip.source_start || silence_start >= clip.source_end {
                    // No overlap, keep the clip as is
                    temp_clips.push(clip.clone());
                } else {
                    // There's overlap, split the clip
                    
                    // Add the part before silence (if any)
                    if silence_start > clip.source_start {
                        temp_clips.push(Clip::new(clip.source_start, silence_start, false));
                    }
                    
                    // Add the silence part
                    let silence_clip_start = silence_start.max(clip.source_start);
                    let silence_clip_end = silence_end.min(clip.source_end);
                    temp_clips.push(Clip::new(silence_clip_start, silence_clip_end, true));
                    
                    // Add the part after silence (if any)
                    if silence_end < clip.source_end {
                        temp_clips.push(Clip::new(silence_end, clip.source_end, false));
                    }
                }
            }
            
            current_clips = temp_clips;
        }
        
        self.clips = current_clips;
        log::info!("Timeline now has {} clips after splitting", self.clips.len());
    }

    /// Removes all clips marked as silence
    pub fn delete_silence_clips(&mut self) {
        let before_count = self.clips.len();
        self.clips.retain(|clip| !clip.is_silence);
        let after_count = self.clips.len();
        log::info!("Deleted {} silence clips, {} clips remaining", 
                   before_count - after_count, after_count);
    }





    /// Recalculates timeline times for all clips
    /// Creates a contiguous timeline from potentially non-contiguous source clips
    /// MUST be called after deleting silence clips
    pub fn recalculate_timeline_times(&mut self) {
        let mut timeline_pos = 0.0;
        
        for clip in &mut self.clips {
            let duration = clip.duration();
            clip.timeline_start = timeline_pos;
            clip.timeline_end = timeline_pos + duration;
            timeline_pos = clip.timeline_end;
        }
        
        log::info!("Recalculated timeline: {} clips, total duration {:.2}s", 
                   self.clips.len(), timeline_pos);
    }







    /// Toggles the include state of a segment at the given index
    pub fn toggle_segment_include(&mut self, index: usize) -> Result<(), String> {
        if let Some(clip) = self.clips.get_mut(index) {
            clip.include = !clip.include;
            log::info!("Toggled segment {} include to {}", index, clip.include);
            Ok(())
        } else {
            Err(format!("Segment index {} out of bounds", index))
        }
    }

    /// Removes a segment at the given index
    pub fn remove_segment(&mut self, index: usize) -> Result<(), String> {
        if index >= self.clips.len() {
            return Err(format!("Segment index {} out of bounds", index));
        }
        self.clips.remove(index);
        self.recalculate_timeline_times();
        log::info!("Removed segment {}, {} segments remaining", index, self.clips.len());
        Ok(())
    }

    /// Merges segment at index with the next segment
    /// The resulting segment takes the type and include state of the first segment
    pub fn merge_segments(&mut self, index: usize) -> Result<(), String> {
        if index >= self.clips.len() {
            return Err(format!("Segment index {} out of bounds", index));
        }
        if index + 1 >= self.clips.len() {
            return Err("Cannot merge: no next segment".to_string());
        }

        let next_clip = self.clips.remove(index + 1);
        if let Some(clip) = self.clips.get_mut(index) {
            clip.source_end = next_clip.source_end;
            clip.timeline_end = clip.timeline_start + clip.duration();
        }
        self.recalculate_timeline_times();
        log::info!("Merged segments {} and {}, {} segments remaining", index, index + 1, self.clips.len());
        Ok(())
    }

    /// Adjusts the boundary between segment `index` and `index + 1`
    /// Updates source_end of `clips[index]` and source_start of `clips[index+1]`
    pub fn adjust_segment_boundary(&mut self, index: usize, new_time: f64) -> Result<(), String> {
        if index + 1 >= self.clips.len() {
            return Err("Cannot adjust boundary: no next segment".to_string());
        }

        // Validate new_time is within reasonable bounds (not before start of current, not after end of next)
        // We use a small epsilon or let strict validation happen in Clip
        let current_start = self.clips[index].source_start;
        let next_end = self.clips[index+1].source_end;

        if new_time <= current_start || new_time >= next_end {
            return Err(format!("New time {:.2} is out of bounds ({:.2} - {:.2})", new_time, current_start, next_end));
        }

        self.clips[index].source_end = new_time;
        self.clips[index+1].source_start = new_time;

        self.recalculate_timeline_times();
        log::info!("Adjusted boundary between {} and {} to {:.2}", index, index+1, new_time);
        Ok(())
    }

    /// Returns the duration of included segments only (for export preview)


    /// Applies cut softness by modifying silence ranges and re-splitting the timeline
    pub fn apply_softness(&mut self, softness_percent: u8) {
        log::info!("Applying cut softness: {}%", softness_percent);

        // 1. Reset clips to original full clip
        self.clips = vec![Clip::new(0.0, self.total_duration, false)];
        
        let softness_ratio = softness_percent as f64 / 100.0;
        let mut effective_silence_ranges = Vec::new();

        for &(start, end) in &self.raw_silence_ranges {
            let duration = end - start;
            
            if softness_ratio <= 0.0 {
                effective_silence_ranges.push((start, end));
                continue;
            }

            // Calculate preserved silence time
            let preserved_time = duration * softness_ratio;
            
            // Split preserved time directionally (40% pre, 60% post)
            let raw_pre = preserved_time * 0.4;
            let raw_post = preserved_time * 0.6;
            
            // Clamp padding values
            let pre_padding = raw_pre.clamp(0.03, 0.2); // 30ms to 200ms
            let post_padding = raw_post.clamp(0.05, 0.3); // 50ms to 300ms
            
            // Calculate new cut window (shrunken silence)
            // Silence is what we REMOVE (mostly), or mark as silence.
            // "Preserving padding" means the Silence Range gets SMALLER.
            // Original Silence: [Start, End]
            // Content expands into it.
            // New Silence Start = Start + pre_padding
            // New Silence End = End - post_padding
            
            let mut new_start = start + pre_padding;
            let mut new_end = end - post_padding;
            
            // Constraints: ensure we don't flip the range if padding > duration?
            // "Padding must never remove non-silent speech" - inherently safe as we only shrink silence.
            // But if padding consumes entire silence, then NO silence clip is created.
            
            if new_start >= new_end {
                // Preserved amount covers the whole silence?
                // Then this silence is skipped entirely (becomes content)
                continue; 
            }
            
            effective_silence_ranges.push((new_start, new_end));
        }
        
        // 2. Re-apply splitting with new ranges
        self.apply_silence_splitting(effective_silence_ranges);
        
        // 3. Recalculate timeline times
        self.recalculate_timeline_times();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new(10.0, "test.mp4".to_string());
        assert_eq!(timeline.clips.len(), 1);
        assert_eq!(timeline.total_duration, 10.0);
        assert_eq!(timeline.clips[0].source_start, 0.0);
        assert_eq!(timeline.clips[0].source_end, 10.0);
    }

    #[test]
    fn test_split_by_silence() {
        let mut timeline = Timeline::new(10.0, "test.mp4".to_string());
        
        // Add silence from 3.0 to 5.0
        timeline.split_by_silence(vec![(3.0, 5.0)]);
        
        // Should have 3 clips: [0-3], [3-5 silence], [5-10]
        assert_eq!(timeline.clips.len(), 3);
        assert_eq!(timeline.clips[0].is_silence, false);
        assert_eq!(timeline.clips[1].is_silence, true);
        assert_eq!(timeline.clips[2].is_silence, false);
    }

    #[test]
    fn test_delete_silence() {
        let mut timeline = Timeline::new(10.0, "test.mp4".to_string());
        timeline.split_by_silence(vec![(3.0, 5.0)]);
        
        timeline.delete_silence_clips();
        
        // Should have 2 clips left
        assert_eq!(timeline.clips.len(), 2);
        assert!(timeline.clips.iter().all(|c| !c.is_silence));
    }
}
