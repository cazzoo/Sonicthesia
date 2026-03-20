//! PLY-based note labels renderer for Neothesia
//!
//! This module provides a bridge between Neothesia's existing note label rendering
//! and the PLY engine, allowing for gradual migration.

use neothesia_core::render::waterfall::NoteList;
use piano_layout::KeyboardLayout;

/// PLY-based note labels renderer for Neothesia
pub struct PlyNoteLabelsRenderer {
    /// Notes to render labels for
    notes: NoteList,
    /// Position of the labels
    pos: (f32, f32),
    /// Whether the renderer is initialized
    initialized: bool,
}

impl PlyNoteLabelsRenderer {
    /// Create a new PLY note labels renderer
    pub fn new(pos: (f32, f32), notes: &NoteList) -> Self {
        Self {
            notes: notes.clone(),
            pos,
            initialized: false,
        }
    }

    /// Initialize the renderer
    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    /// Update the labels with current time
    pub fn update(
        &mut self,
        layout: &KeyboardLayout,
        animation_speed: f32,
        time: f32,
        scale: f32,
        keyboard_y: f32,
    ) {
        if !self.initialized {
            return;
        }
        // Update labels based on time
        // In a full PLY implementation, we'd update PLY entities here
    }

    /// Set position
    pub fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }

    /// Get position
    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    /// Get the label text for a note
    pub fn note_label(note: u8) -> &'static str {
        match note % 12 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => "",
        }
    }
}
