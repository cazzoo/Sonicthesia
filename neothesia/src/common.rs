//! Common types used by Neothesia

use midi_file::midly::MidiMessage;

use crate::scoring_data::ScoreData;
use crate::song_library::SongEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Learn,
    Practice,
    Play,
}

impl PlayMode {
    pub fn label(&self) -> &'static str {
        match self {
            PlayMode::Learn => "Learn",
            PlayMode::Practice => "Practice",
            PlayMode::Play => "Play",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PlayMode::Learn => "🎓",
            PlayMode::Practice => "🎹",
            PlayMode::Play => "🎮",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl DifficultyLevel {
    pub fn label(&self) -> &'static str {
        match self {
            DifficultyLevel::Easy => "EASY",
            DifficultyLevel::Medium => "MED",
            DifficultyLevel::Hard => "HARD",
        }
    }

    pub fn to_track_count_modifier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 0.4,
            DifficultyLevel::Medium => 0.7,
            DifficultyLevel::Hard => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSelection {
    Left,
    Right,
    Both,
}

impl HandSelection {
    pub fn label(&self) -> &'static str {
        match self {
            HandSelection::Left => "Left",
            HandSelection::Right => "Right",
            HandSelection::Both => "Both",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub mode: PlayMode,
    pub difficulty: DifficultyLevel,
    pub hand_selection: HandSelection,
    pub speed: f32,
    pub midi_gain: f32,
    pub keyboard_enabled: bool,
    pub midi_enabled: bool,
    pub fingering_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            mode: PlayMode::Practice,
            difficulty: DifficultyLevel::Medium,
            hand_selection: HandSelection::Both,
            speed: 1.0,
            midi_gain: 1.0,
            keyboard_enabled: true,
            midi_enabled: true,
            fingering_enabled: true,
        }
    }
}

impl SessionConfig {
    pub fn for_mode(mode: PlayMode) -> Self {
        let mut config = Self::default();
        config.mode = mode;
        match mode {
            PlayMode::Learn => {
                config.speed = 0.75;
                config.midi_enabled = true;
                config.keyboard_enabled = true;
            }
            PlayMode::Practice => {
                config.speed = 1.0;
                config.midi_enabled = true;
                config.keyboard_enabled = true;
            }
            PlayMode::Play => {
                config.speed = 1.0;
                config.midi_enabled = false;
                config.keyboard_enabled = true;
                config.hand_selection = HandSelection::Both;
            }
        }
        config
    }
}

/// Events that can be sent to the Neothesia application
#[derive(Debug)]
pub enum NeothesiaEvent {
    StartSession {
        song: crate::Song,
        config: SessionConfig,
    },
    ResumePlay(crate::Song, f32),
    FreePlay(Option<crate::Song>),
    MainMenu(Option<crate::Song>),
    ShowSettings,
    ResumeFromSettings,
    ShowSongLibrary(Option<crate::Song>),
    ShowSongSelected {
        song: crate::Song,
        entry: SongEntry,
    },
    ShowScore {
        song: crate::Song,
        score_data: ScoreData,
    },
    MidiInput {
        channel: u8,
        message: MidiMessage,
    },
    Exit,
}
