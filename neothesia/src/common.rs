//! Common types used by both WGPU and PLY rendering paths

use midi_file::midly::MidiMessage;

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
        score_data: crate::scene::playing_scene::midi_player::ScoreData,
    },
    MidiInput {
        channel: u8,
        message: MidiMessage,
    },
    Exit,
}
