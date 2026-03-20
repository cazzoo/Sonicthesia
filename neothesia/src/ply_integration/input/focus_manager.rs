//! Unified Focus Manager
//!
//! Manages focus and hover state for UI elements across all scenes.

use super::priority_manager::{InputPriorityManager, InputPriority};

/// Type of focusable element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Button,
    Toggle,
    Spinner,
    Slider,
    Picker,
    Other,
}

/// A focusable element in a scene
pub struct FocusableElement {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Element type
    pub element_type: ElementType,
    /// Current position (for hover detection)
    pub position: (f32, f32),
    /// Current size (for hover detection)
    pub size: (f32, f32),
    /// Whether this element can be focused
    pub focusable: bool,
}

/// Keyboard input for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardInput {
    Next,
    Previous,
    Activate,
    Adjust(i32),
    Cancel,
}

/// Result of keyboard input
#[derive(Debug, Clone, PartialEq)]
pub enum FocusAction {
    None,
    FocusChanged(String),
    Activated(String),
    Adjusted(String, i32),
    Cancelled,
}

/// Unified focus manager for all scenes
pub struct FocusManager {
    /// All focusable elements
    focusable_elements: Vec<FocusableElement>,
    /// Currently focused element index (the ONLY focus indicator)
    focused_index: Option<usize>,
    /// Priority manager reference
    priority: InputPriorityManager,
    /// Mouse hover state (internal only, used to update focused_index when mouse has priority)
    hovered_element: Option<usize>,
    /// Previous mouse position for movement detection
    last_mouse_pos: Option<(f32, f32)>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            focusable_elements: Vec::new(),
            focused_index: None,
            priority: InputPriorityManager::new(),
            hovered_element: None,
            last_mouse_pos: None,
        }
    }

    /// Register a focusable element
    pub fn register_element(&mut self, element: FocusableElement) {
        self.focusable_elements.push(element);
    }

    /// Update element position (call each frame for moving elements)
    pub fn update_element_position(&mut self, id: &str, position: (f32, f32)) {
        if let Some(element) = self.focusable_elements.iter_mut().find(|e| e.id == id) {
            element.position = position;
        }
    }

    /// Handle mouse movement (returns true if focus changed)
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) -> bool {
        let mut new_hover = None;

        // Find element under cursor
        for (i, element) in self.focusable_elements.iter().enumerate() {
            if element.focusable {
                let (ex, ey) = element.position;
                let (ew, eh) = element.size;

                if x >= ex && x <= ex + ew && y >= ey && y <= ey + eh {
                    new_hover = Some(i);
                    break;
                }
            }
        }

        // Update priority manager with mouse position
        let mouse_moved = self.priority.update_mouse_position(x, y);
        
        // Store hover state internally
        let hover_changed = self.hovered_element != new_hover;
        self.hovered_element = new_hover;
        
        // When mouse has priority, hovered element becomes the focused element
        if self.priority.has_mouse_priority() {
            if let Some(hovered) = self.hovered_element {
                let focus_changed = self.focused_index != Some(hovered);
                self.focused_index = Some(hovered);
                return focus_changed;
            } else {
                // No element hovered, clear focus
                let focus_changed = self.focused_index.is_some();
                self.focused_index = None;
                return focus_changed;
            }
        }
        
        hover_changed
    }

    /// Handle keyboard navigation
    pub fn handle_keyboard_input(&mut self, input: KeyboardInput) -> FocusAction {
        // Set keyboard priority on any keyboard input
        self.priority.set_keyboard_priority();

        match input {
            KeyboardInput::Next => {
                self.focus_next();
                if let Some(element) = self.focused_element() {
                    FocusAction::FocusChanged(element.id.clone())
                } else {
                    FocusAction::None
                }
            }
            KeyboardInput::Previous => {
                self.focus_previous();
                if let Some(element) = self.focused_element() {
                    FocusAction::FocusChanged(element.id.clone())
                } else {
                    FocusAction::None
                }
            }
            KeyboardInput::Activate => {
                if let Some(element) = self.focused_element() {
                    FocusAction::Activated(element.id.clone())
                } else {
                    FocusAction::None
                }
            }
            KeyboardInput::Adjust(delta) => {
                if let Some(element) = self.focused_element() {
                    FocusAction::Adjusted(element.id.clone(), delta)
                } else {
                    FocusAction::None
                }
            }
            KeyboardInput::Cancel => FocusAction::Cancelled,
        }
    }

    /// Navigate to next element
    pub fn focus_next(&mut self) {
        if self.focusable_elements.is_empty() {
            return;
        }

        // Filter to only focusable elements
        let focusable_indices: Vec<usize> = self
            .focusable_elements
            .iter()
            .enumerate()
            .filter(|(_, e)| e.focusable)
            .map(|(i, _)| i)
            .collect();

        if focusable_indices.is_empty() {
            return;
        }

        let current = self.focused_index.unwrap_or(0);

        // Find the next focusable index
        if let Some(pos) = focusable_indices.iter().position(|&i| i == current) {
            let next_pos = (pos + 1) % focusable_indices.len();
            self.focused_index = Some(focusable_indices[next_pos]);
        } else {
            // Current focus is not focusable, start from first
            self.focused_index = Some(focusable_indices[0]);
        }

        // Clear hover state when keyboard navigates (prevents visual conflicts)
        self.hovered_element = None;
    }

    /// Navigate to previous element
    pub fn focus_previous(&mut self) {
        if self.focusable_elements.is_empty() {
            return;
        }

        // Filter to only focusable elements
        let focusable_indices: Vec<usize> = self
            .focusable_elements
            .iter()
            .enumerate()
            .filter(|(_, e)| e.focusable)
            .map(|(i, _)| i)
            .collect();

        if focusable_indices.is_empty() {
            return;
        }

        let current = self.focused_index.unwrap_or(0);

        // Find the previous focusable index
        if let Some(pos) = focusable_indices.iter().position(|&i| i == current) {
            let prev_pos = if pos == 0 {
                focusable_indices.len() - 1
            } else {
                pos - 1
            };
            self.focused_index = Some(focusable_indices[prev_pos]);
        } else {
            // Current focus is not focusable, start from last
            self.focused_index = Some(focusable_indices[focusable_indices.len() - 1]);
        }

        // Clear hover state when keyboard navigates (prevents visual conflicts)
        self.hovered_element = None;
    }

    /// Set focus by element ID
    pub fn set_focus(&mut self, id: &str) -> bool {
        if let Some(index) = self.focusable_elements.iter().position(|e| e.id == id) {
            if self.focusable_elements[index].focusable {
                self.focused_index = Some(index);
                // Clear hover state when setting focus programmatically
                self.hovered_element = None;
                return true;
            }
        }
        false
    }

    /// Get currently focused element
    pub fn focused_element(&self) -> Option<&FocusableElement> {
        self.focused_index
            .and_then(|i| self.focusable_elements.get(i))
    }

    /// Get currently hovered element
    pub fn hovered_element(&self) -> Option<&FocusableElement> {
        self.hovered_element
            .and_then(|i| self.focusable_elements.get(i))
    }

    /// Check if an element is focused
    pub fn is_focused(&self, id: &str) -> bool {
        self.focused_element()
            .map(|e| e.id == id)
            .unwrap_or(false)
    }

    /// Check if an element is hovered
    pub fn is_hovered(&self, id: &str) -> bool {
        self.hovered_element()
            .map(|e| e.id == id)
            .unwrap_or(false)
    }

    /// Clear all elements (call when scene changes)
    pub fn clear(&mut self) {
        self.focusable_elements.clear();
        self.focused_index = None;
        self.hovered_element = None;
        self.last_mouse_pos = None;
        self.priority.reset();
    }

    /// Update state (call each frame)
    pub fn update(&mut self, delta_time: f64) {
        self.priority.update(delta_time);
    }

    /// Get priority manager reference
    pub fn priority(&mut self) -> &mut InputPriorityManager {
        &mut self.priority
    }

    /// Get all elements
    pub fn elements(&self) -> &[FocusableElement] {
        &self.focusable_elements
    }

    /// Get focused index
    pub fn focused_index(&self) -> Option<usize> {
        self.focused_index
    }

    /// Get hovered index (internal use only - for debugging)
    pub fn hovered_index(&self) -> Option<usize> {
        self.hovered_element
    }

    /// Get current input priority (internal method for UnifiedInputManager)
    pub fn get_priority_internal(&self) -> InputPriority {
        self.priority.get_priority()
    }
    
    /// Check if mouse should be visible (true when mouse has priority)
    pub fn should_show_cursor(&self) -> bool {
        self.priority.has_mouse_priority()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_empty_focus_manager() {
        let focus = FocusManager::new();
        assert!(focus.focused_element().is_none());
        assert!(focus.hovered_element().is_none());
        assert!(!focus.is_focused("test"));
        assert!(!focus.is_hovered("test"));
    }

    #[test]
    fn test_register_element() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        assert_eq!(focus.elements().len(), 1);
    }

    #[test]
    fn test_set_focus() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));

        assert!(focus.set_focus("btn2"));
        assert!(focus.is_focused("btn2"));
        assert!(!focus.is_focused("btn1"));
    }

    #[test]
    fn test_set_focus_non_focusable() {
        let mut focus = FocusManager::new();
        let mut element = create_test_element("btn1", 100.0);
        element.focusable = false;
        focus.register_element(element);

        assert!(!focus.set_focus("btn1"));
        assert!(!focus.is_focused("btn1"));
    }

    #[test]
    fn test_focus_next() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));
        focus.register_element(create_test_element("btn3", 300.0));

        focus.set_focus("btn1");
        focus.focus_next();
        assert!(focus.is_focused("btn2"));

        focus.focus_next();
        assert!(focus.is_focused("btn3"));

        // Wrap around
        focus.focus_next();
        assert!(focus.is_focused("btn1"));
    }

    #[test]
    fn test_focus_previous() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));
        focus.register_element(create_test_element("btn3", 300.0));

        focus.set_focus("btn3");
        focus.focus_previous();
        assert!(focus.is_focused("btn2"));

        focus.focus_previous();
        assert!(focus.is_focused("btn1"));

        // Wrap around
        focus.focus_previous();
        assert!(focus.is_focused("btn3"));
    }

    #[test]
    fn test_handle_mouse_move() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));

        // Mouse over btn1 (100-300, 100-150)
        assert!(focus.handle_mouse_move(150.0, 125.0));
        assert!(focus.is_hovered("btn1"));

        // Mouse over btn2 (100-300, 200-250)
        assert!(focus.handle_mouse_move(150.0, 225.0));
        assert!(focus.is_hovered("btn2"));
        assert!(!focus.is_hovered("btn1"));
    }

    #[test]
    fn test_handle_keyboard_input_next() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));

        focus.set_focus("btn1");
        let action = focus.handle_keyboard_input(KeyboardInput::Next);

        assert!(matches!(action, FocusAction::FocusChanged(id) if id == "btn2"));
        assert!(focus.is_focused("btn2"));
    }

    #[test]
    fn test_handle_keyboard_input_previous() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));

        focus.set_focus("btn2");
        let action = focus.handle_keyboard_input(KeyboardInput::Previous);

        assert!(matches!(action, FocusAction::FocusChanged(id) if id == "btn1"));
        assert!(focus.is_focused("btn1"));
    }

    #[test]
    fn test_handle_keyboard_input_activate() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));

        focus.set_focus("btn1");
        let action = focus.handle_keyboard_input(KeyboardInput::Activate);

        assert!(matches!(action, FocusAction::Activated(id) if id == "btn1"));
    }

    #[test]
    fn test_handle_keyboard_input_adjust() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));

        focus.set_focus("btn1");
        let action = focus.handle_keyboard_input(KeyboardInput::Adjust(5));

        assert!(matches!(action, FocusAction::Adjusted(id, 5) if id == "btn1"));
    }

    #[test]
    fn test_handle_keyboard_input_cancel() {
        let mut focus = FocusManager::new();
        let action = focus.handle_keyboard_input(KeyboardInput::Cancel);

        assert!(matches!(action, FocusAction::Cancelled));
    }

    #[test]
    fn test_keyboard_clears_hover() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.register_element(create_test_element("btn2", 200.0));

        // Set hover on btn1
        focus.handle_mouse_move(150.0, 125.0);
        assert!(focus.is_hovered("btn1"));

        // Navigate with keyboard
        focus.handle_keyboard_input(KeyboardInput::Next);

        // Hover should be cleared
        assert!(!focus.is_hovered("btn1"));
        assert!(focus.hovered_index().is_none());
    }

    #[test]
    fn test_update_element_position() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));

        focus.update_element_position("btn1", (200.0, 300.0));
        assert_eq!(focus.elements()[0].position, (200.0, 300.0));
    }

    #[test]
    fn test_clear() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));
        focus.set_focus("btn1");
        focus.handle_mouse_move(150.0, 125.0);

        focus.clear();

        assert!(focus.elements().is_empty());
        assert!(focus.focused_element().is_none());
        assert!(focus.hovered_element().is_none());
    }

    #[test]
    fn test_only_focusable_elements_can_be_focused() {
        let mut focus = FocusManager::new();
        focus.register_element(create_test_element("btn1", 100.0));

        let mut non_focusable = create_test_element("btn2", 200.0);
        non_focusable.focusable = false;
        focus.register_element(non_focusable);

        // Should only cycle through focusable elements
        focus.set_focus("btn1");
        focus.focus_next();
        assert!(focus.is_focused("btn1")); // Should wrap back to btn1
    }
}
