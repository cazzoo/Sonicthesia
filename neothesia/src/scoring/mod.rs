//! Scoring and gamification system for Neothesia
//!
//! This module provides real-time scoring, streak tracking, star ratings,
//! and performance evaluation inspired by rhythm games like Guitar Hero.

pub mod live_score;
pub mod streak;
pub mod timing;
pub mod stars;

pub use live_score::LiveScoreTracker;
pub use streak::{StreakTracker, StreakMilestone};
pub use timing::TimingQuality;
pub use stars::StarRating;
