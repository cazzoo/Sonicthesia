//! Cursor Visibility Management
//!
//! Provides helper functions for managing mouse cursor visibility based on input priority.
//! This module provides a callback-based approach where the application receives
//! cursor visibility change notifications and handles the actual cursor visibility
//! through winit's window API.

/// Cursor visibility callback type
pub type CursorVisibilityCallback = Box<dyn Fn(bool) + Send>;

/// Current cursor visibility state
static mut CURSOR_VISIBLE: bool = true;
static mut CURSOR_CALLBACK: Option<CursorVisibilityCallback> = None;

/// Initialize cursor visibility system with a callback
pub fn init_cursor_with_callback(callback: CursorVisibilityCallback) {
    unsafe {
        CURSOR_VISIBLE = true;
        CURSOR_CALLBACK = Some(callback);
        // Call the callback to set initial visibility
        if let Some(ref cb) = CURSOR_CALLBACK {
            cb(true);
        }
    }
}

/// Set cursor visibility (this will call the registered callback)
pub fn set_cursor_visibility(visible: bool) {
    unsafe {
        if CURSOR_VISIBLE != visible {
            CURSOR_VISIBLE = visible;
            // Call the callback to actually change cursor visibility
            if let Some(ref cb) = CURSOR_CALLBACK {
                cb(visible);
            }
        }
    }
}

/// Get current cursor visibility state
pub fn is_cursor_visible() -> bool {
    unsafe { CURSOR_VISIBLE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_initialization() {
        // This test just verifies the functions exist
        // Actual cursor manipulation can't be tested in unit tests
        let _ = is_cursor_visible();
    }
}
