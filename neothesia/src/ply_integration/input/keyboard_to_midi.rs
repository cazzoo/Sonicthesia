//! Keyboard to MIDI Input Conversion
//!
//! Converts keyboard input events to MIDI events for Neothesia's piano keyboard.

use midi_file::midly::MidiMessage;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::NeothesiaEvent;

/// Keyboard to MIDI converter
pub struct KeyboardToMidiConverter {
    /// Currently active notes from keyboard input
    active_notes: std::collections::HashSet<u8>,
}

impl KeyboardToMidiConverter {
    /// Create a new keyboard to MIDI converter
    pub fn new() -> Self {
        Self {
            active_notes: std::collections::HashSet::new(),
        }
    }

    /// Handle a keyboard event and convert to MIDI if applicable
    pub fn handle_keyboard_event(
        &mut self,
        event: &WindowEvent,
        proxy: &winit::event_loop::EventLoopProxy<NeothesiaEvent>,
    ) {
        let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
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

        let note = match ch.as_str() {
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

        // Calculate MIDI note number
        let mut midi_note: u8 = note;
        midi_note += 21; // Start of 88 keyboard
        midi_note += 3; // Offset to C
        midi_note += 12 * 3; // Move 3 octaves up

        let message = match state {
            ElementState::Pressed => {
                if !self.active_notes.insert(midi_note) {
                    return; // Already pressed
                }
                MidiMessage::NoteOn {
                    key: midi_note.into(),
                    vel: 100.into(),
                }
            }
            ElementState::Released => {
                if !self.active_notes.remove(&midi_note) {
                    return; // Not active
                }
                MidiMessage::NoteOff {
                    key: midi_note.into(),
                    vel: 0.into(),
                }
            }
        };

        proxy
            .send_event(NeothesiaEvent::MidiInput {
                channel: 0,
                message,
            })
            .ok();
    }

    /// Check if a specific key is a valid piano key
    pub fn is_piano_key(key: &Key) -> bool {
        match key {
            Key::Character(ch) => {
                matches!(
                    ch.as_str(),
                    "a" | "w" | "s" | "e" | "d" | "f" | "t" | "g" | "y" | "h" | "u" | "j" | "k" | "o" | "l" | "p" | ";" | "'"
                )
            }
            _ => false,
        }
    }

    /// Get all currently active notes
    pub fn active_notes(&self) -> &std::collections::HashSet<u8> {
        &self.active_notes
    }

    /// Clear all active notes (call when switching scenes)
    pub fn clear_all_notes(&mut self, proxy: &winit::event_loop::EventLoopProxy<NeothesiaEvent>) {
        for note in self.active_notes.drain() {
            proxy
                .send_event(NeothesiaEvent::MidiInput {
                    channel: 0,
                    message: MidiMessage::NoteOff {
                        key: note.into(),
                        vel: 0.into(),
                    },
                })
                .ok();
        }
    }
}

impl Default for KeyboardToMidiConverter {
    fn default() -> Self {
        Self::new()
    }
}
