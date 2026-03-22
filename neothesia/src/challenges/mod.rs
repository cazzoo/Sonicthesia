use chrono::{DateTime, Duration as ChronoDuration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeType {
    AccuracyTarget,
    StreakTarget,
    ScoreTarget,
    SpeedChallenge,
    PerfectRun,
}

impl ChallengeType {
    pub fn id_str(&self) -> &'static str {
        match self {
            ChallengeType::AccuracyTarget => "accuracy_target",
            ChallengeType::StreakTarget => "streak_target",
            ChallengeType::ScoreTarget => "score_target",
            ChallengeType::SpeedChallenge => "speed_challenge",
            ChallengeType::PerfectRun => "perfect_run",
        }
    }

    pub fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "accuracy_target" => Some(ChallengeType::AccuracyTarget),
            "streak_target" => Some(ChallengeType::StreakTarget),
            "score_target" => Some(ChallengeType::ScoreTarget),
            "speed_challenge" => Some(ChallengeType::SpeedChallenge),
            "perfect_run" => Some(ChallengeType::PerfectRun),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub name: String,
    pub description: String,
    pub target: f64,
    pub progress: f64,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reward_xp: u32,
    pub reward_badge: Option<String>,
}

impl Challenge {
    pub fn new_daily(index: u32) -> Self {
        let now = Utc::now();
        let expires = now + ChronoHours(24);

        let (challenge_type, name, description, target, reward_xp, reward_badge) = match index % 5 {
            0 => (
                ChallengeType::AccuracyTarget,
                "Perfect Practice".to_string(),
                "Get 95% accuracy on any song".to_string(),
                95.0,
                500,
                Some("Precision".to_string()),
            ),
            1 => (
                ChallengeType::StreakTarget,
                "Streak Runner".to_string(),
                "Achieve a 100 note streak".to_string(),
                100.0,
                400,
                Some("Consistency".to_string()),
            ),
            2 => (
                ChallengeType::ScoreTarget,
                "High Scorer".to_string(),
                "Score 200,000 points in a single song".to_string(),
                200_000.0,
                600,
                Some("Challenger".to_string()),
            ),
            3 => (
                ChallengeType::SpeedChallenge,
                "Speed Demon".to_string(),
                "Complete any song at 1.2x speed".to_string(),
                1.2,
                450,
                Some("Velocity".to_string()),
            ),
            _ => (
                ChallengeType::PerfectRun,
                "Flawless".to_string(),
                "Complete a song with no misses".to_string(),
                0.0,
                700,
                Some("Perfection".to_string()),
            ),
        };

        Self {
            id: format!("daily_{}", now.format("%Y%m%d")),
            challenge_type,
            name,
            description,
            target,
            progress: 0.0,
            completed: false,
            created_at: now,
            expires_at: expires,
            reward_xp,
            reward_badge,
        }
    }

    pub fn new_weekly() -> Self {
        let now = Utc::now();
        let expires = now + ChronoHours(168);

        Self {
            id: format!("weekly_{}", now.format("%Y%W")),
            challenge_type: ChallengeType::AccuracyTarget,
            name: "Weekly Dedication".to_string(),
            description: "Play 10 songs with 80%+ accuracy".to_string(),
            target: 10.0,
            progress: 0.0,
            completed: false,
            created_at: now,
            expires_at: expires,
            reward_xp: 2000,
            reward_badge: Some("Dedicated".to_string()),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn time_remaining(&self) -> ChronoDuration {
        let now = Utc::now();
        if now >= self.expires_at {
            ChronoDuration::zero()
        } else {
            self.expires_at - now
        }
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.target <= 0.0 {
            return if self.completed { 100.0 } else { 0.0 };
        }
        (self.progress / self.target * 100.0).min(100.0)
    }

    pub fn update_progress(&mut self, value: f64) -> bool {
        if self.completed || self.is_expired() {
            return false;
        }

        self.progress = value.max(self.progress);

        if self.progress >= self.target {
            self.completed = true;
            return true;
        }

        false
    }
}

fn ChronoHours(hours: i64) -> ChronoDuration {
    ChronoDuration::hours(hours)
}

#[derive(Debug)]
pub struct ChallengeManager {
    daily_challenge: Option<Challenge>,
    weekly_challenge: Option<Challenge>,
}

impl ChallengeManager {
    pub fn new() -> Self {
        Self {
            daily_challenge: None,
            weekly_challenge: None,
        }
    }

    pub fn get_or_create_daily(&mut self) -> &Challenge {
        let now = Utc::now();
        let today_id = format!("daily_{}", now.format("%Y%m%d"));

        let needs_new = match &self.daily_challenge {
            Some(challenge) => challenge.id != today_id || challenge.is_expired(),
            None => true,
        };

        if needs_new {
            let day_index = (now.timestamp() / 86400) as u32;
            self.daily_challenge = Some(Challenge::new_daily(day_index));
        }

        self.daily_challenge.as_ref().unwrap()
    }

    pub fn get_or_create_weekly(&mut self) -> &Challenge {
        let now = Utc::now();
        let week_id = format!("weekly_{}", now.format("%Y%W"));

        let needs_new = match &self.weekly_challenge {
            Some(challenge) => challenge.id != week_id || challenge.is_expired(),
            None => true,
        };

        if needs_new {
            self.weekly_challenge = Some(Challenge::new_weekly());
        }

        self.weekly_challenge.as_ref().unwrap()
    }

    pub fn daily_mut(&mut self) -> Option<&mut Challenge> {
        self.daily_challenge.as_mut()
    }

    pub fn weekly_mut(&mut self) -> Option<&mut Challenge> {
        self.weekly_challenge.as_mut()
    }

    pub fn daily(&self) -> Option<&Challenge> {
        self.daily_challenge.as_ref()
    }

    pub fn weekly(&self) -> Option<&Challenge> {
        self.weekly_challenge.as_ref()
    }
}

impl Default for ChallengeManager {
    fn default() -> Self {
        Self::new()
    }
}
