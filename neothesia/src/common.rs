//! Common types used by both WGPU and PLY rendering paths

use midi_file::midly::MidiMessage;

/// Events that can be sent to the Neothesia application
#[derive(Debug)]
pub enum NeothesiaEvent {
    /// Go to playing scene
    Play(crate::Song),
    /// Go to freeplay mode
    FreePlay(Option<crate::Song>),
    /// Go to main menu scene
    MainMenu(Option<crate::Song>),
    /// Show score screen after song completion
    ShowScore {
        song: crate::Song,
        score_data: crate::scene::playing_scene::midi_player::ScoreData,
    },
    /// MIDI input event
    MidiInput {
        /// The MIDI channel that this message is associated with.
        channel: u8,
        /// The MIDI message type and associated data.
        message: MidiMessage,
    },
    /// Exit the application
    Exit,
}
