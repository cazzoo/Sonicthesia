pub mod ply_fonts;
pub mod ply_scene;
pub mod ply_settings;
pub use ply_scene::{PlyScene, PlyMenuScene, PlyPlayingScene, PlyFreeplayScene, PlyScoreScene, PlySongLibraryScene, PlyNewSongLibraryScene, PlySongSelectedScene};
pub use ply_settings::PlySettingsScene;

use midi_file::midly::MidiMessage;
use crate::NeothesiaEvent;
use winit::event::WindowEvent;

/// Handle PC keyboard input and convert to MIDI events
pub fn handle_pc_keyboard_to_midi_event(
    proxy: &winit::event_loop::EventLoopProxy<NeothesiaEvent>,
    event: &winit::event::WindowEvent,
) {
    use winit::event::{ElementState, KeyEvent};
    use winit::keyboard::Key;

    let WindowEvent::KeyboardInput {
        event:
            KeyEvent {
                state,
                logical_key: Key::Character(ch),
                repeat: false,
                ..
            },
        ..
    } = event
    else {
        return;
    };

    let mut note: u8 = match ch.as_str() {
        "a" => 0,
        "w" => 1,
        "s" => 2,
        "e" => 3,
        "d" => 4,
        "f" => 5,
        "t" => 6,
        "g" => 7,
        "y" => 8,
        "h" => 9,
        "u" => 10,
        "j" => 11,
        "k" => 12,
        "o" => 13,
        "l" => 14,
        "p" => 15,
        ";" => 16,
        "'" => 17,
        _ => return,
    };

    note += 21; // Start of 88 keyboard
    note += 3; // Offset to C
    note += 12 * 3; // Move 3oct up

    let message = match state {
        ElementState::Pressed => MidiMessage::NoteOn {
            key: note.into(),
            vel: 100.into(),
        },
        ElementState::Released => MidiMessage::NoteOff {
            key: note.into(),
            vel: 0.into(),
        },
    };
    proxy
        .send_event(NeothesiaEvent::MidiInput {
            channel: 0,
            message,
        })
        .ok();
}
