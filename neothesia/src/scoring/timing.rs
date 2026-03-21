//! Timing quality evaluation for note hits

use std::time::Duration;

/// Quality of a note hit based on timing accuracy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingQuality {
    /// ≤50ms timing delta - perfect hit
    Perfect,
    /// ≤100ms timing delta - good hit
    Good,
    /// ≤200ms timing delta - acceptable hit
    Okay,
    /// >200ms or missed - poor timing
    Miss,
}

impl TimingQuality {
    /// Classify timing quality from a duration delta
    pub fn from_delta(delta: Duration) -> Self {
        let ms = delta.as_millis();
        if ms <= 50 {
            TimingQuality::Perfect
        } else if ms <= 100 {
            TimingQuality::Good
        } else if ms <= 200 {
            TimingQuality::Okay
        } else {
            TimingQuality::Miss
        }
    }

    /// Base points awarded for this timing quality
    pub fn base_points(&self) -> u64 {
        match self {
            TimingQuality::Perfect => 50,
            TimingQuality::Good => 25,
            TimingQuality::Okay => 10,
            TimingQuality::Miss => 0,
        }
    }

    /// Color for visual feedback (RGB normalized 0.0-1.0)
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            TimingQuality::Perfect => (1.0, 0.84, 0.0), // Gold
            TimingQuality::Good => (0.0, 1.0, 0.0),     // Green
            TimingQuality::Okay => (0.0, 0.53, 1.0),    // Blue
            TimingQuality::Miss => (1.0, 0.0, 0.0),     // Red
        }
    }

    /// Particle count for visual effect
    pub fn particle_count(&self) -> usize {
        match self {
            TimingQuality::Perfect => 20,
            TimingQuality::Good => 10,
            TimingQuality::Okay => 5,
            TimingQuality::Miss => 0,
        }
    }

    /// Display label
    pub fn label(&self) -> &'static str {
        match self {
            TimingQuality::Perfect => "PERFECT",
            TimingQuality::Good => "GOOD",
            TimingQuality::Okay => "OKAY",
            TimingQuality::Miss => "MISS",
        }
    }
}

impl std::fmt::Display for TimingQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_classification() {
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(30)),
            TimingQuality::Perfect
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(50)),
            TimingQuality::Perfect
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(75)),
            TimingQuality::Good
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(100)),
            TimingQuality::Good
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(150)),
            TimingQuality::Okay
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(200)),
            TimingQuality::Okay
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(250)),
            TimingQuality::Miss
        );
        assert_eq!(
            TimingQuality::from_delta(Duration::from_millis(500)),
            TimingQuality::Miss
        );
    }

    #[test]
    fn test_base_points() {
        assert_eq!(TimingQuality::Perfect.base_points(), 50);
        assert_eq!(TimingQuality::Good.base_points(), 25);
        assert_eq!(TimingQuality::Okay.base_points(), 10);
        assert_eq!(TimingQuality::Miss.base_points(), 0);
    }
}
