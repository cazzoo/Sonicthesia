//! Unified Input Management System
//!
//! Provides a unified interface for managing focus, keyboard, and mouse input
//! with automatic priority switching between input methods.

use super::focus_manager::{FocusManager, FocusAction, KeyboardInput};
use super::keyboard::PlyKeyboardHandler;
use super::mouse::PlyMouseHandler;
use super::priority_manager::InputPriority;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::NamedKey,
};

/// Result of input handling
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    None,
    FocusChanged(String),
    ElementActivated(String),
    ValueAdjusted(String, i32),
    NavigationCancelled,
}

/// Unified input management system
pub struct UnifiedInputManager {
    /// Focus manager
    focus: FocusManager,
    /// Keyboard handler
    keyboard: PlyKeyboardHandler,
    /// Mouse handler
    mouse: PlyMouseHandler,
}

impl UnifiedInputManager {
    /// Create a new unified input manager
    pub fn new() -> Self {
        Self {
            focus: FocusManager::new(),
            keyboard: PlyKeyboardHandler::new(),
            mouse: PlyMouseHandler::new(),
        }
    }

    /// Handle window event
    pub fn handle_event(&mut self, event: &WindowEvent) -> InputAction {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                // Handle mouse movement
                self.mouse.handle_move_physical(position);

                // Update priority manager with mouse position
                let new_pos = self.mouse.cursor_pos();
                if self.focus.priority().update_mouse_position(new_pos.0, new_pos.1) {
                    // Mouse movement detected, clear keyboard hover state
                    self.focus.handle_mouse_move(new_pos.0, new_pos.1);
                } else {
                    // Just update hover state without priority change
                    self.focus.handle_mouse_move(new_pos.0, new_pos.1);
                }

                InputAction::None
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                // Check for navigation keys
                if key_event.state == ElementState::Pressed && !key_event.repeat {
                    let focus_action = match key_event.logical_key {
                        Key::Named(NamedKey::Tab) => {
                            let input = if self.keyboard.is_modifier_pressed(super::keyboard::KeyModifier::Shift) {
                                KeyboardInput::Previous
                            } else {
                                KeyboardInput::Next
                            };
                            self.focus.handle_keyboard_input(input)
                        }
                        Key::Named(NamedKey::ArrowUp) | Key::Named(NamedKey::ArrowLeft) => {
                            self.focus.handle_keyboard_input(KeyboardInput::Previous)
                        }
                        Key::Named(NamedKey::ArrowDown) | Key::Named(NamedKey::ArrowRight) => {
                            self.focus.handle_keyboard_input(KeyboardInput::Next)
                        }
                        Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => {
                            self.focus.handle_keyboard_input(KeyboardInput::Activate)
                        }
                        Key::Named(NamedKey::Escape) => {
                            self.focus.handle_keyboard_input(KeyboardInput::Cancel)
                        }
                        _ => FocusAction::None,
                    };
                    return self.map_focus_action(focus_action);
                }

                InputAction::None
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Check for left click activation
                if *state == ElementState::Pressed && *button == MouseButton::Left {
                    if let Some(hovered) = self.focus.hovered_element() {
                        return InputAction::ElementActivated(hovered.id.clone());
                    }
                }

                InputAction::None
            }
            _ => InputAction::None,
        }
    }

    /// Update state (call each frame)
    pub fn update(&mut self, delta_time: f64) {
        self.focus.update(delta_time);
        self.keyboard.update();
        self.mouse.update();
    }

    /// Get focus manager reference (mutable)
    pub fn focus(&mut self) -> &mut FocusManager {
        &mut self.focus
    }

    /// Get the currently focused element ID (immutable accessor)
    pub fn focused_element_id(&self) -> Option<&str> {
        self.focus.focused_element().map(|e| e.id.as_str())
    }

    /// Get the currently focused element index (immutable accessor)
    pub fn focused_index(&self) -> Option<usize> {
        self.focus.focused_index()
    }

    /// Get keyboard handler reference
    pub fn keyboard(&self) -> &PlyKeyboardHandler {
        &self.keyboard
    }

    /// Get mouse handler reference
    pub fn mouse(&self) -> &PlyMouseHandler {
        &self.mouse
    }

    /// Get current input priority
    pub fn get_priority(&self) -> InputPriority {
        // Access priority through the focus manager's internal state
        // We need to add a method to FocusManager to expose this
        self.focus.get_priority_internal()
    }

    /// Map focus action to input action
    fn map_focus_action(&self, action: FocusAction) -> InputAction {
        match action {
            FocusAction::None => InputAction::None,
            FocusAction::FocusChanged(id) => InputAction::FocusChanged(id),
            FocusAction::Activated(id) => InputAction::ElementActivated(id),
            FocusAction::Adjusted(id, delta) => InputAction::ValueAdjusted(id, delta),
            FocusAction::Cancelled => InputAction::NavigationCancelled,
        }
    }
}

impl Default for UnifiedInputManager {
    fn default() -> Self {
        Self::new()
    }
}

// Import Key for use in handle_event
use winit::keyboard::Key;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::focus_manager::{FocusableElement, ElementType};

    fn create_test_element(id: &str, y: f32) -> FocusableElement {
        FocusableElement {
            id: id.to_string(),
            label: id.to_string(),
            element_type: ElementType::Button,
            position: (100.0, y),
            size: (200.0, 50.0),
            focusable: true,
        }
    }

    #[test]
    fn test_unified_input_manager_creation() {
        let input = UnifiedInputManager::new();
        assert_eq!(input.get_priority(), InputPriority::None);
    }

    #[test]
    fn test_default_unified_input_manager() {
        let input = UnifiedInputManager::default();
        assert_eq!(input.get_priority(), InputPriority::None);
    }

    #[test]
    fn test_get_focus_manager() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));
        assert_eq!(input.focus().elements().len(), 1);
    }

    #[test]
    fn test_get_keyboard_handler() {
        let input = UnifiedInputManager::new();
        assert!(!input.keyboard().any_key_pressed());
    }

    #[test]
    fn test_get_mouse_handler() {
        let input = UnifiedInputManager::new();
        assert_eq!(input.mouse().cursor_pos(), (0.0, 0.0));
    }

    #[test]
    fn test_update_updates_all_components() {
        let mut input = UnifiedInputManager::new();
        input.update(0.016); // 60 FPS frame time
        // Should not panic
    }

    #[test]
    fn test_register_element_through_focus() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));
        input.focus().register_element(create_test_element("btn2", 200.0));

        assert_eq!(input.focus().elements().len(), 2);
    }

    #[test]
    fn test_set_focus_through_manager() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));
        input.focus().register_element(create_test_element("btn2", 200.0));

        assert!(input.focus().set_focus("btn2"));
        assert!(input.focus().is_focused("btn2"));
    }

    #[test]
    fn test_keyboard_navigation_through_manager() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));
        input.focus().register_element(create_test_element("btn2", 200.0));

        input.focus().set_focus("btn1");
        let action = input.focus().handle_keyboard_input(KeyboardInput::Next);

        assert!(matches!(action, FocusAction::FocusChanged(id) if id == "btn2"));
        assert!(input.focus().is_focused("btn2"));
    }

    #[test]
    fn test_mouse_hover_through_manager() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));

        // Simulate mouse over element
        input.focus().handle_mouse_move(150.0, 125.0);
        assert!(input.focus().is_hovered("btn1"));
    }

    #[test]
    fn test_clear_through_manager() {
        let mut input = UnifiedInputManager::new();
        input.focus().register_element(create_test_element("btn1", 100.0));
        input.focus().set_focus("btn1");

        input.focus().clear();

        assert!(input.focus().elements().is_empty());
        assert!(input.focus().focused_element().is_none());
    }
}
