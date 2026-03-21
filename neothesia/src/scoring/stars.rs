//! Star rating system for performance evaluation

/// Star rating (0-5 stars)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StarRating(pub u32);

impl StarRating {
    pub const ZERO: StarRating = StarRating(0);
    pub const ONE: StarRating = StarRating(1);
    pub const TWO: StarRating = StarRating(2);
    pub const THREE: StarRating = StarRating(3);
    pub const FOUR: StarRating = StarRating(4);
    pub const FIVE: StarRating = StarRating(5);

    /// Calculate stars based on accuracy and max streak
    pub fn calculate(accuracy: f64, max_streak: u32) -> Self {
        let base_stars = match accuracy {
            95.0..=100.0 => 5,
            85.0..95.0 => 4,
            70.0..85.0 => 3,
            50.0..70.0 => 2,
            30.0..50.0 => 1,
            _ => 0,
        };

        // Bonus star for exceptional streak
        let bonus = if max_streak >= 100 { 1 } else { 0 };

        StarRating((base_stars + bonus).min(5))
    }

    /// Get star count
    pub fn count(&self) -> u32 {
        self.0
    }

    /// Get filled and empty star counts
    pub fn filled_empty(&self) -> (u32, u32) {
        (self.0, 5 - self.0)
    }

    /// Display as string (e.g., "★★★★☆")
    pub fn display(&self) -> String {
        let filled = "★".repeat(self.0 as usize);
        let empty = "☆".repeat((5 - self.0) as usize);
        format!("{}{}", filled, empty)
    }
}

impl std::fmt::Display for StarRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Complete score result after a song
#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub score: u64,
    pub accuracy: f64,
    pub max_streak: u32,
    pub stars: StarRating,
    pub perfect_count: u32,
    pub good_count: u32,
    pub okay_count: u32,
    pub miss_count: u32,
    pub total_notes: u32,
}

impl ScoreResult {
    /// Convert accuracy to letter grade
    pub fn grade(&self) -> &'static str {
        if self.accuracy >= 95.0 {
            "S"
        } else if self.accuracy >= 85.0 {
            "A"
        } else if self.accuracy >= 70.0 {
            "B"
        } else if self.accuracy >= 55.0 {
            "C"
        } else if self.accuracy >= 40.0 {
            "D"
        } else {
            "F"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_calculation() {
        assert_eq!(StarRating::calculate(100.0, 50).count(), 5);
        assert_eq!(StarRating::calculate(98.0, 50).count(), 5);
        assert_eq!(StarRating::calculate(95.0, 50).count(), 5);
        assert_eq!(StarRating::calculate(90.0, 50).count(), 4);
        assert_eq!(StarRating::calculate(85.0, 50).count(), 4);
        assert_eq!(StarRating::calculate(80.0, 50).count(), 3);
        assert_eq!(StarRating::calculate(70.0, 50).count(), 3);
        assert_eq!(StarRating::calculate(60.0, 50).count(), 2);
        assert_eq!(StarRating::calculate(50.0, 50).count(), 2);
        assert_eq!(StarRating::calculate(40.0, 50).count(), 1);
        assert_eq!(StarRating::calculate(30.0, 50).count(), 1);
        assert_eq!(StarRating::calculate(20.0, 50).count(), 0);
    }

    #[test]
    fn test_bonus_star_for_streak() {
        // 90% accuracy = 4 stars base, +1 for 100+ streak = 5 stars
        assert_eq!(StarRating::calculate(90.0, 100).count(), 5);
        // 90% accuracy = 4 stars base, no bonus for <100 streak
        assert_eq!(StarRating::calculate(90.0, 99).count(), 4);
        // Max 5 stars even with bonus
        assert_eq!(StarRating::calculate(95.0, 200).count(), 5);
    }

    #[test]
    fn test_star_display() {
        assert_eq!(StarRating::FIVE.display(), "★★★★★");
        assert_eq!(StarRating::FOUR.display(), "★★★★☆");
        assert_eq!(StarRating::THREE.display(), "★★★☆☆");
        assert_eq!(StarRating::TWO.display(), "★★☆☆☆");
        assert_eq!(StarRating::ONE.display(), "★☆☆☆☆");
        assert_eq!(StarRating::ZERO.display(), "☆☆☆☆☆");
    }

    #[test]
    fn test_grade() {
        let result = |acc| ScoreResult {
            score: 0,
            accuracy: acc,
            max_streak: 0,
            stars: StarRating::ZERO,
            perfect_count: 0,
            good_count: 0,
            okay_count: 0,
            miss_count: 0,
            total_notes: 0,
        };

        assert_eq!(result(100.0).grade(), "S");
        assert_eq!(result(95.0).grade(), "S");
        assert_eq!(result(90.0).grade(), "A");
        assert_eq!(result(85.0).grade(), "A");
        assert_eq!(result(75.0).grade(), "B");
        assert_eq!(result(60.0).grade(), "C");
        assert_eq!(result(45.0).grade(), "D");
        assert_eq!(result(30.0).grade(), "F");
    }
}
