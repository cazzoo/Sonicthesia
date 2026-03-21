use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DifficultyFilter {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl DifficultyFilter {
    pub fn label(&self) -> &'static str {
        match self {
            DifficultyFilter::Easy => "Easy",
            DifficultyFilter::Medium => "Medium",
            DifficultyFilter::Hard => "Hard",
            DifficultyFilter::Expert => "Expert",
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0..=3 => DifficultyFilter::Easy,
            4..=6 => DifficultyFilter::Medium,
            7..=8 => DifficultyFilter::Hard,
            _ => DifficultyFilter::Expert,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveDifficulty {
    recent_accuracy: Vec<f64>,
    recent_streaks: Vec<u32>,
    skill_estimate: f64,
}

impl AdaptiveDifficulty {
    pub fn new() -> Self {
        Self {
            recent_accuracy: Vec::new(),
            recent_streaks: Vec::new(),
            skill_estimate: 0.5,
        }
    }

    pub fn update(&mut self, accuracy: f64, max_streak: u32) {
        self.recent_accuracy.push(accuracy);
        self.recent_streaks.push(max_streak);

        if self.recent_accuracy.len() > 10 {
            self.recent_accuracy.remove(0);
            self.recent_streaks.remove(0);
        }

        let avg_accuracy: f64 =
            self.recent_accuracy.iter().sum::<f64>() / self.recent_accuracy.len() as f64;
        let avg_streak: f64 =
            self.recent_streaks.iter().sum::<u32>() as f64 / self.recent_streaks.len() as f64;

        self.skill_estimate = (avg_accuracy / 100.0 * 0.7) + (avg_streak / 200.0 * 0.3).min(1.0);
    }

    pub fn skill_estimate(&self) -> f64 {
        self.skill_estimate
    }

    pub fn suggested_difficulty(&self) -> DifficultyFilter {
        match self.skill_estimate {
            x if x < 0.3 => DifficultyFilter::Easy,
            x if x < 0.5 => DifficultyFilter::Medium,
            x if x < 0.8 => DifficultyFilter::Hard,
            _ => DifficultyFilter::Expert,
        }
    }

    pub fn suggested_speed(&self) -> f32 {
        (0.8 + (self.skill_estimate as f32 * 0.4)).clamp(0.5, 1.5)
    }

    pub fn skill_label(&self) -> &'static str {
        match self.skill_estimate {
            x if x < 0.3 => "Beginner",
            x if x < 0.5 => "Intermediate",
            x if x < 0.8 => "Advanced",
            _ => "Expert",
        }
    }
}

impl Default for AdaptiveDifficulty {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StageRequirement {
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct LearningStage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requirements: Vec<StageRequirement>,
    pub unlocked: bool,
    pub completed: bool,
}

impl LearningStage {
    pub fn new(id: &str, name: &str, description: &str, requirements: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            requirements: requirements
                .into_iter()
                .map(|r| StageRequirement {
                    description: r.to_string(),
                    completed: false,
                })
                .collect(),
            unlocked: false,
            completed: false,
        }
    }

    pub fn all_requirements_met(&self) -> bool {
        self.requirements.iter().all(|r| r.completed)
    }

    pub fn progress(&self) -> f64 {
        if self.requirements.is_empty() {
            return if self.completed { 1.0 } else { 0.0 };
        }
        let completed = self.requirements.iter().filter(|r| r.completed).count();
        completed as f64 / self.requirements.len() as f64
    }
}

#[derive(Debug, Clone)]
pub struct LearningPath {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stages: Vec<LearningStage>,
    pub current_stage: usize,
}

impl LearningPath {
    pub fn first_steps() -> Self {
        Self {
            id: "first_steps".to_string(),
            name: "First Steps".to_string(),
            description: "Learn the basics of piano playing".to_string(),
            stages: vec![
                LearningStage::new(
                    "getting_started",
                    "Getting Started",
                    "Play your first song",
                    vec!["Complete any song"],
                ),
                LearningStage::new(
                    "rhythm_basics",
                    "Rhythm Basics",
                    "Develop basic rhythm skills",
                    vec!["Complete 3 songs with 50%+ accuracy"],
                ),
                LearningStage::new(
                    "timing_master",
                    "Timing Master",
                    "Master note timing",
                    vec!["Achieve 70% accuracy on any Easy song"],
                ),
                LearningStage::new(
                    "streak_starter",
                    "Streak Starter",
                    "Build your first streak",
                    vec!["Get a 30-note streak"],
                ),
                LearningStage::new(
                    "ready_for_more",
                    "Ready for More",
                    "Move to medium difficulty",
                    vec!["Get 3 stars on any Medium song"],
                ),
            ],
            current_stage: 0,
        }
    }

    pub fn rhythm_mastery() -> Self {
        Self {
            id: "rhythm_mastery".to_string(),
            name: "Rhythm Mastery".to_string(),
            description: "Master intermediate rhythm techniques".to_string(),
            stages: vec![
                LearningStage::new(
                    "chord_explorer",
                    "Chord Explorer",
                    "Learn to play chords",
                    vec!["Complete 5 songs with chords"],
                ),
                LearningStage::new(
                    "speed_demon",
                    "Speed Demon",
                    "Play at increased speed",
                    vec!["Complete any song at 1.2x speed"],
                ),
                LearningStage::new(
                    "perfect_timing",
                    "Perfect Timing",
                    "Achieve perfect timing accuracy",
                    vec!["Get 85% accuracy on 3 different songs"],
                ),
                LearningStage::new(
                    "streak_legend",
                    "Streak Legend",
                    "Build impressive streaks",
                    vec!["Get a 100-note streak"],
                ),
                LearningStage::new(
                    "ready_for_hard",
                    "Ready for Hard",
                    "Tackle hard difficulty",
                    vec!["Get 4 stars on any Hard song"],
                ),
            ],
            current_stage: 0,
        }
    }

    pub fn virtuoso() -> Self {
        Self {
            id: "virtuoso".to_string(),
            name: "Virtuoso".to_string(),
            description: "Become a piano virtuoso".to_string(),
            stages: vec![
                LearningStage::new(
                    "expert_mode",
                    "Expert Mode",
                    "Play on expert difficulty",
                    vec!["Complete 5 songs on Expert difficulty"],
                ),
                LearningStage::new(
                    "perfectionist",
                    "Perfectionist",
                    "Achieve near-perfect accuracy",
                    vec!["Get 95% accuracy on any Expert song"],
                ),
                LearningStage::new(
                    "marathon_runner",
                    "Marathon Runner",
                    "Play for extended periods",
                    vec!["Play for 1 hour total"],
                ),
                LearningStage::new(
                    "virtuoso",
                    "Virtuoso",
                    "Master the piano",
                    vec!["Get 5 stars on 10 different songs"],
                ),
            ],
            current_stage: 0,
        }
    }

    pub fn unlock_next_stage(&mut self) -> bool {
        if self.current_stage < self.stages.len() {
            let stage = &mut self.stages[self.current_stage];
            if stage.all_requirements_met() {
                stage.completed = true;
                self.current_stage += 1;

                if self.current_stage < self.stages.len() {
                    self.stages[self.current_stage].unlocked = true;
                }

                return true;
            }
        }
        false
    }

    pub fn progress(&self) -> f64 {
        let completed = self.stages.iter().filter(|s| s.completed).count();
        completed as f64 / self.stages.len() as f64
    }

    pub fn is_completed(&self) -> bool {
        self.stages.iter().all(|s| s.completed)
    }
}

#[derive(Debug)]
pub struct LearningProgressManager {
    paths: Vec<LearningPath>,
    completed_paths: usize,
}

impl LearningProgressManager {
    pub fn new() -> Self {
        let mut first_steps = LearningPath::first_steps();
        if !first_steps.stages.is_empty() {
            first_steps.stages[0].unlocked = true;
        }

        Self {
            paths: vec![
                first_steps,
                LearningPath::rhythm_mastery(),
                LearningPath::virtuoso(),
            ],
            completed_paths: 0,
        }
    }

    pub fn paths(&self) -> &[LearningPath] {
        &self.paths
    }

    pub fn get_path(&self, id: &str) -> Option<&LearningPath> {
        self.paths.iter().find(|p| p.id == id)
    }

    pub fn get_path_mut(&mut self, id: &str) -> Option<&mut LearningPath> {
        self.paths.iter_mut().find(|p| p.id == id)
    }

    pub fn overall_progress(&self) -> f64 {
        let total: f64 = self.paths.iter().map(|p| p.progress()).sum();
        total / self.paths.len() as f64
    }

    pub fn completed_paths(&self) -> usize {
        self.completed_paths
    }
}

impl Default for LearningProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SkillMetrics {
    pub avg_accuracy: f64,
    pub avg_streak: u32,
    pub max_streak_ever: u32,
    pub songs_completed: u32,
    pub songs_5_starred: u32,
    pub total_play_time_secs: u64,
    pub sessions_count: u32,
    pub difficulty_distribution: HashMap<DifficultyFilter, u32>,
}

impl SkillMetrics {
    pub fn new() -> Self {
        Self {
            avg_accuracy: 0.0,
            avg_streak: 0,
            max_streak_ever: 0,
            songs_completed: 0,
            songs_5_starred: 0,
            total_play_time_secs: 0,
            sessions_count: 0,
            difficulty_distribution: HashMap::new(),
        }
    }

    pub fn update_session(
        &mut self,
        accuracy: f64,
        max_streak: u32,
        stars: u32,
        difficulty: DifficultyFilter,
        play_time_secs: u64,
    ) {
        self.sessions_count += 1;

        let n = self.sessions_count as f64;
        self.avg_accuracy = ((self.avg_accuracy * (n - 1.0)) + accuracy) / n;
        self.avg_streak = ((self.avg_streak as f64 * (n - 1.0)) + max_streak as f64 / n) as u32;

        if max_streak > self.max_streak_ever {
            self.max_streak_ever = max_streak;
        }

        self.songs_completed += 1;
        if stars >= 5 {
            self.songs_5_starred += 1;
        }

        self.total_play_time_secs += play_time_secs;

        *self.difficulty_distribution.entry(difficulty).or_insert(0) += 1;
    }

    pub fn overall_skill_level(&self) -> &'static str {
        match self.avg_accuracy {
            x if x >= 90.0 => "Expert",
            x if x >= 75.0 => "Advanced",
            x if x >= 60.0 => "Intermediate",
            _ => "Beginner",
        }
    }
}

impl Default for SkillMetrics {
    fn default() -> Self {
        Self::new()
    }
}
