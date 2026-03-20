//! PLY UI Input Handler
//!
//! This module provides input handling for the PLY UI system,
//! integrating with the main PLY input handler.

use crate::context::Context;
use winit::event::{WindowEvent, MouseButton};

use super::super::input::{PlyInputHandler as PlyGameInputHandler, NeothesiaAction};

/// Input handler for PLY UI
pub struct PlyInputHandler {
    /// Current cursor position
    cursor_pos: (f32, f32),
    
    /// Mouse button state
    mouse_pressed: bool,
    mouse_down: bool,
    
    /// Scroll wheel delta
    scroll_delta: f32,
    
    /// Reference to the main game input handler
    game_input: *const PlyGameInputHandler,
}

// SAFETY: We only use this pointer for read access and ensure the reference is valid
unsafe impl Send for PlyInputHandler {}
unsafe impl Sync for PlyInputHandler {}

impl PlyInputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            cursor_pos: (0.0, 0.0),
            mouse_pressed: false,
            mouse_down: false,
            scroll_delta: 0.0,
            game_input: std::ptr::null(),
        }
    }
    
    /// Set the game input handler reference
    pub fn set_game_input(&mut self, game_input: &PlyGameInputHandler) {
        self.game_input = game_input;
    }
    
    /// Handle a window event and update input state
    pub fn handle_event(&mut self, event: &WindowEvent, ctx: &Context) -> InputEvent {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (
                    position.x as f32 / ctx.window_state.scale_factor as f32,
                    position.y as f32 / ctx.window_state.scale_factor as f32,
                );
                InputEvent::MouseMoved(self.cursor_pos.0, self.cursor_pos.1)
            }

            WindowEvent::MouseInput { state, button, .. } if *button == MouseButton::Left => {
                match state {
                    winit::event::ElementState::Pressed => {
                        self.mouse_pressed = true;
                        self.mouse_down = true;
                        InputEvent::MousePressed
                    }
                    winit::event::ElementState::Released => {
                        self.mouse_pressed = false;
                        self.mouse_down = false;
                        InputEvent::MouseReleased
                    }
                }
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 60.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.scroll_delta = delta;
                InputEvent::MouseScrolled(delta)
            }
            
            _ => InputEvent::None,
        }
    }
    
    /// Get current cursor position
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.cursor_pos
    }
    
    /// Check if mouse is pressed
    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_pressed
    }
    
    /// Check if mouse is down
    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down
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
    
    /// Reset mouse pressed state (call after processing)
    pub fn reset_pressed(&mut self) {
        self.mouse_pressed = false;
    }
    
    /// Check if a Neothesia action is active
    pub fn is_action_active(&self, action: NeothesiaAction) -> bool {
        if !self.game_input.is_null() {
            unsafe {
                if let Some(game_input) = self.game_input.as_ref() {
                    return game_input.is_action_active(action);
                }
            }
        }
        false
    }
}

impl Default for PlyInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Input event type
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    None,
    MouseMoved(f32, f32),
    MousePressed,
    MouseReleased,
    MouseScrolled(f32),
    KeyPressed(winit::keyboard::Key),
    KeyReleased(winit::keyboard::Key),
}

/// Helper trait for checking window events
pub trait WindowEventExt {
    /// Check if cursor moved
    fn cursor_moved(&self) -> bool;
    
    /// Check if left mouse button was pressed
    fn left_mouse_pressed(&self) -> bool;
    
    /// Check if left mouse button was released
    fn left_mouse_released(&self) -> bool;
    
    /// Check if a specific key was pressed
    fn key_pressed(&self, key: winit::keyboard::Key) -> bool;
    
    /// Check if back button was pressed
    fn back_mouse_pressed(&self) -> bool;
}

impl WindowEventExt for WindowEvent {
    fn cursor_moved(&self) -> bool {
        matches!(self, WindowEvent::CursorMoved { .. })
    }
    
    fn left_mouse_pressed(&self) -> bool {
        matches!(
            self,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            }
        )
    }
    
    fn left_mouse_released(&self) -> bool {
        matches!(
            self,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
                ..
            }
        )
    }
    
    fn key_pressed(&self, key: winit::keyboard::Key) -> bool {
        matches!(
            self,
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    logical_key: k,
                    ..
                },
                ..
            } if k == &key
        )
    }
    
    fn back_mouse_pressed(&self) -> bool {
        matches!(
            self,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Back,
                ..
            }
        )
    }
}
