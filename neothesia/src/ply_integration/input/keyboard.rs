//! PLY Keyboard Input Handler
//!
//! Handles keyboard input events and maps them to Neothesia actions.

use std::collections::HashSet;

use winit::{
    event::{ElementState, KeyEvent},
    event_loop::EventLoopProxy,
    keyboard::Key,
};

use crate::NeothesiaEvent;
use super::NeothesiaAction;

/// Keyboard input handler
pub struct PlyKeyboardHandler {
    /// Currently pressed keys
    pressed_keys: HashSet<Key>,
    /// Keys that were just pressed this frame
    just_pressed: HashSet<Key>,
    /// Keys that were just released this frame
    just_released: HashSet<Key>,
}

impl PlyKeyboardHandler {
    /// Create a new keyboard handler
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    /// Handle a keyboard event
    pub fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        reverse_bindings: &std::collections::HashMap<String, NeothesiaAction>,
        proxy: &EventLoopProxy<NeothesiaEvent>,
    ) {
        let key_str = format!("key:{}", key_to_string(&event.logical_key));

        match event.state {
            ElementState::Pressed => {
                if !event.repeat && self.pressed_keys.insert(event.logical_key.clone()) {
                    self.just_pressed.insert(event.logical_key.clone());
                }
            }
            ElementState::Released => {
                if self.pressed_keys.remove(&event.logical_key) {
                    self.just_released.insert(event.logical_key.clone());
                }
            }
        }

        // Map to Neothesia action if binding exists
        if let Some(&action) = reverse_bindings.get(&key_str) {
            if event.state == ElementState::Pressed && !event.repeat {
                self.trigger_action(action, proxy);
            }
        }
    }

    /// Trigger a Neothesia action
    fn trigger_action(&self, action: NeothesiaAction, proxy: &EventLoopProxy<NeothesiaEvent>) {
        // Convert action to NeothesiaEvent
        let event = match action {
            NeothesiaAction::Quit => Some(NeothesiaEvent::Exit),
            _ => None,
        };

        if let Some(event) = event {
            proxy.send_event(event).ok();
        }
    }

    /// Update keyboard state (call each frame)
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: &Key) -> bool {
        self.just_pressed.contains(key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: &Key) -> bool {
        self.just_released.contains(key)
    }

    /// Get all currently pressed keys
    pub fn pressed_keys(&self) -> &HashSet<Key> {
        &self.pressed_keys
    }

    /// Check if any key is pressed
    pub fn any_key_pressed(&self) -> bool {
        !self.pressed_keys.is_empty()
    }

    /// Check if a specific character key is pressed
    pub fn is_char_pressed(&self, ch: char) -> bool {
        self.pressed_keys.iter().any(|k| {
            if let Key::Character(c) = k {
                c.as_str().chars().next() == Some(ch)
            } else {
                false
            }
        })
    }

    /// Check if a modifier key is pressed
    pub fn is_modifier_pressed(&self, modifier: KeyModifier) -> bool {
        match modifier {
            KeyModifier::Shift => self.is_key_pressed(&Key::Named(winit::keyboard::NamedKey::Shift)),
            KeyModifier::Control => self.is_key_pressed(&Key::Named(winit::keyboard::NamedKey::Control)),
            KeyModifier::Alt => self.is_key_pressed(&Key::Named(winit::keyboard::NamedKey::Alt)),
            KeyModifier::Super => {
                self.is_key_pressed(&Key::Named(winit::keyboard::NamedKey::Super))
            }
        }
    }
}

/// Modifier keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyModifier {
    Shift,
    Control,
    Alt,
    Super,
}

/// Convert a winit Key to a string for hashing
fn key_to_string(key: &Key) -> String {
    match key {
        Key::Named(named) => format!("named:{:?}", named),
        Key::Character(ch) => format!("char:{}", ch),
        Key::Unidentified(_) => "unidentified".to_string(),
        Key::Dead(dead) => format!("dead:{:?}", dead),
        _ => "unknown".to_string(),
    }
}

impl Default for PlyKeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}
