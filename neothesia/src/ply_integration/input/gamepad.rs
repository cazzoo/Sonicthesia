//! PLY Gamepad Input Handler
//!
//! Handles gamepad/controller input events and maps them to Neothesia actions.

use std::collections::{HashMap, HashSet};

use winit::event_loop::EventLoopProxy;

use crate::NeothesiaEvent;
use super::{GamepadButton, NeothesiaAction};

/// Gamepad connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadConnection {
    Connected,
    Disconnected,
}

/// Gamepad state
#[derive(Debug, Clone)]
struct GamepadState {
    /// Currently pressed buttons
    pressed_buttons: HashSet<GamepadButton>,
    /// Buttons that were just pressed this frame
    just_pressed: HashSet<GamepadButton>,
    /// Buttons that were just released this frame
    just_released: HashSet<GamepadButton>,
    /// Left stick position (-1.0 to 1.0)
    left_stick: (f32, f32),
    /// Right stick position (-1.0 to 1.0)
    right_stick: (f32, f32),
    /// Left trigger value (0.0 to 1.0)
    left_trigger: f32,
    /// Right trigger value (0.0 to 1.0)
    right_trigger: f32,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            pressed_buttons: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            left_stick: (0.0, 0.0),
            right_stick: (0.0, 0.0),
            left_trigger: 0.0,
            right_trigger: 0.0,
        }
    }
}

/// Gamepad input handler
pub struct PlyGamepadHandler {
    /// Connected gamepads and their states
    gamepads: HashMap<usize, GamepadState>,
    /// Next available gamepad ID
    next_id: usize,
}

impl PlyGamepadHandler {
    /// Create a new gamepad handler
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
            next_id: 0,
        }
    }

    /// Handle gamepad connection event
    pub fn handle_connection_event(&mut self, id: usize, connected: bool) {
        if connected {
            log::info!("Gamepad {} connected", id);
            self.gamepads.insert(id, GamepadState::default());
        } else {
            log::info!("Gamepad {} disconnected", id);
            self.gamepads.remove(&id);
        }
    }

    /// Handle gamepad button event
    pub fn handle_button_event(
        &mut self,
        id: usize,
        button: GamepadButton,
        pressed: bool,
        reverse_bindings: &HashMap<String, NeothesiaAction>,
        proxy: &EventLoopProxy<NeothesiaEvent>,
    ) {
        if let Some(state) = self.gamepads.get_mut(&id) {
            let button_str = format!("gamepad:{:?}", button);

            if pressed {
                if state.pressed_buttons.insert(button) {
                    state.just_pressed.insert(button);
                }
            } else {
                if state.pressed_buttons.remove(&button) {
                    state.just_released.insert(button);
                }
            }

            // Map to Neothesia action if binding exists
            if pressed {
                if let Some(&action) = reverse_bindings.get(&button_str) {
                    self.trigger_action(action, proxy);
                }
            }
        }
    }

    /// Handle gamepad axis event
    pub fn handle_axis_event(&mut self, id: usize, axis: GamepadAxis, value: f32) {
        if let Some(state) = self.gamepads.get_mut(&id) {
            match axis {
                GamepadAxis::LeftStickX => state.left_stick.0 = value,
                GamepadAxis::LeftStickY => state.left_stick.1 = value,
                GamepadAxis::RightStickX => state.right_stick.0 = value,
                GamepadAxis::RightStickY => state.right_stick.1 = value,
                GamepadAxis::LeftTrigger => state.left_trigger = value,
                GamepadAxis::RightTrigger => state.right_trigger = value,
            }
        }
    }

    /// Trigger a Neothesia action
    fn trigger_action(&self, _action: NeothesiaAction, _proxy: &EventLoopProxy<NeothesiaEvent>) {
        // Gamepad actions are handled by the input system
        // This is a placeholder for future gamepad-based actions
    }

    /// Update gamepad state (call each frame)
    pub fn update(&mut self, reverse_bindings: &HashMap<String, NeothesiaAction>, proxy: &EventLoopProxy<NeothesiaEvent>) {
        // Clear just pressed/released states
        for state in self.gamepads.values_mut() {
            state.just_pressed.clear();
            state.just_released.clear();
        }

        // Check for continuous actions (like stick movement)
        for (id, state) in &self.gamepads {
            // Left stick navigation
            if state.left_stick.1 < -0.5 {
                if let Some(&action) = reverse_bindings.get("stick:left_up") {
                    self.trigger_action_for_id(*id, action, proxy);
                }
            } else if state.left_stick.1 > 0.5 {
                if let Some(&action) = reverse_bindings.get("stick:left_down") {
                    self.trigger_action_for_id(*id, action, proxy);
                }
            }

            if state.left_stick.0 < -0.5 {
                if let Some(&action) = reverse_bindings.get("stick:left_left") {
                    self.trigger_action_for_id(*id, action, proxy);
                }
            } else if state.left_stick.0 > 0.5 {
                if let Some(&action) = reverse_bindings.get("stick:left_right") {
                    self.trigger_action_for_id(*id, action, proxy);
                }
            }
        }
    }

    /// Trigger an action for a specific gamepad
    fn trigger_action_for_id(&self, _id: usize, _action: NeothesiaAction, _proxy: &EventLoopProxy<NeothesiaEvent>) {
        // Placeholder for gamepad-specific actions
    }

    /// Check if a button is currently pressed on any gamepad
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.gamepads.values().any(|state| state.pressed_buttons.contains(&button))
    }

    /// Check if a button is currently pressed on a specific gamepad
    pub fn is_button_pressed_on(&self, id: usize, button: GamepadButton) -> bool {
        self.gamepads
            .get(&id)
            .map(|state| state.pressed_buttons.contains(&button))
            .unwrap_or(false)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.gamepads
            .values()
            .any(|state| state.just_pressed.contains(&button))
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.gamepads
            .values()
            .any(|state| state.just_released.contains(&button))
    }

    /// Get the number of connected gamepads
    pub fn connected_count(&self) -> usize {
        self.gamepads.len()
    }

    /// Check if any gamepad is connected
    pub fn any_connected(&self) -> bool {
        !self.gamepads.is_empty()
    }

    /// Get left stick position for a gamepad
    pub fn left_stick(&self, id: usize) -> Option<(f32, f32)> {
        self.gamepads
            .get(&id)
            .map(|state| state.left_stick)
    }

    /// Get right stick position for a gamepad
    pub fn right_stick(&self, id: usize) -> Option<(f32, f32)> {
        self.gamepads
            .get(&id)
            .map(|state| state.right_stick)
    }

    /// Get left trigger value for a gamepad
    pub fn left_trigger(&self, id: usize) -> Option<f32> {
        self.gamepads
            .get(&id)
            .map(|state| state.left_trigger)
    }

    /// Get right trigger value for a gamepad
    pub fn right_trigger(&self, id: usize) -> Option<f32> {
        self.gamepads
            .get(&id)
            .map(|state| state.right_trigger)
    }
}

/// Gamepad axis enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

impl Default for PlyGamepadHandler {
    fn default() -> Self {
        Self::new()
    }
}
