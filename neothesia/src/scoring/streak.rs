//! Streak/combo tracking system

use super::timing::TimingQuality;

/// Streak milestone events for visual/audio feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreakMilestone {
    /// 10 notes - 2x multiplier
    Multiplier2x,
    /// 30 notes - 4x multiplier
    Multiplier4x,
    /// 50 notes - 8x multiplier, screen flash
    Multiplier8x,
    /// 100 notes - "ON FIRE!" effect
    OnFire,
    /// 200 notes - "LEGENDARY!" effect
    Legendary,
}

impl StreakMilestone {
    /// Get milestone for a given streak count
    pub fn for_streak(streak: u32) -> Option<Self> {
        match streak {
            10 => Some(StreakMilestone::Multiplier2x),
            30 => Some(StreakMilestone::Multiplier4x),
            50 => Some(StreakMilestone::Multiplier8x),
            100 => Some(StreakMilestone::OnFire),
            200 => Some(StreakMilestone::Legendary),
            _ => None,
        }
    }

    /// Display text for the milestone
    pub fn display_text(&self) -> &'static str {
        match self {
            StreakMilestone::Multiplier2x => "2× MULTIPLIER!",
            StreakMilestone::Multiplier4x => "4× MULTIPLIER!",
            StreakMilestone::Multiplier8x => "8× MULTIPLIER!",
            StreakMilestone::OnFire => "ON FIRE!",
            StreakMilestone::Legendary => "LEGENDARY!",
        }
    }

    /// Screen flash intensity
    pub fn screen_flash_intensity(&self) -> f32 {
        match self {
            StreakMilestone::Multiplier2x => 0.0,
            StreakMilestone::Multiplier4x => 0.0,
            StreakMilestone::Multiplier8x => 0.3,
            StreakMilestone::OnFire => 0.5,
            StreakMilestone::Legendary => 0.8,
        }
    }

    /// Screen shake intensity
    pub fn screen_shake_intensity(&self) -> f32 {
        match self {
            StreakMilestone::Multiplier2x => 0.0,
            StreakMilestone::Multiplier4x => 0.0,
            StreakMilestone::Multiplier8x => 0.0,
            StreakMilestone::OnFire => 0.5,
            StreakMilestone::Legendary => 0.8,
        }
    }
}

/// Tracks streak state and multipliers
#[derive(Debug, Clone)]
pub struct StreakTracker {
    current_streak: u32,
    max_streak: u32,
    multiplier: u32,
}

impl StreakTracker {
    pub fn new() -> Self {
        Self {
            current_streak: 0,
            max_streak: 0,
            multiplier: 1,
        }
    }

    /// Get current streak count
    pub fn current(&self) -> u32 {
        self.current_streak
    }

    /// Get maximum streak achieved
    pub fn max(&self) -> u32 {
        self.max_streak
    }

    /// Get current multiplier (1x, 2x, 4x, 8x)
    pub fn multiplier(&self) -> u32 {
        self.multiplier
    }

    /// Calculate multiplier based on streak count
    fn calculate_multiplier(streak: u32) -> u32 {
        match streak {
            0..=9 => 1,
            10..=29 => 2,
            30..=49 => 4,
            _ => 8,
        }
    }

    /// Update streak based on timing quality
    /// Returns any milestone reached
    pub fn update(&mut self, quality: TimingQuality) -> Option<StreakMilestone> {
        if quality == TimingQuality::Miss {
            self.current_streak = 0;
            self.multiplier = 1;
            return None;
        }

        self.current_streak += 1;
        self.multiplier = Self::calculate_multiplier(self.current_streak);

        if self.current_streak > self.max_streak {
            self.max_streak = self.current_streak;
        }

        StreakMilestone::for_streak(self.current_streak)
    }

    /// Reset streak (e.g., when pausing)
    pub fn reset(&mut self) {
        self.current_streak = 0;
        self.multiplier = 1;
    }

    /// Check if "on fire" (100+ streak)
    pub fn is_on_fire(&self) -> bool {
        self.current_streak >= 100
    }

    /// Check if "legendary" (200+ streak)
    pub fn is_legendary(&self) -> bool {
        self.current_streak >= 200
    }
}

impl Default for StreakTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplier_calculation() {
        assert_eq!(StreakTracker::calculate_multiplier(0), 1);
        assert_eq!(StreakTracker::calculate_multiplier(5), 1);
        assert_eq!(StreakTracker::calculate_multiplier(10), 2);
        assert_eq!(StreakTracker::calculate_multiplier(25), 2);
        assert_eq!(StreakTracker::calculate_multiplier(30), 4);
        assert_eq!(StreakTracker::calculate_multiplier(49), 4);
        assert_eq!(StreakTracker::calculate_multiplier(50), 8);
        assert_eq!(StreakTracker::calculate_multiplier(100), 8);
    }

    #[test]
    fn test_streak_progression() {
        let mut tracker = StreakTracker::new();

        // Build streak
        for i in 0..9 {
            let milestone = tracker.update(TimingQuality::Perfect);
            assert!(milestone.is_none(), "No milestone at streak {}", i + 1);
            assert_eq!(tracker.current(), i + 1);
            assert_eq!(tracker.multiplier(), 1);
        }

        // Hit 10 - 2x multiplier
        let milestone = tracker.update(TimingQuality::Perfect);
        assert_eq!(milestone, Some(StreakMilestone::Multiplier2x));
        assert_eq!(tracker.current(), 10);
        assert_eq!(tracker.multiplier(), 2);

        // Miss resets
        tracker.update(TimingQuality::Miss);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.multiplier(), 1);
        assert_eq!(tracker.max(), 10); // Max preserved
    }

    #[test]
    fn test_milestones() {
        let mut tracker = StreakTracker::new();

        // Build to 30
        for _ in 0..29 {
            tracker.update(TimingQuality::Good);
        }
        let milestone = tracker.update(TimingQuality::Good);
        assert_eq!(milestone, Some(StreakMilestone::Multiplier4x));

        // Build to 50
        for _ in 30..49 {
            tracker.update(TimingQuality::Okay);
        }
        let milestone = tracker.update(TimingQuality::Okay);
        assert_eq!(milestone, Some(StreakMilestone::Multiplier8x));

        // Build to 100
        for _ in 50..99 {
            tracker.update(TimingQuality::Perfect);
        }
        let milestone = tracker.update(TimingQuality::Perfect);
        assert_eq!(milestone, Some(StreakMilestone::OnFire));
        assert!(tracker.is_on_fire());

        // Build to 200
        for _ in 100..199 {
            tracker.update(TimingQuality::Good);
        }
        let milestone = tracker.update(TimingQuality::Good);
        assert_eq!(milestone, Some(StreakMilestone::Legendary));
        assert!(tracker.is_legendary());
    }
}
