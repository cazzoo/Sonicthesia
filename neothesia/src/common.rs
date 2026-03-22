//! Common types used by Neothesia

use midi_file::midly::MidiMessage;

use crate::scoring_data::ScoreData;

/// Events that can be sent to the Neothesia application
#[derive(Debug)]
pub enum NeothesiaEvent {
    Play(crate::Song),
    FreePlay(Option<crate::Song>),
    MainMenu(Option<crate::Song>),
    ShowSettings,
    ShowSongLibrary(Option<crate::Song>),
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
