# Neothesia Gamification Plan

## Executive Summary

This plan transforms Neothesia from a MIDI visualizer into an engaging, replayable rhythm game inspired by Rockband/Guitar Hero. The core philosophy: **make every session feel rewarding, every mistake feel recoverable, and every replay feel worth it**.

**Rendering Engine**: PLY (Macroquad) only. No WGPU references. No PLYUi.

---

## Current State Analysis

### Existing Foundation (Build Upon)
- **Scoring System**: Basic timing evaluation (perfect ≤50ms, good ≤100ms, okay ≤200ms, miss)
- **Grade System**: Letter grades (S/A/B/C/D/F) based on accuracy percentage
- **Visual Effects**: Glow effects (`PlyGlowRenderer`), particle systems (`PlyParticleRenderer`), background animations (`PlyBackgroundRenderer`)
- **Song Library**: SQLite database with metadata, difficulty ratings, play statistics
- **LUMI Integration**: LED keyboard feedback (green=required, blue=upcoming)
- **Playback Modes**: Watch/Learn/Play with channel configurations
- **Rendering**: PLY engine with Macroquad (`neothesia/src/render/ply/effects.rs`)

### Missing Elements (To Implement)
- Real-time scoring feedback during gameplay
- Streak/combo system with multipliers
- Replayability incentives (stars, unlocks, leaderboards)
- Difficulty filtering (Easy/Medium/Hard/Expert)
- Performance ranking beyond accuracy
- Structured learning progression system

---

## Phase 1: Real-Time Scoring & Feedback

### 1.1 Live Score Display

**Goal**: Show score updating in real-time during gameplay.

**Implementation**:
```
File: neothesia/src/scene/playing_scene/mod.rs
- Add `LiveScoreTracker` struct to PlayingScene
- Track: current_score, streak_count, multiplier, max_streak
- Render score in top-right corner using Macroquad text rendering
- Update on every note hit/miss event
```

**Scoring Formula** (Guitar Hero inspired):
```rust
struct LiveScoreTracker {
    current_score: u64,
    base_points_per_note: u64,      // 50 points
    streak_count: u32,
    multiplier: u32,                // 1x, 2x, 4x, 8x
    max_streak: u32,
}

impl LiveScoreTracker {
    fn calculate_multiplier(streak: u32) -> u32 {
        match streak {
            0..=9 => 1,
            10..=29 => 2,
            30..=49 => 4,
            _ => 8,
        }
    }

    fn on_note_hit(&mut self, timing_quality: TimingQuality) {
        let base_points = match timing_quality {
            TimingQuality::Perfect => 50,
            TimingQuality::Good => 25,
            TimingQuality::Okay => 10,
            TimingQuality::Miss => 0,
        };

        self.current_score += base_points * self.multiplier as u64;

        if timing_quality != TimingQuality::Miss {
            self.streak_count += 1;
            self.multiplier = Self::calculate_multiplier(self.streak_count);
            if self.streak_count > self.max_streak {
                self.max_streak = self.streak_count;
            }
        } else {
            self.streak_count = 0;
            self.multiplier = 1;
        }
    }
}
```

**UI Layout** (rendered via Macroquad):
```
┌─────────────────────────────────────────┐
│  125,000  ×4                            │
│  ████████████░░░░  Streak: 47           │
│  ┌─────────────────────────────────┐    │
│  │         Waterfall               │    │
│  │                                 │    │
│  └─────────────────────────────────┘    │
│  🎹 Piano Keyboard                      │
└─────────────────────────────────────────┘
```

### 1.2 Timing Quality Feedback

**Goal**: Visual feedback for each note hit quality.

**Implementation**:
```
File: neothesia/src/scene/playing_scene/midi_player.rs
- Add TimingQuality enum (Perfect/Good/Okay/Miss)
- Return timing quality from PlayAlong evaluation
- Trigger visual feedback based on quality

File: neothesia/src/render/ply/effects.rs
- Use existing PlyParticleRenderer for hit effects
- Use existing PlyGlowRenderer for key glow
```

**Visual Feedback Effects**:
| Quality | Color | Particle Count | Screen Effect | Sound |
|---------|-------|----------------|---------------|-------|
| Perfect | Gold (#FFD700) | 20 | Screen flash | Chime |
| Good | Green (#00FF00) | 10 | Glow burst | Click |
| Okay | Blue (#0088FF) | 5 | Small sparkle | Tick |
| Miss | Red (#FF0000) | 0 | Screen shake | Thud |

**Particle Spawn Logic** (using existing `PlyParticleRenderer`):
```rust
fn spawn_hit_effect(&mut self, x: f32, y: f32, quality: TimingQuality) {
    let (color, count) = match quality {
        TimingQuality::Perfect => (Color::rgb(1.0, 0.84, 0.0), 20),
        TimingQuality::Good => (Color::rgb(0.0, 1.0, 0.0), 10),
        TimingQuality::Okay => (Color::rgb(0.0, 0.53, 1.0), 5),
        TimingQuality::Miss => (Color::rgb(1.0, 0.0, 0.0), 0),
    };

    if count > 0 {
        self.particles.spawn(x, y, color, count);
    }

    if quality == TimingQuality::Miss {
        self.screen_shake.intensity = 0.3;
        self.screen_shake.duration = Duration::from_millis(100);
    }
}
```

### 1.3 Streak Display

**Goal**: Show current streak with visual emphasis at milestones.

**Milestones**:
- **10 notes**: Streak counter turns green, multiplier 2×
- **30 notes**: Streak counter turns blue, multiplier 4×
- **50 notes**: Streak counter turns gold, multiplier 8×, screen flash
- **100 notes**: Special "On Fire" effect, persistent glow
- **200 notes**: "Legendary" status, rainbow note colors

**Implementation**:
```
File: neothesia/src/render/ply/effects.rs
- Add StreakEffect struct
- Track streak milestones reached
- Trigger special effects at thresholds using PlyParticleRenderer
- Render "ON FIRE!" text using Macroquad text
```

---

## Phase 2: Performance Ranking & Replayability

### 2.1 Star Rating System

**Goal**: Replace binary grade with 5-star rating that motivates replay.

**Star Thresholds** (Guitar Hero inspired):
```rust
fn calculate_stars(accuracy: f64, streak_bonus: u32) -> u32 {
    let base_stars = match accuracy {
        95.0..=100.0 => 5,
        85.0..95.0 => 4,
        70.0..85.0 => 3,
        50.0..70.0 => 2,
        30.0..50.0 => 1,
        _ => 0,
    };

    // Bonus stars for exceptional performance
    let mut bonus = 0;
    if streak_bonus >= 100 { bonus += 1; }

    (base_stars + bonus).min(5)
}
```

**Star Display** (Macroquad rendered):
```
┌─────────────────────────────────────────┐
│         Song Complete!                  │
│                                         │
│            ★★★★☆                        │
│           4 / 5 Stars                   │
│                                         │
│  Accuracy: 87%                          │
│  Best Streak: 156                       │
│                                         │
│  [Replay]  [Continue]                   │
└─────────────────────────────────────────┘
```

### 2.2 High Score System

**Goal**: Track and display best performances per song.

**Data Structure**:
```rust
struct SongHighScore {
    song_id: i64,
    best_score: u64,
    best_accuracy: f64,
    best_streak: u32,
    best_stars: u32,
    play_count: u32,
    first_played: DateTime<Utc>,
    last_played: DateTime<Utc>,
    recent_scores: Vec<u64>,  // Last 10 scores
}
```

**Leaderboard Display** (in Song Library):
```
┌─────────────────────────────────────────┐
│  🎵 Fur Elise                           │
│  Difficulty: ████████░░ 8/10            │
│                                         │
│  Your Best:                             │
│  ★★★★★  245,000 pts  98% accuracy      │
│  Streak: 312  |  Played: 15 times      │
│                                         │
│  Recent Scores:                         │
│  1. 245,000 (★★★★★)  ← New Best!      │
│  2. 232,000 (★★★★☆)                    │
│  3. 218,000 (★★★☆☆)                    │
│  ...                                    │
└─────────────────────────────────────────┘
```

**Implementation**:
```
File: neothesia/src/song_library/models.rs
- Add HighScore struct
- Add recent_scores field to SongEntry
- Update database schema for new fields
- Track high scores in ScoreScene
```

### 2.3 Achievement System

**Goal**: Unlockable achievements that reward specific playstyles.

**Achievement Categories**:

| Achievement | Description | Reward |
|-------------|-------------|--------|
| First Steps | Complete first song | Unlock "Classic" theme |
| Perfectionist | Get 100% accuracy | Unlock "Gold" notes |
| Streak Master | 200 note streak | Unlock "Fire" effects |
| Speed Demon | Complete at 1.5x speed | Unlock "Lightning" background |
| Night Owl | Play 10 songs after midnight | Unlock "Midnight" theme |
| Genre Explorer | Play 5 different genres | Unlock "Rainbow" notes |
| Marathon | Play for 2 hours straight | Unlock "Eternal" effects |
| Comeback | Improve score by 20% | Unlock "Phoenix" effects |
| Completionist | 100% any Hard song | Unlock "Diamond" notes |

**Implementation**:
```
File: neothesia/src/achievements/mod.rs (NEW)
- Create Achievement enum with all achievements
- Track achievement progress in database
- Show unlock notifications during gameplay (Macroquad overlay)
- Store unlocked cosmetics in config
```

### 2.4 Daily/Weekly Challenges

**Goal**: Time-limited challenges that encourage regular play.

**Challenge Types**:
```rust
enum ChallengeType {
    AccuracyTarget(f64),           // "Get 95% accuracy"
    StreakTarget(u32),             // "Achieve 100 note streak"
    ScoreTarget(u64),              // "Score 200,000 points"
    SpeedChallenge(f32),           // "Complete at 1.2x speed"
    PerfectRun,                    // "No misses"
}
```

**Challenge UI** (Macroquad rendered):
```
┌─────────────────────────────────────────┐
│  📅 Daily Challenge                     │
│                                         │
│  "Perfect Practice"                     │
│  Get 95% accuracy on any Hard song      │
│                                         │
│  Progress: 87% / 95%                    │
│  ████████████████░░░░░░░░               │
│                                         │
│  Reward: 500 XP + "Precision" badge     │
│  Time Left: 18h 23m                     │
└─────────────────────────────────────────┘
```

---

## Phase 3: Difficulty System & Track Management

### 3.1 Difficulty Filtering

**Goal**: Allow players to filter notes by difficulty (Guitar Hero style).

**Difficulty Levels**:
```rust
enum DifficultyFilter {
    Easy,      // Only root notes, slow sections
    Medium,    // Simplified chords, reduced speed
    Hard,      // All notes, original speed
    Expert,    // All notes + embellishments
}

impl DifficultyFilter {
    fn filter_notes(&self, notes: &[MidiNote]) -> Vec<MidiNote> {
        match self {
            DifficultyFilter::Easy => {
                // Keep only lowest note in each chord
                // Remove notes faster than 1/8th notes
                notes.iter()
                    .filter(|n| n.velocity > 80)
                    .cloned()
                    .collect()
            }
            DifficultyFilter::Medium => {
                // Simplify chords to 2 notes max
                // Keep original speed
                notes.iter()
                    .take(notes.len() * 2 / 3)
                    .cloned()
                    .collect()
            }
            DifficultyFilter::Hard => notes.to_vec(),
            DifficultyFilter::Expert => {
                // Add passing notes and ornaments
                notes.to_vec()
            }
        }
    }
}
```

**Implementation**:
```
File: neothesia/src/song.rs
- Add DifficultyFilter enum
- Add filter_notes method to Song
- Apply filter during track loading
- Show filtered note count in UI
```

### 3.2 Track Difficulty Rating

**Goal**: More accurate difficulty calculation based on multiple factors.

**Enhanced Difficulty Formula**:
```rust
fn calculate_difficulty(metadata: &SongMetadata) -> DifficultyRating {
    let note_density = metadata.note_count as f32 / metadata.duration_secs as f32;
    let chord_complexity = metadata.avg_notes_per_chord;
    let rhythm_complexity = metadata.rhythm_variety;
    let tempo_factor = metadata.tempo_changes as f32 / 100.0;

    // Weighted score (1-10)
    let raw_score = (note_density * 0.4)
        + (chord_complexity * 0.3)
        + (rhythm_complexity * 0.2)
        + (tempo_factor * 0.1);

    DifficultyRating {
        numeric: raw_score.clamp(1.0, 10.0) as u8,
        label: match raw_score {
            1.0..=3.0 => "Beginner",
            3.0..=5.0 => "Easy",
            5.0..=7.0 => "Medium",
            7.0..=9.0 => "Hard",
            _ => "Expert",
        },
        recommended_filter: if raw_score < 4.0 {
            DifficultyFilter::Easy
        } else if raw_score < 6.0 {
            DifficultyFilter::Medium
        } else if raw_score < 8.0 {
            DifficultyFilter::Hard
        } else {
            DifficultyFilter::Expert
        },
    }
}
```

### 3.3 Song Library Enhancements

**Goal**: Better song browsing with gamification context.

**Enhanced Song Card** (Macroquad rendered):
```
┌─────────────────────────────────────────┐
│  🎵 Fur Elise                           │
│  Beethoven                              │
│                                         │
│  Difficulty: ████████░░ 8/10 (Hard)     │
│  Duration: 3:45                         │
│  Your Best: ★★★★☆  232,000 pts         │
│  Last Played: 2 days ago                │
│                                         │
│  [Play] [Practice] [Watch]              │
└─────────────────────────────────────────┘
```

**Sorting Options** (add to existing):
- By difficulty (Easy → Hard)
- By last played (Recent → Oldest)
- By best score (Highest → Lowest)
- By play count (Most → Least)
- By completion (100% first)

**Filtering Options** (add to existing):
- By difficulty range
- By star rating achieved
- By completion status
- By genre/mood

---

## Phase 4: Learning Progression System

### 4.1 Skill Tree / Learning Path

**Goal**: Structured progression that guides players from beginner to expert.

**Learning Path Structure**:
```rust
struct LearningPath {
    id: String,
    name: String,
    description: String,
    stages: Vec<LearningStage>,
    current_stage: usize,
}

struct LearningStage {
    id: String,
    name: String,
    description: String,
    requirements: Vec<StageRequirement>,
    rewards: Vec<Reward>,
    songs: Vec<SongRecommendation>,
    unlocked: bool,
    completed: bool,
}

enum StageRequirement {
    CompleteSong(String, u32),      // Song ID, min stars
    AchieveAccuracy(f64),           // Min accuracy percentage
    AchieveStreak(u32),             // Min streak length
    PlayCount(u32),                 // Min play count
    CompleteChallenge(String),      // Challenge ID
}
```

**Example Learning Paths**:

**Path 1: "First Steps" (Beginner)**
```
Stage 1: "Getting Started"
  - Play any song
  - Reward: "Rookie" badge
  
Stage 2: "Rhythm Basics"
  - Complete 3 songs with 50%+ accuracy
  - Reward: Unlock metronome practice mode
  
Stage 3: "Timing Master"
  - Achieve 70% accuracy on any Easy song
  - Reward: Unlock "Precise" note effects
  
Stage 4: "Streak Starter"
  - Get a 30-note streak
  - Reward: Unlock "Streak" visual effects
  
Stage 5: "Ready for More"
  - Get 3 stars on any Medium song
  - Reward: Unlock "Intermediate" learning path
```

**Path 2: "Rhythm Mastery" (Intermediate)**
```
Stage 1: "Chord Explorer"
  - Complete 5 songs with chords
  - Reward: Unlock chord highlighting
  
Stage 2: "Speed Demon"
  - Complete any song at 1.2x speed
  - Reward: Unlock speed multiplier display
  
Stage 3: "Perfect Timing"
  - Get 85% accuracy on 3 different songs
  - Reward: Unlock "Perfect" timing window expansion
  
Stage 4: "Streak Legend"
  - Get a 100-note streak
  - Reward: Unlock "On Fire" effects
  
Stage 5: "Ready for Hard"
  - Get 4 stars on any Hard song
  - Reward: Unlock "Advanced" learning path
```

**Path 3: "Virtuoso" (Advanced)**
```
Stage 1: "Expert Mode"
  - Complete 5 songs on Expert difficulty
  - Reward: Unlock "Expert" note effects
  
Stage 2: "Perfectionist"
  - Get 95% accuracy on any Expert song
  - Reward: Unlock "Diamond" notes
  
Stage 3: "Marathon Runner"
  - Play for 1 hour total
  - Reward: Unlock "Eternal" background
  
Stage 4: "Virtuoso"
  - Get 5 stars on 10 different songs
  - Reward: Unlock "Virtuoso" title and all cosmetics
```

### 4.2 Practice Modes

**Goal**: Dedicated practice modes that help players improve specific skills.

**Practice Mode Types**:
```rust
enum PracticeMode {
    FullSong,              // Normal playthrough
    SectionLoop,           // Loop a difficult section
    SlowMotion,            // Play at reduced speed
    Metronome,             // Play with metronome guide
    HandSeparate,          // Practice left/right hand
    NoMiss,                // Restart on any miss
    Accuracy,              // Focus on timing accuracy
}
```

**Section Loop Mode**:
```
┌─────────────────────────────────────────┐
│  🔄 Section Practice                    │
│                                         │
│  Section: Measures 16-24                │
│  Difficulty: ████████░░ 7/10            │
│                                         │
│  Your Best: 78% accuracy                │
│  Target: 90% accuracy                   │
│                                         │
│  Speed: 75%                             │
│  ████████████░░░░░░░░                   │
│                                         │
│  [Start Loop] [Adjust Speed] [Exit]     │
└─────────────────────────────────────────┘
```

**Slow Motion Mode**:
```rust
struct SlowMotionConfig {
    speed: f32,              // 0.25 - 1.0
    show_note_names: bool,
    highlight_finger_positions: bool,
    pause_on_miss: bool,
    auto_increase_speed: bool,  // Gradually increase speed as you improve
}
```

### 4.3 Adaptive Difficulty

**Goal**: System that adjusts to player skill level.

**Adaptive Mechanics**:
```rust
struct AdaptiveDifficulty {
    player_skill_estimate: f32,     // 0.0 - 1.0
    recent_accuracy: Vec<f32>,      // Last 10 play sessions
    recent_streaks: Vec<u32>,
    suggested_difficulty: DifficultyFilter,
    suggested_speed: f32,
}

impl AdaptiveDifficulty {
    fn update(&mut self, session_result: &SessionResult) {
        self.recent_accuracy.push(session_result.accuracy);
        self.recent_streaks.push(session_result.max_streak);
        
        // Keep only last 10
        if self.recent_accuracy.len() > 10 {
            self.recent_accuracy.remove(0);
            self.recent_streaks.remove(0);
        }
        
        // Calculate skill estimate
        let avg_accuracy = self.recent_accuracy.iter().sum::<f32>() / self.recent_accuracy.len() as f32;
        let avg_streak = self.recent_streaks.iter().sum::<u32>() as f32 / self.recent_streaks.len() as f32;
        
        self.player_skill_estimate = (avg_accuracy / 100.0 * 0.7) + (avg_streak / 200.0 * 0.3).min(1.0);
        
        // Suggest difficulty
        self.suggested_difficulty = match self.player_skill_estimate {
            0.0..0.3 => DifficultyFilter::Easy,
            0.3..0.5 => DifficultyFilter::Medium,
            0.5..0.8 => DifficultyFilter::Hard,
            _ => DifficultyFilter::Expert,
        };
        
        // Suggest speed (between 0.8 and 1.2)
        self.suggested_speed = 0.8 + (self.player_skill_estimate * 0.4);
    }
}
```

**Suggestion UI** (shown before song starts):
```
┌─────────────────────────────────────────┐
│  💡 Recommended Settings                │
│                                         │
│  Based on your recent performance:      │
│                                         │
│  Difficulty: Medium (Recommended)       │
│  Speed: 1.0x                            │
│                                         │
│  Your skill level: Intermediate         │
│  ████████████████░░░░ 68%               │
│                                         │
│  [Use Recommended] [Custom]             │
└─────────────────────────────────────────┘
```

### 4.4 Skill Metrics & Progress Tracking

**Goal**: Detailed tracking of player improvement over time.

**Metrics Tracked**:
```rust
struct SkillMetrics {
    // Timing
    avg_accuracy: f32,
    accuracy_trend: Vec<f32>,           // Per session
    perfect_note_percentage: f32,
    
    // Consistency
    avg_streak: u32,
    max_streak_ever: u32,
    streak_consistency: f32,            // Standard deviation
    
    // Speed
    avg_speed_multiplier: f32,
    max_speed_achieved: f32,
    
    // Repertoire
    songs_completed: u32,
    songs_5_starred: u32,
    difficulty_distribution: HashMap<DifficultyFilter, u32>,
    
    // Engagement
    total_play_time: Duration,
    sessions_count: u32,
    avg_session_length: Duration,
    days_active: u32,
}
```

**Progress Dashboard** (Macroquad rendered):
```
┌─────────────────────────────────────────┐
│  📊 Your Progress                       │
│                                         │
│  Overall Skill: Advanced                │
│  ████████████████████░░░░ 78%           │
│                                         │
│  Timing Accuracy                        │
│  Current: 87%  |  Best: 96%             │
│  ████████████████████░░░░               │
│  ↑ +3% from last week                   │
│                                         │
│  Streak Consistency                     │
│  Average: 45  |  Max: 234               │
│  ████████████████░░░░░░░░               │
│                                         │
│  Songs Mastered (5★)                    │
│  12 / 47 songs                          │
│  ████████░░░░░░░░░░░░░░░░               │
│                                         │
│  [View Detailed Stats] [Back]           │
└─────────────────────────────────────────┘
```

### 4.5 Guided Practice Sessions

**Goal**: AI-suggested practice sessions based on weaknesses.

**Session Structure**:
```rust
struct PracticeSession {
    warmup: Vec<WarmupExercise>,
    focus_area: FocusArea,
    songs: Vec<PracticeSong>,
    cooldown: Vec<CooldownExercise>,
    estimated_duration: Duration,
}

enum FocusArea {
    Timing,              // Accuracy-focused
    Speed,               // Tempo increase
    Endurance,           // Long songs, consistency
    Chords,              # Chord recognition
    SightReading,        // New songs, no practice
    SpecificWeakness(String),  // Targeted improvement
}

struct PracticeSong {
    song_id: String,
    difficulty: DifficultyFilter,
    speed: f32,
    target_accuracy: f32,
    reason: String,      // "This song has similar patterns to ones you struggle with"
}
```

**Suggested Session UI**:
```
┌─────────────────────────────────────────┐
│  🎯 Recommended Practice Session        │
│                                         │
│  Focus: Timing Accuracy                 │
│  Duration: ~25 minutes                  │
│                                         │
│  Warmup (5 min)                         │
│  • Scale exercises at 0.8x speed        │
│                                         │
│  Main Practice (15 min)                 │
│  • "Für Elise" - Medium, 0.9x speed     │
│  • Target: 85% accuracy                 │
│  • Reason: Similar to songs you         │
│    struggle with                        │
│                                         │
│  Challenge (5 min)                      │
│  • "Moonlight Sonata" - Hard, 1.0x      │
│  • Target: 70% accuracy                 │
│                                         │
│  [Start Session] [Customize] [Skip]     │
└─────────────────────────────────────────┘
```

### 4.6 Weekly Learning Goals

**Goal**: Weekly objectives that guide skill development.

**Goal Types**:
```rust
enum WeeklyGoal {
    AccuracyImprovement(f64),      // "Improve accuracy by 5%"
    StreakAchievement(u32),        // "Get a 100-note streak"
    SongCompletion(u32),           // "Complete 5 new songs"
    DifficultyProgression,         // "Try a harder difficulty"
    PracticeConsistency(u32),      // "Practice 4 days this week"
    SkillFocus(SkillArea),         // "Focus on timing this week"
}
```

**Weekly Goal UI**:
```
┌─────────────────────────────────────────┐
│  📅 This Week's Goals                   │
│                                         │
│  1. Improve accuracy by 5%              │
│     Progress: 82% → 85% (+3%)           │
│     ████████████████░░░░░░░░            │
│                                         │
│  2. Complete 3 new songs                │
│     Progress: 2 / 3                     │
│     ████████████████░░░░░░░░            │
│                                         │
│  3. Practice 4 days                     │
│     Progress: 3 / 4 days                │
│     ████████████████████░░░░            │
│                                         │
│  Reward: "Dedicated Learner" badge      │
│  Time Left: 3 days 14 hours             │
│                                         │
│  [View All Goals] [Back]                │
└─────────────────────────────────────────┘
```

### 4.7 Technique Tutorials & Tips

**Goal**: In-game tutorials that teach piano techniques.

**Tutorial Types**:
```rust
enum TutorialType {
    FingerPositioning,     // Which fingers for which keys
    HandIndependence,      // Left/right hand coordination
    ChordShapes,           // Common chord patterns
    Scales,                // Major/minor scales
    Arpeggios,             # Broken chords
    Dynamics,              // Playing soft/loud
    Pedaling,              // Sustain pedal usage
}
```

**Tutorial Integration**:
- Show tips before songs that use specific techniques
- Highlight finger positions on the virtual keyboard
- Suggest practice exercises for weak areas
- Link to external resources (YouTube, etc.)

**Tip Display** (shown during gameplay):
```
┌─────────────────────────────────────────┐
│  💡 Tip: Hand Independence              │
│                                         │
│  This section requires your left and    │
│  right hand to play different rhythms.  │
│                                         │
│  Practice each hand separately first,   │
│  then combine slowly.                   │
│                                         │
│  [Show Hand Parts] [Dismiss]            │
└─────────────────────────────────────────┘
```

---

## Phase 5: Visual Polish & Juice (Macroquad)

### 5.1 Screen Effects System

**Goal**: Add satisfying visual feedback for all game events.

**Screen Shake** (using Macroquad camera shake):
```rust
struct ScreenShake {
    intensity: f32,
    duration: Duration,
    elapsed: Duration,
    frequency: f32,
}

impl ScreenShake {
    fn apply(&self, camera: &mut Camera2D) {
        if self.elapsed < self.duration {
            let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
            let decay = 1.0 - progress;
            let shake_x = (self.elapsed.as_secs_f32() * self.frequency).sin() * self.intensity * decay;
            let shake_y = (self.elapsed.as_secs_f32() * self.frequency * 1.3).cos() * self.intensity * decay;
            camera.offset = vec2(shake_x, shake_y);
        }
    }
}
```

**Screen Flash** (using Macroquad draw_rectangle):
```rust
struct ScreenFlash {
    color: Color,
    intensity: f32,
    duration: Duration,
    elapsed: Duration,
}

impl ScreenFlash {
    fn render(&self) {
        if self.elapsed < self.duration {
            let alpha = self.intensity * (1.0 - self.elapsed.as_secs_f32() / self.duration.as_secs_f32());
            draw_rectangle(
                0.0, 0.0,
                screen_width(), screen_height(),
                Color::new(self.color.r, self.color.g, self.color.b, alpha)
            );
        }
    }
}
```

### 5.2 Note Hit Effects

**Goal**: Make hitting notes feel satisfying.

**Perfect Hit** (using existing `PlyParticleRenderer`):
```rust
fn perfect_hit_effect(particles: &mut PlyParticleRenderer, x: f32, y: f32) {
    // Particle explosion
    particles.spawn(x, y, Color::rgb(1.0, 0.84, 0.0), 20);
    
    // Ring burst (draw_circle with expanding radius)
    // Score popup (draw_text that floats up)
}
```

### 5.3 Streak Effects

**Goal**: Make long streaks feel rewarding.

**Milestone Effects**:
```rust
fn streak_milestone_effect(
    particles: &mut PlyParticleRenderer,
    streak: u32,
    screen_flash: &mut ScreenFlash,
    screen_shake: &mut ScreenShake,
) {
    match streak {
        10 => {
            // "2× Multiplier!" text
            particles.spawn(screen_width() / 2.0, 100.0, Color::GREEN, 10);
        },
        30 => {
            // "4× Multiplier!" text
            particles.spawn(screen_width() / 2.0, 100.0, Color::BLUE, 15);
        },
        50 => {
            // "8× Multiplier!" text
            particles.spawn(screen_width() / 2.0, 100.0, Color::GOLD, 20);
            *screen_flash = ScreenFlash {
                color: Color::GOLD,
                intensity: 0.3,
                duration: Duration::from_millis(200),
                elapsed: Duration::ZERO,
            };
        },
        100 => {
            // "ON FIRE!" text
            particles.spawn(screen_width() / 2.0, 100.0, Color::ORANGE, 30);
            *screen_shake = ScreenShake {
                intensity: 0.5,
                duration: Duration::from_millis(300),
                elapsed: Duration::ZERO,
                frequency: 20.0,
            };
        },
        _ => {},
    }
}
```

---

## Additional Game Mechanics (Genre Best Practices)

### 6.1 Section Retry

**Goal**: Practice specific sections without restarting entire song.

**Mechanics**:
- Mark sections during playback
- Loop marked section
- Track section-specific scores

```rust
struct SectionRetry {
    section_start: Duration,
    section_end: Duration,
    attempts: Vec<SectionAttempt>,
    best_score: u64,
}

struct SectionAttempt {
    score: u64,
    accuracy: f64,
    max_streak: u32,
    timestamp: DateTime<Utc>,
}
```

### 6.2 Rhythm Accuracy Bonus

**Goal**: Reward consistent rhythm, not just note accuracy.

**Mechanics**:
- Track timing consistency (standard deviation of hit timing)
- Bonus points for consistent rhythm
- Separate "Rhythm Score" displayed alongside accuracy

```rust
struct RhythmTracker {
    hit_times: Vec<Duration>,
    consistency_score: f32,
}

impl RhythmTracker {
    fn calculate_consistency(&self) -> f32 {
        if self.hit_times.len() < 2 {
            return 1.0;
        }
        
        let mean = self.hit_times.iter().sum::<Duration>() / self.hit_times.len() as u32;
        let variance = self.hit_times.iter()
            .map(|t| {
                let diff = if *t > mean { *t - mean } else { mean - *t };
                diff.as_secs_f32().powi(2)
            })
            .sum::<f32>() / self.hit_times.len() as f32;
        
        // Lower variance = higher consistency
        1.0 / (1.0 + variance * 100.0)
    }
}
```

---

## Implementation Priority & Timeline (AI Agent)

### Phase 1: Core Scoring & Feedback — **3-4 days**
1. Live score display during gameplay
2. Timing quality feedback (Perfect/Good/Okay/Miss)
3. Basic streak system with multipliers
4. Star rating system (replacing letter grades)

### Phase 2: Replayability — **3-4 days**
1. High score tracking per song
2. Achievement system
3. Daily/weekly challenges
4. Enhanced song library UI

### Phase 3: Difficulty & Tracks — **2-3 days**
1. Difficulty filtering (Easy/Medium/Hard/Expert)
2. Enhanced difficulty rating
3. Song library enhancements

### Phase 4: Learning System — **4-5 days**
1. Skill tree / learning paths
2. Practice modes (Section Loop, Slow Motion)
3. Adaptive difficulty
4. Skill metrics & progress tracking
5. Guided practice sessions
6. Weekly learning goals

### Phase 5: Visual Polish — **2-3 days**
1. Screen effects (shake, flash)
2. Note hit particle effects
3. Streak milestone effects

### Phase 6: Advanced Features — **2-3 days**
1. Section retry
2. Rhythm accuracy bonus

**Total Estimated Time: 16-22 days (AI agent)**

---

## Technical Implementation Details

### New Files to Create (PLY/Macroquad only)
```
neothesia/src/
├── scoring/
│   ├── mod.rs              # Scoring module
│   ├── live_score.rs       # Real-time score tracking
│   ├── streak.rs           # Streak/combo system
│   └── rhythm.rs           # Rhythm accuracy tracking
├── achievements/
│   ├── mod.rs              # Achievement system
│   ├── definitions.rs      # Achievement definitions
│   └── tracker.rs          # Progress tracking
├── challenges/
│   ├── mod.rs              # Challenge system
│   ├── daily.rs            # Daily challenges
│   └── weekly.rs           # Weekly challenges
├── learning/
│   ├── mod.rs              # Learning system
│   ├── paths.rs            # Learning paths
│   ├── practice.rs         # Practice modes
│   ├── adaptive.rs         # Adaptive difficulty
│   └── metrics.rs          # Skill metrics
└── effects/
    ├── mod.rs              # Effects module
    ├── screen_shake.rs     # Screen shake system
    ├── screen_flash.rs     # Screen flash system
    └── hit_effects.rs      # Note hit effects
```

### Rendering (Macroquad Only)
```rust
// All rendering uses Macroquad primitives
use macroquad::prelude::*;

// Text rendering
draw_text(&score_text, x, y, font_size, color);

// Rectangle rendering (for UI panels, progress bars)
draw_rectangle(x, y, w, h, color);

// Circle rendering (for particles, effects)
draw_circle(x, y, radius, color);

// Line rendering (for guidelines, streak indicators)
draw_line(x1, y1, x2, y2, thickness, color);

// Camera shake
let mut camera = Camera2D::default();
camera.offset = vec2(shake_x, shake_y);
set_camera(&camera);
```

### Database Schema Updates
```sql
-- Add to song_library.db
ALTER TABLE songs ADD COLUMN best_stars INTEGER DEFAULT 0;
ALTER TABLE songs ADD COLUMN best_streak INTEGER DEFAULT 0;
ALTER TABLE songs ADD COLUMN adaptive_difficulty TEXT DEFAULT 'Medium';

CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    unlocked_at TIMESTAMP,
    progress REAL DEFAULT 0.0
);

CREATE TABLE challenges (
    id TEXT PRIMARY KEY,
    type TEXT,
    target REAL,
    progress REAL DEFAULT 0.0,
    completed BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMP
);

CREATE TABLE high_scores (
    song_id INTEGER,
    score INTEGER,
    accuracy REAL,
    streak INTEGER,
    stars INTEGER,
    played_at TIMESTAMP,
    FOREIGN KEY (song_id) REFERENCES songs(id)
);

CREATE TABLE learning_progress (
    path_id TEXT,
    stage_id TEXT,
    completed BOOLEAN DEFAULT FALSE,
    completed_at TIMESTAMP,
    PRIMARY KEY (path_id, stage_id)
);

CREATE TABLE skill_metrics (
    date DATE PRIMARY KEY,
    avg_accuracy REAL,
    avg_streak INTEGER,
    max_streak INTEGER,
    songs_played INTEGER,
    total_play_time INTEGER
);
```

### Configuration Additions
```rust
// In config/mod.rs
pub struct GamificationConfig {
    pub enable_live_score: bool,
    pub enable_streak_effects: bool,
    pub enable_achievements: bool,
    pub enable_challenges: bool,
    pub enable_learning_paths: bool,
    pub enable_adaptive_difficulty: bool,
    pub difficulty_filter: DifficultyFilter,
    pub score_display_position: ScorePosition,
    pub effect_intensity: f32,  // 0.0 - 1.0
    pub practice_speed: f32,
    pub show_finger_positions: bool,
}

pub enum ScorePosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}
```

---

## Testing Strategy

### Unit Tests
- Score calculation accuracy
- Streak multiplier logic
- Achievement condition checking
- Adaptive difficulty algorithm
- Rhythm consistency calculation

### Integration Tests
- Score persistence across sessions
- High score tracking
- Challenge progress tracking
- Learning path progression

### Performance Tests
- Effect rendering at 60fps (Macroquad)
- Database query performance
- Memory usage with many particles

---

## Success Metrics

### Engagement
- **Replay Rate**: % of players who replay a song within 7 days
- **Session Length**: Average play time per session
- **Completion Rate**: % of songs started that are finished

### Progression
- **Star Distribution**: How many stars players typically earn
- **Streak Achievement**: Average max streak per player
- **Challenge Completion**: % of daily/weekly challenges completed
- **Learning Path Progress**: Average stage completion rate

### Learning
- **Accuracy Improvement**: % improvement over 30 days
- **Difficulty Progression**: % of players advancing to harder difficulties
- **Practice Mode Usage**: % of sessions using practice modes
- **Skill Metric Trends**: Improvement in tracked metrics

### Retention
- **Daily Active Users**: Players returning each day
- **Weekly Retention**: % of players returning after 7 days
- **Feature Usage**: Adoption of new gamification features

---

## Conclusion

This gamification plan transforms Neothesia from a MIDI visualizer into a compelling rhythm game that encourages:

1. **Mastery**: Through streaks, stars, and difficulty progression
2. **Replayability**: Via high scores, achievements, and challenges
3. **Satisfaction**: With immediate visual/audio feedback (Macroquad)
4. **Progression**: Through unlockable cosmetics and learning paths
5. **Learning**: Structured skill development with adaptive difficulty

The implementation is designed for PLY (Macroquad) rendering only, modular for incremental implementation, and optimized for AI agent development timeline. Each phase builds upon the previous, creating a cohesive gamification system that respects the app's educational roots while adding engaging game mechanics.
