//! Real-time score tracking during gameplay

use super::{streak::StreakMilestone, streak::StreakTracker, timing::TimingQuality};

/// Real-time score tracker for gameplay
#[derive(Debug, Clone)]
pub struct LiveScoreTracker {
    current_score: u64,
    streak: StreakTracker,
    perfect_count: u32,
    good_count: u32,
    okay_count: u32,
    miss_count: u32,
    attempted_notes: u32,
    total_song_notes: u32,
}

impl LiveScoreTracker {
    pub fn new() -> Self {
        Self {
            current_score: 0,
            streak: StreakTracker::new(),
            perfect_count: 0,
            good_count: 0,
            okay_count: 0,
            miss_count: 0,
            attempted_notes: 0,
            total_song_notes: 0,
        }
    }

    /// Initialize with total notes in the song
    pub fn with_total_notes(mut self, total: u32) -> Self {
        self.total_song_notes = total;
        self
    }

    /// Get current score
    pub fn score(&self) -> u64 {
        self.current_score
    }

    /// Get current streak
    pub fn streak(&self) -> &StreakTracker {
        &self.streak
    }

    /// Get current multiplier
    pub fn multiplier(&self) -> u32 {
        self.streak.multiplier()
    }

    /// Get perfect hit count
    pub fn perfect_count(&self) -> u32 {
        self.perfect_count
    }

    /// Get good hit count
    pub fn good_count(&self) -> u32 {
        self.good_count
    }

    /// Get okay hit count
    pub fn okay_count(&self) -> u32 {
        self.okay_count
    }

    /// Get miss count
    pub fn miss_count(&self) -> u32 {
        self.miss_count
    }

    /// Get attempted notes count
    pub fn attempted_notes(&self) -> u32 {
        self.attempted_notes
    }

    /// Get total notes in the song
    pub fn total_song_notes(&self) -> u32 {
        self.total_song_notes
    }

    /// Calculate accuracy percentage (0.0 - 100.0)
    /// Based on correct notes / total notes in song
    pub fn accuracy(&self) -> f64 {
        if self.total_song_notes == 0 {
            return 0.0;
        }
        let correct = self.perfect_count + self.good_count + self.okay_count;
        (correct as f64 / self.total_song_notes as f64) * 100.0
    }

    /// Process a note hit with given timing quality
    /// Returns (points_earned, optional_milestone)
    pub fn on_note_hit(&mut self, quality: TimingQuality) -> (u64, Option<StreakMilestone>) {
        self.attempted_notes += 1;

        match quality {
            TimingQuality::Perfect => self.perfect_count += 1,
            TimingQuality::Good => self.good_count += 1,
            TimingQuality::Okay => self.okay_count += 1,
            TimingQuality::Miss => self.miss_count += 1,
        }

        let base_points = quality.base_points();
        let points = base_points * self.streak.multiplier() as u64;
        self.current_score += points;

        let milestone = self.streak.update(quality);

        (points, milestone)
    }

    /// Reset all tracking (e.g., when restarting)
    pub fn reset(&mut self) {
        self.current_score = 0;
        self.streak = StreakTracker::new();
        self.perfect_count = 0;
        self.good_count = 0;
        self.okay_count = 0;
        self.miss_count = 0;
        self.attempted_notes = 0;
    }

    /// Convert to final score data for display
    pub fn to_score_data(&self) -> super::stars::ScoreResult {
        let accuracy = self.accuracy();
        let stars = super::stars::StarRating::calculate(accuracy, self.streak.max());

        super::stars::ScoreResult {
            score: self.current_score,
            accuracy,
            max_streak: self.streak.max(),
            stars,
            perfect_count: self.perfect_count,
            good_count: self.good_count,
            okay_count: self.okay_count,
            miss_count: self.miss_count,
            total_notes: self.attempted_notes,
        }
    }
}

impl Default for LiveScoreTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_accumulation() {
        let mut tracker = LiveScoreTracker::new().with_total_notes(100);

        // Perfect with 1x multiplier
        let (points, _) = tracker.on_note_hit(TimingQuality::Perfect);
        assert_eq!(points, 50);
        assert_eq!(tracker.score(), 50);

        // Good with 1x multiplier
        let (points, _) = tracker.on_note_hit(TimingQuality::Good);
        assert_eq!(points, 25);
        assert_eq!(tracker.score(), 75);
    }

    #[test]
    fn test_multiplier_effect() {
        let mut tracker = LiveScoreTracker::new().with_total_notes(100);

        // Build streak to 10 for 2x multiplier
        for _ in 0..9 {
            tracker.on_note_hit(TimingQuality::Perfect);
        }

        // 10th hit should have 2x multiplier
        let (points, milestone) = tracker.on_note_hit(TimingQuality::Perfect);
        assert_eq!(points, 100); // 50 * 2
        assert_eq!(milestone, Some(StreakMilestone::Multiplier2x));
    }

    #[test]
    fn test_accuracy_calculation() {
        let mut tracker = LiveScoreTracker::new().with_total_notes(3);

        tracker.on_note_hit(TimingQuality::Perfect);
        tracker.on_note_hit(TimingQuality::Good);
        tracker.on_note_hit(TimingQuality::Miss);

        // 2 correct out of 3 total = 66.67%
        let accuracy = tracker.accuracy();
        assert!((accuracy - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_reset() {
        let mut tracker = LiveScoreTracker::new().with_total_notes(100);

        tracker.on_note_hit(TimingQuality::Perfect);
        tracker.on_note_hit(TimingQuality::Perfect);
        assert_eq!(tracker.score(), 100);
        assert_eq!(tracker.streak().current(), 2);

        tracker.reset();
        assert_eq!(tracker.score(), 0);
        assert_eq!(tracker.streak().current(), 0);
        assert_eq!(tracker.attempted_notes(), 0);
    }
}
