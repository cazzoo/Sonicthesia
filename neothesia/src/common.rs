//! Common types used by Neothesia

use midi_file::midly::MidiMessage;

use crate::scoring_data::ScoreData;
use crate::song_library::SongEntry;

/// Events that can be sent to the Neothesia application
#[derive(Debug)]
pub enum NeothesiaEvent {
    Play(crate::Song),
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
