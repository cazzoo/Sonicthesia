//! PLY-based keyboard renderer for Neothesia
//! 
//! This module provides a bridge between Neothesia's existing keyboard rendering
//! and the PLY engine, allowing for gradual migration.

use piano_layout::KeyboardLayout;
use neothesia_core::config::Config;

/// PLY-based keyboard renderer for Neothesia
pub struct PlyKeyboardRenderer {
    /// Keyboard layout
    layout: KeyboardLayout,
    /// Key states (simplified version for PLY integration)
    key_states: Vec<KeyState>,
    /// Position of the keyboard
    pos: (f32, f32),
    /// Whether the renderer is initialized
    initialized: bool,
}

impl PlyKeyboardRenderer {
    /// Create a new PLY keyboard renderer
    pub fn new(layout: KeyboardLayout) -> Self {
        let key_states: Vec<KeyState> = layout
            .range
            .iter()
            .map(|id| KeyState::new(id.is_black()))
            .collect();

        Self {
            layout,
            key_states,
            pos: (0.0, 0.0),
            initialized: false,
        }
    }

    /// Initialize the renderer
    pub fn initialize(&mut self, pos: (f32, f32)) {
        self.pos = pos;
        self.initialized = true;
    }

    /// Update the keyboard state
    pub fn update(&mut self) {
        if !self.initialized {
            return;
        }
        // Update keyboard state
        // In a full PLY implementation, we'd update PLY entities here
    }

    /// Get mutable reference to key states
    pub fn key_states_mut(&mut self) -> &mut [KeyState] {
        &mut self.key_states
    }

    /// Get reference to key states
    pub fn key_states(&self) -> &[KeyState] {
        &self.key_states
    }

    /// Get keyboard layout
    pub fn layout(&self) -> &KeyboardLayout {
        &self.layout
    }

    /// Set keyboard layout
    pub fn set_layout(&mut self, layout: KeyboardLayout) {
        // Rebuild key states for new layout first
        let key_states: Vec<KeyState> = layout
            .range
            .iter()
            .map(|id| KeyState::new(id.is_black()))
            .collect();
        
        self.layout = layout;
        self.key_states = key_states;
    }

    /// Get keyboard position
    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    /// Set keyboard position
    pub fn set_pos(&mut self, pos: (f32, f32)) {
        self.pos = pos;
    }

    /// Reset all notes
    pub fn reset_notes(&mut self) {
        for key in self.key_states.iter_mut() {
            key.pressed_by_file_off();
        }
    }
}

/// Simplified key state for PLY integration
#[derive(Clone, Debug)]
pub struct KeyState {
    /// Whether this is a black key
    is_black: bool,
    /// Color when pressed by user (RGB)
    pressed_by_user: Option<[f32; 4]>,
    /// Color when pressed by file (RGB)
    pressed_by_file: Option<[f32; 4]>,
}

impl KeyState {
    /// Create a new key state
    pub fn new(is_black: bool) -> Self {
        Self {
            is_black,
            pressed_by_user: None,
            pressed_by_file: None,
        }
    }

    /// Get color based on state
    pub fn color(&self) -> [f32; 4] {
        if let Some(color) = self.pressed_by_user {
            return color;
        }
        
        if let Some(color) = self.pressed_by_file {
            return color;
        }
        
        // Default key color
        if self.is_black {
            [0.1, 0.1, 0.1, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    }

    /// Set pressed by user
    pub fn pressed_by_user_on(&mut self, color: [f32; 4]) {
        self.pressed_by_user = Some(color);
    }

    /// Set pressed by file
    pub fn pressed_by_file_on(&mut self, color: [f32; 4]) {
        self.pressed_by_file = Some(color);
    }

    /// Turn off pressed by user
    pub fn pressed_by_user_off(&mut self) {
        self.pressed_by_user = None;
    }

    /// Turn off pressed by file
    pub fn pressed_by_file_off(&mut self) {
        self.pressed_by_file = None;
    }

    /// Get pressed by user color
    pub fn pressed_by_user(&self) -> Option<[f32; 4]> {
        self.pressed_by_user
    }

    /// Get pressed by file color
    pub fn pressed_by_file(&self) -> Option<[f32; 4]> {
        self.pressed_by_file
    }

    /// Check if this is a black key
    pub fn is_black(&self) -> bool {
        self.is_black
    }
}
