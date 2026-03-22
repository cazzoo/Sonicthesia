use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AchievementId {
    FirstSteps,
    Perfectionist,
    StreakMaster,
    SpeedDemon,
    NightOwl,
    GenreExplorer,
    Marathon,
    Comeback,
    Completionist,
}

impl AchievementId {
    pub fn all() -> &'static [AchievementId] {
        &[
            AchievementId::FirstSteps,
            AchievementId::Perfectionist,
            AchievementId::StreakMaster,
            AchievementId::SpeedDemon,
            AchievementId::NightOwl,
            AchievementId::GenreExplorer,
            AchievementId::Marathon,
            AchievementId::Comeback,
            AchievementId::Completionist,
        ]
    }

    pub fn id_str(&self) -> &'static str {
        match self {
            AchievementId::FirstSteps => "first_steps",
            AchievementId::Perfectionist => "perfectionist",
            AchievementId::StreakMaster => "streak_master",
            AchievementId::SpeedDemon => "speed_demon",
            AchievementId::NightOwl => "night_owl",
            AchievementId::GenreExplorer => "genre_explorer",
            AchievementId::Marathon => "marathon",
            AchievementId::Comeback => "comeback",
            AchievementId::Completionist => "completionist",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AchievementId::FirstSteps => "First Steps",
            AchievementId::Perfectionist => "Perfectionist",
            AchievementId::StreakMaster => "Streak Master",
            AchievementId::SpeedDemon => "Speed Demon",
            AchievementId::NightOwl => "Night Owl",
            AchievementId::GenreExplorer => "Genre Explorer",
            AchievementId::Marathon => "Marathon",
            AchievementId::Comeback => "Comeback",
            AchievementId::Completionist => "Completionist",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AchievementId::FirstSteps => "Complete your first song",
            AchievementId::Perfectionist => "Get 100% accuracy on any song",
            AchievementId::StreakMaster => "Achieve a 200 note streak",
            AchievementId::SpeedDemon => "Complete a song at 1.5x speed",
            AchievementId::NightOwl => "Play 10 songs after midnight",
            AchievementId::GenreExplorer => "Play songs from 5 different genres",
            AchievementId::Marathon => "Play for 2 hours total",
            AchievementId::Comeback => "Improve your score by 20% on any song",
            AchievementId::Completionist => "Get 5 stars on any Hard song",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Achievement {
    pub id: AchievementId,
    pub unlocked: bool,
    pub unlocked_at: Option<i64>,
    pub progress: f64,
}

impl Achievement {
    pub fn new(id: AchievementId) -> Self {
        Self {
            id,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
        }
    }

    pub fn unlock(&mut self, timestamp: i64) {
        self.unlocked = true;
        self.unlocked_at = Some(timestamp);
        self.progress = 1.0;
    }
}

#[derive(Debug)]
pub struct AchievementTracker {
    achievements: Vec<Achievement>,
    unlocked_ids: HashSet<AchievementId>,
}

impl AchievementTracker {
    pub fn new() -> Self {
        let achievements: Vec<Achievement> = AchievementId::all()
            .iter()
            .map(|id| Achievement::new(*id))
            .collect();

        let unlocked_ids = HashSet::new();

        Self {
            achievements,
            unlocked_ids,
        }
    }

    pub fn load_from_db(unlocked: Vec<(String, f64)>) -> Self {
        let mut tracker = Self::new();

        for (id_str, progress) in unlocked {
            for achievement in &mut tracker.achievements {
                if achievement.id.id_str() == id_str {
                    achievement.progress = progress;
                    if progress >= 1.0 {
                        achievement.unlocked = true;
                        tracker.unlocked_ids.insert(achievement.id);
                    }
                }
            }
        }

        tracker
    }

    pub fn achievements(&self) -> &[Achievement] {
        &self.achievements
    }

    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked_ids.contains(&id)
    }

    pub fn unlock(&mut self, id: AchievementId, timestamp: i64) -> bool {
        if self.is_unlocked(id) {
            return false;
        }

        for achievement in &mut self.achievements {
            if achievement.id == id {
                achievement.unlock(timestamp);
                self.unlocked_ids.insert(id);
                return true;
            }
        }

        false
    }

    pub fn update_progress(&mut self, id: AchievementId, progress: f64) {
        for achievement in &mut self.achievements {
            if achievement.id == id {
                achievement.progress = progress.max(achievement.progress);
                break;
            }
        }
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked_ids.len()
    }

    pub fn total_count(&self) -> usize {
        AchievementId::all().len()
    }
}

impl Default for AchievementTracker {
    fn default() -> Self {
        Self::new()
    }
}
