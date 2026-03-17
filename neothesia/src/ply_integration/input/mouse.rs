//! PLY Mouse Input Handler
//!
//! Handles mouse input events and maps them to Neothesia actions.

use std::collections::HashSet;

use winit::{
    dpi::Position,
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};

use crate::NeothesiaEvent;
use super::NeothesiaAction;

/// Mouse input handler
pub struct PlyMouseHandler {
    /// Current cursor position (logical coordinates)
    cursor_pos: (f32, f32),
    /// Previous cursor position
    prev_cursor_pos: (f32, f32),
    /// Cursor delta (movement since last frame)
    cursor_delta: (f32, f32),
    /// Currently pressed mouse buttons
    pressed_buttons: HashSet<MouseButton>,
    /// Buttons that were just pressed this frame
    just_pressed: HashSet<MouseButton>,
    /// Buttons that were just released this frame
    just_released: HashSet<MouseButton>,
    /// Current scroll wheel delta
    scroll_delta: f32,
    /// Scale factor for converting physical to logical pixels
    scale_factor: f32,
}

impl PlyMouseHandler {
    /// Create a new mouse handler
    pub fn new() -> Self {
        Self {
            cursor_pos: (0.0, 0.0),
            prev_cursor_pos: (0.0, 0.0),
            cursor_delta: (0.0, 0.0),
            pressed_buttons: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            scroll_delta: 0.0,
            scale_factor: 1.0,
        }
    }

    /// Handle a mouse button event
    pub fn handle_button_event(
        &mut self,
        state: ElementState,
        button: MouseButton,
        reverse_bindings: &std::collections::HashMap<String, NeothesiaAction>,
        proxy: &EventLoopProxy<NeothesiaEvent>,
    ) {
        let button_str = format!("mouse:{:?}", button);

        match state {
            ElementState::Pressed => {
                if self.pressed_buttons.insert(button) {
                    self.just_pressed.insert(button);
                }
            }
            ElementState::Released => {
                if self.pressed_buttons.remove(&button) {
                    self.just_released.insert(button);
                }
            }
        }

        // Map to Neothesia action if binding exists
        if let Some(&action) = reverse_bindings.get(&button_str) {
            if state == ElementState::Pressed {
                self.trigger_action(action, proxy);
            }
        }
    }

    /// Handle cursor movement event
    pub fn handle_move_event(&mut self, position: &Position) {
        let logical_pos = match position {
            Position::Physical(physical_pos) => (
                physical_pos.x as f32 / self.scale_factor,
                physical_pos.y as f32 / self.scale_factor,
            ),
            Position::Logical(logical_pos) => (logical_pos.x as f32, logical_pos.y as f32),
        };

        self.prev_cursor_pos = self.cursor_pos;
        self.cursor_pos = logical_pos;
        self.cursor_delta = (
            self.cursor_pos.0 - self.prev_cursor_pos.0,
            self.cursor_pos.1 - self.prev_cursor_pos.1,
        );
    }

    /// Handle cursor movement event from PhysicalPosition
    pub fn handle_move_physical(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
        let logical_pos = (
            position.x as f32 / self.scale_factor,
            position.y as f32 / self.scale_factor,
        );

        self.prev_cursor_pos = self.cursor_pos;
        self.cursor_pos = logical_pos;
        self.cursor_delta = (
            self.cursor_pos.0 - self.prev_cursor_pos.0,
            self.cursor_pos.1 - self.prev_cursor_pos.1,
        );
    }

    /// Handle mouse wheel event
    pub fn handle_wheel_event(
        &mut self,
        delta: &MouseScrollDelta,
        reverse_bindings: &std::collections::HashMap<String, NeothesiaAction>,
        proxy: &EventLoopProxy<NeothesiaEvent>,
    ) {
        let delta_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y * 60.0,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };

        self.scroll_delta = delta_y;

        // Map scroll to actions
        if delta_y > 0.0 {
            if let Some(&action) = reverse_bindings.get("scroll:up") {
                self.trigger_action(action, proxy);
            }
        } else if delta_y < 0.0 {
            if let Some(&action) = reverse_bindings.get("scroll:down") {
                self.trigger_action(action, proxy);
            }
        }
    }

    /// Trigger a Neothesia action
    fn trigger_action(&self, _action: NeothesiaAction, _proxy: &EventLoopProxy<NeothesiaEvent>) {
        // Mouse button actions are handled by the UI system
        // This is a placeholder for future mouse-based game actions
    }

    /// Update mouse state (call each frame)
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.cursor_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
    }

    /// Set the scale factor for pixel conversion
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Get current cursor position (logical coordinates)
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.cursor_pos
    }

    /// Get cursor delta (movement since last frame)
    pub fn cursor_delta(&self) -> (f32, f32) {
        self.cursor_delta
    }

    /// Check if a mouse button is currently pressed
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Get scroll delta
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Consume scroll delta
    pub fn consume_scroll(&mut self) -> f32 {
        let delta = self.scroll_delta;
        self.scroll_delta = 0.0;
        delta
    }

    /// Check if any mouse button is pressed
    pub fn any_button_pressed(&self) -> bool {
        !self.pressed_buttons.is_empty()
    }

    /// Check if left mouse button is pressed
    pub fn is_left_pressed(&self) -> bool {
        self.is_button_pressed(MouseButton::Left)
    }

    /// Check if right mouse button is pressed
    pub fn is_right_pressed(&self) -> bool {
        self.is_button_pressed(MouseButton::Right)
    }

    /// Check if middle mouse button is pressed
    pub fn is_middle_pressed(&self) -> bool {
        self.is_button_pressed(MouseButton::Middle)
    }
}

impl Default for PlyMouseHandler {
    fn default() -> Self {
        Self::new()
    }
}
