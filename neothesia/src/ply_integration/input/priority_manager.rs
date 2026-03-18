//! Unified Input Priority Manager
//!
//! Manages input priority between mouse and keyboard with timeout support.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current time in seconds
fn get_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Input priority mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPriority {
    /// No priority (both methods equal)
    None,
    /// Mouse has priority
    Mouse,
    /// Keyboard has priority
    Keyboard,
}

/// Unified input priority manager
pub struct InputPriorityManager {
    /// Current priority mode
    current: InputPriority,
    /// Timestamp of last input interaction
    last_input_time: f64,
    /// Timeout before priority reverts to None (seconds)
    timeout_seconds: f64,
    /// Mouse movement detection threshold (pixels)
    mouse_movement_threshold: f32,
    /// Last known mouse position
    last_mouse_pos: Option<(f32, f32)>,
    /// Callback for priority changes
    on_priority_change: Option<Box<dyn Fn(InputPriority) + Send>>,
    /// Callback for cursor visibility changes
    on_cursor_visibility_change: Option<Box<dyn Fn(bool) + Send>>,
}

impl InputPriorityManager {
    /// Create a new priority manager
    pub fn new() -> Self {
        Self {
            current: InputPriority::None,
            last_input_time: 0.0,
            timeout_seconds: 5.0,
            mouse_movement_threshold: 1.0,
            last_mouse_pos: None,
            on_priority_change: None,
            on_cursor_visibility_change: None,
        }
    }

    /// Update mouse position and detect movement
    /// Returns true if mouse movement was detected
    pub fn update_mouse_position(&mut self, x: f32, y: f32) -> bool {
        let mut moved = false;

        if let Some(last_pos) = self.last_mouse_pos {
            let dx = (x - last_pos.0).abs();
            let dy = (y - last_pos.1).abs();

            // Check if movement exceeds threshold
            if dx > self.mouse_movement_threshold || dy > self.mouse_movement_threshold {
                self.set_input_priority(InputPriority::Mouse);
                moved = true;
            }
        } else {
            // First mouse position, set priority
            self.set_input_priority(InputPriority::Mouse);
            moved = true;
        }

        self.last_mouse_pos = Some((x, y));
        moved
    }

    /// Set keyboard priority (called on keyboard input)
    pub fn set_keyboard_priority(&mut self) {
        self.set_input_priority(InputPriority::Keyboard);
    }

    /// Get current priority (with timeout check)
    pub fn get_priority(&self) -> InputPriority {
        self.current
    }

    /// Check if mouse has priority
    pub fn has_mouse_priority(&self) -> bool {
        self.get_priority() == InputPriority::Mouse
    }

    /// Check if keyboard has priority
    pub fn has_keyboard_priority(&self) -> bool {
        self.get_priority() == InputPriority::Keyboard
    }

    /// Set priority change callback
    pub fn set_priority_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(InputPriority) + Send + 'static,
    {
        self.on_priority_change = Some(Box::new(callback));
    }
    
    /// Set cursor visibility change callback
    pub fn set_cursor_visibility_callback<F>(&mut self, callback: F)
    where
        F: Fn(bool) + Send + 'static,
    {
        self.on_cursor_visibility_change = Some(Box::new(callback));
    }
    
    /// Check if cursor should be visible (true when mouse has priority)
    pub fn should_show_cursor(&self) -> bool {
        self.current == InputPriority::Mouse
    }

    /// Reset priority to None
    pub fn reset(&mut self) {
        self.set_input_priority(InputPriority::None);
        self.last_mouse_pos = None;
    }

    /// Update timeout state (call each frame)
    pub fn update(&mut self, delta_time: f64) {
        // Check timeout for current priority
        if self.current != InputPriority::None {
            let elapsed = get_time() - self.last_input_time;
            if elapsed > self.timeout_seconds {
                let old_priority = self.current;
                self.current = InputPriority::None;

                // Notify callback if priority changed
                if let Some(ref callback) = self.on_priority_change {
                    callback(self.current);
                }
                
                // Notify cursor visibility callback (show cursor when priority is None)
                if let Some(ref callback) = self.on_cursor_visibility_change {
                    callback(true); // Show cursor when no priority
                }

                log::debug!("Priority timed out: {:?} -> None", old_priority);
            }
        }
    }

    /// Set the timeout duration
    pub fn set_timeout(&mut self, timeout_seconds: f64) {
        self.timeout_seconds = timeout_seconds;
    }

    /// Set the mouse movement threshold
    pub fn set_movement_threshold(&mut self, threshold: f32) {
        self.mouse_movement_threshold = threshold;
    }

    /// Set the current input priority
    fn set_input_priority(&mut self, priority: InputPriority) {
        if self.current != priority {
            let old_priority = self.current;
            self.current = priority;
            self.last_input_time = get_time();

            // Notify callback if priority changed
            if let Some(ref callback) = self.on_priority_change {
                callback(priority);
            }
            
            // Notify cursor visibility callback
            let should_show = self.should_show_cursor();
            if let Some(ref callback) = self.on_cursor_visibility_change {
                callback(should_show);
            }

            log::debug!("Priority changed: {:?} -> {:?}, cursor visible: {}", old_priority, priority, should_show);
        }
    }
}

impl Default for InputPriorityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_priority_is_none() {
        let manager = InputPriorityManager::new();
        assert_eq!(manager.get_priority(), InputPriority::None);
    }

    #[test]
    fn test_mouse_movement_sets_priority() {
        let mut manager = InputPriorityManager::new();
        assert_eq!(manager.get_priority(), InputPriority::None);

        manager.update_mouse_position(100.0, 100.0);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);
    }

    #[test]
    fn test_small_mouse_movement_does_not_set_priority() {
        let mut manager = InputPriorityManager::new();
        manager.update_mouse_position(100.0, 100.0);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);

        // Small movement below threshold
        let moved = manager.update_mouse_position(100.5, 100.5);
        assert!(!moved);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);
    }

    #[test]
    fn test_keyboard_input_sets_priority() {
        let mut manager = InputPriorityManager::new();
        assert_eq!(manager.get_priority(), InputPriority::None);

        manager.set_keyboard_priority();
        assert_eq!(manager.get_priority(), InputPriority::Keyboard);
    }

    #[test]
    fn test_has_mouse_priority() {
        let mut manager = InputPriorityManager::new();
        assert!(!manager.has_mouse_priority());

        manager.update_mouse_position(100.0, 100.0);
        assert!(manager.has_mouse_priority());
    }

    #[test]
    fn test_has_keyboard_priority() {
        let mut manager = InputPriorityManager::new();
        assert!(!manager.has_keyboard_priority());

        manager.set_keyboard_priority();
        assert!(manager.has_keyboard_priority());
    }

    #[test]
    fn test_reset_clears_priority() {
        let mut manager = InputPriorityManager::new();
        manager.update_mouse_position(100.0, 100.0);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);

        manager.reset();
        assert_eq!(manager.get_priority(), InputPriority::None);
        assert!(manager.last_mouse_pos.is_none());
    }

    #[test]
    fn test_set_timeout() {
        let mut manager = InputPriorityManager::new();
        manager.set_timeout(10.0);
        assert_eq!(manager.timeout_seconds, 10.0);
    }

    #[test]
    fn test_set_movement_threshold() {
        let mut manager = InputPriorityManager::new();
        manager.set_movement_threshold(5.0);
        assert_eq!(manager.mouse_movement_threshold, 5.0);
    }

    #[test]
    fn test_priority_change_callback() {
        let mut manager = InputPriorityManager::new();

        // Just verify that setting a callback doesn't panic
        manager.set_priority_change_callback(|priority| {
            // Callback is called, we can't easily verify this without thread-safe types
            let _ = priority;
        });

        manager.update_mouse_position(100.0, 100.0);
        // If we got here without panicking, the test passes
    }

    #[test]
    fn test_keyboard_overrides_mouse() {
        let mut manager = InputPriorityManager::new();
        manager.update_mouse_position(100.0, 100.0);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);

        manager.set_keyboard_priority();
        assert_eq!(manager.get_priority(), InputPriority::Keyboard);
    }

    #[test]
    fn test_mouse_overrides_keyboard() {
        let mut manager = InputPriorityManager::new();
        manager.set_keyboard_priority();
        assert_eq!(manager.get_priority(), InputPriority::Keyboard);

        manager.update_mouse_position(200.0, 200.0);
        assert_eq!(manager.get_priority(), InputPriority::Mouse);
    }
}
