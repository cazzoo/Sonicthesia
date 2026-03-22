/// Score data for tracking player performance
#[derive(Debug, Clone)]
pub struct ScoreData {
    pub total_notes: usize,
    pub correct_notes: usize,
    pub missed_notes: usize,
    pub too_early: usize,
    pub too_late: usize,
    pub on_time: usize,
    pub accuracy: f64,
    pub grade: String,
    pub stars: u32,
    pub max_streak: u32,
    pub score: u64,
    pub perfect_count: u32,
    pub good_count: u32,
    pub okay_count: u32,
}

impl Default for ScoreData {
    fn default() -> Self {
        Self {
            total_notes: 0,
            correct_notes: 0,
            missed_notes: 0,
            too_early: 0,
            too_late: 0,
            on_time: 0,
            accuracy: 0.0,
            grade: String::new(),
            stars: 0,
            max_streak: 0,
            score: 0,
            perfect_count: 0,
            good_count: 0,
            okay_count: 0,
        }
    }
}
