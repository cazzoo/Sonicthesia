# Unified Input System - Complete Guide

## Overview

The unified input system provides a single, consistent focus indicator across the entire Neothesia application. It ensures that only ONE element is focused at any time, with automatic priority switching between mouse and keyboard input.

## Key Principles

### Single Focus Indicator
- **Only ONE element is focused at any time**
- The focus indicator shows which element currently has focus
- Mouse hover does NOT create a separate focus indicator
- When mouse has priority, the hovered element becomes the focused element
- When keyboard has priority, the keyboard-navigated element is the focused element

### Input Priority Behavior
- **Mouse has priority**: When mouse is moving, the hovered element becomes focused
- **Keyboard has priority**: When any keyboard navigation key is pressed, keyboard gets priority
- **Priority timeout**: After 5 seconds of no input, priority reverts to None
- **Mouse stops moving**: When mouse stops, keyboard gets priority back on next keypress
- **Mouse hover when keyboard has priority**: Mouse hover does NOT change focus

### Mouse Cursor Visibility
- **Hide mouse cursor** when keyboard has priority
- **Show mouse cursor** when mouse has priority
- This makes it clear which input mode is active

## Architecture

### Core Components

#### 1. FocusManager ([`neothesia/src/ply_integration/input/focus_manager.rs`](neothesia/src/ply_integration/input/focus_manager.rs:1))
Manages focus and hover state for UI elements across all scenes.

**Key Features:**
- Tracks all focusable elements
- Maintains a single `focused_index` (the ONLY focus indicator)
- Handles mouse movement and keyboard navigation
- Automatically updates focus based on input priority

**Important Methods:**
- `register_element()` - Register a focusable element
- `handle_mouse_move()` - Handle mouse movement (updates focus when mouse has priority)
- `handle_keyboard_input()` - Handle keyboard navigation (sets keyboard priority)
- `focused_element()` - Get the currently focused element
- `should_show_cursor()` - Check if mouse cursor should be visible

#### 2. InputPriorityManager ([`neothesia/src/ply_integration/input/priority_manager.rs`](neothesia/src/ply_integration/input/priority_manager.rs:1))
Manages input priority between mouse and keyboard with timeout support.

**Priority States:**
- `None` - No priority (both methods equal)
- `Mouse` - Mouse has priority
- `Keyboard` - Keyboard has priority

**Key Features:**
- Automatic priority switching based on input
- 5-second timeout before reverting to None
- Callback support for priority changes
- Callback support for cursor visibility changes

**Important Methods:**
- `update_mouse_position()` - Update mouse position and detect movement
- `set_keyboard_priority()` - Set keyboard priority (called on keyboard input)
- `has_mouse_priority()` - Check if mouse has priority
- `has_keyboard_priority()` - Check if keyboard has priority
- `should_show_cursor()` - Check if cursor should be visible
- `set_cursor_visibility_callback()` - Set callback for cursor visibility changes

#### 3. UnifiedInputManager ([`neothesia/src/ply_integration/input/unified_input.rs`](neothesia/src/ply_integration/input/unified_input.rs:1))
Combines focus, keyboard, and mouse handlers into a unified interface.

**Key Features:**
- Single entry point for all input handling
- Automatic priority management
- Consistent behavior across all scenes

**Important Methods:**
- `handle_event()` - Handle winit window events
- `update()` - Update state (call each frame)
- `focus()` - Get focus manager reference
- `get_priority()` - Get current input priority

#### 4. PlyUi ([`neothesia/src/ply_integration/ui/mod.rs`](neothesia/src/ply_integration/ui/mod.rs:1))
PLY-based UI framework with integrated unified input support.

**Key Features:**
- Built-in focus management using unified input system
- Automatic priority-based focus updates
- Cursor visibility callbacks

**Important Methods:**
- `mouse_move()` - Handle mouse movement
- `handle_key_event()` - Handle keyboard events
- `should_show_cursor()` - Check if cursor should be visible
- `set_cursor_visibility_callback()` - Set callback for cursor visibility changes

#### 5. Cursor Visibility Helper ([`neothesia/src/ply_integration/input/cursor.rs`](neothesia/src/ply_integration/input/cursor.rs:1))
Provides helper functions for managing mouse cursor visibility.

**Important Methods:**
- `init_cursor_with_callback()` - Initialize with a callback for actual cursor control
- `set_cursor_visibility()` - Set cursor visibility (calls the callback)
- `is_cursor_visible()` - Get current cursor visibility state

## Usage Guide

### For Scene Implementations

#### Step 1: Initialize the Unified Input System

In your scene's initialization:

```rust
use crate::ply_integration::input::{UnifiedInputManager, init_cursor_with_callback};

pub struct MyScene {
    input_manager: UnifiedInputManager,
}

impl MyScene {
    pub fn new() -> Self {
        // Initialize cursor visibility with callback
        init_cursor_with_callback(Box::new(|visible| {
            // This callback will be called when cursor visibility should change
            // Use winit's window API to actually show/hide the cursor
            // For example: window.set_cursor_visible(visible);
        }));

        Self {
            input_manager: UnifiedInputManager::new(),
        }
    }
}
```

#### Step 2: Register Focusable Elements

When building your UI, register all focusable elements:

```rust
fn build_ui(&mut self) {
    // Register buttons
    self.input_manager.focus().register_element(FocusableElement {
        id: "button1".to_string(),
        label: "Button 1".to_string(),
        element_type: ElementType::Button,
        position: (100.0, 100.0),
        size: (200.0, 50.0),
        focusable: true,
    });

    // Register more elements...
}
```

#### Step 3: Handle Input Events

In your scene's event handling:

```rust
fn handle_event(&mut self, event: &WindowEvent) -> InputAction {
    self.input_manager.handle_event(event)
}
```

#### Step 4: Update State Each Frame

In your scene's update loop:

```rust
fn update(&mut self, delta_time: f64) {
    self.input_manager.update(delta_time);
    
    // Update element positions if they move
    self.input_manager.focus().update_element_position("button1", (100.0, 100.0));
}
```

#### Step 5: Render Focus Indicator

When rendering your UI, check if an element is focused:

```rust
fn render(&self) {
    // Draw focus indicator for focused element
    if let Some(focused) = self.input_manager.focus().focused_element() {
        // Draw focus indicator around the focused element
        draw_focus_indicator(focused.position, focused.size);
    }
}
```

### For PlyUi Usage

If you're using PlyUi for your UI, the unified input system is already integrated:

```rust
use crate::ply_integration::ui::PlyUi;

pub struct MyScene {
    ui: PlyUi,
}

impl MyScene {
    pub fn new() -> Self {
        let mut ui = PlyUi::new();
        
        // Set cursor visibility callback
        ui.set_cursor_visibility_callback(Box::new(|visible| {
            // Use winit's window API to show/hide cursor
            // window.set_cursor_visible(visible);
        }));
        
        Self { ui }
    }

    fn handle_event(&mut self, event: &WindowEvent) {
        // Handle mouse movement
        if let WindowEvent::CursorMoved { position, .. } = event {
            let (x, y) = position.to_logical::<f32>(scale_factor).into();
            self.ui.mouse_move(x, y);
        }

        // Handle keyboard events
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.state == ElementState::Pressed {
                let action = self.ui.handle_key_event(&event.logical_key);
                // Handle action...
            }
        }
    }

    fn update(&mut self, delta_time: f64) {
        self.ui.update(delta_time);
    }
}
```

## Testing Checklist

After implementing the unified input system, test the following:

### Basic Functionality
- [ ] Start application - should have no focus initially
- [ ] Move mouse - hovered element should become focused, cursor visible
- [ ] Press keyboard key - keyboard gets priority, focus moves to first element, cursor hidden
- [ ] Navigate with arrow keys - focus moves, cursor stays hidden
- [ ] Move mouse - mouse gets priority, hovered element becomes focused, cursor visible
- [ ] Stop moving mouse for 5 seconds - priority reverts to None, cursor becomes visible
- [ ] Press keyboard key - keyboard gets priority again, cursor hidden

### Scene-Specific Testing
- [ ] Main menu - single focus indicator works correctly
- [ ] Settings scene - single focus indicator works correctly
- [ ] Song library - single focus indicator works correctly
- [ ] Free play scene - single focus indicator works correctly
- [ ] Playing scene - single focus indicator works correctly

### Edge Cases
- [ ] Mouse hover when keyboard has priority - focus does NOT change
- [ ] Keyboard navigation when mouse has priority - keyboard gets priority, focus changes
- [ ] Rapid switching between mouse and keyboard - priority switches correctly
- [ ] Long periods of inactivity - priority times out correctly

## Migration Guide

### From Separate Hover/Focus Systems

If your scene currently has separate hover and focus tracking:

**Before (Wrong):**
```rust
struct MyScene {
    hovered_element: Option<String>,
    focused_element: Option<String>,
}

// Mouse hover sets hovered_element
// Keyboard navigation sets focused_element
// Both can be set at the same time - WRONG!
```

**After (Correct):**
```rust
struct MyScene {
    input_manager: UnifiedInputManager,
}

// Both mouse and keyboard update the same focused_element
// Only ONE element is focused at any time - CORRECT!
```

### From PlyUi Without Unified Input

If you're using PlyUi but not the unified input system:

**Before:**
```rust
let mut ui = PlyUi::new();
// Separate hovered and focused states
```

**After:**
```rust
let mut ui = PlyUi::new();
ui.set_cursor_visibility_callback(Box::new(|visible| {
    // Handle cursor visibility
}));
// Single focused state, automatic priority management
```

## Common Pitfalls

### 1. Creating Separate Focus Indicators
**Wrong:** Drawing separate green and purple focus indicators
**Right:** Draw only ONE focus indicator that changes color based on priority

### 2. Not Updating Element Positions
**Wrong:** Registering elements once and never updating their positions
**Right:** Call `update_element_position()` each frame for moving elements

### 3. Ignoring Priority in Hover Detection
**Wrong:** Always updating focus on mouse hover
**Right:** Only update focus on mouse hover when mouse has priority

### 4. Forgetting to Handle Cursor Visibility
**Wrong:** Never hiding/showing the cursor based on input priority
**Right:** Set up cursor visibility callback and handle it properly

### 5. Not Clearing Hover on Keyboard Navigation
**Wrong:** Leaving hover state when keyboard navigates
**Right:** Clear hover state when keyboard navigation occurs

## API Reference

### FocusManager

```rust
// Create a new focus manager
let mut focus = FocusManager::new();

// Register a focusable element
focus.register_element(FocusableElement {
    id: "button1".to_string(),
    label: "Button 1".to_string(),
    element_type: ElementType::Button,
    position: (100.0, 100.0),
    size: (200.0, 50.0),
    focusable: true,
});

// Handle mouse movement (returns true if focus changed)
let focus_changed = focus.handle_mouse_move(x, y);

// Handle keyboard input
let action = focus.handle_keyboard_input(KeyboardInput::Next);

// Get currently focused element
if let Some(focused) = focus.focused_element() {
    println!("Focused: {}", focused.id);
}

// Check if cursor should be visible
if focus.should_show_cursor() {
    // Show cursor
} else {
    // Hide cursor
}
```

### InputPriorityManager

```rust
// Create a new priority manager
let mut priority = InputPriorityManager::new();

// Update mouse position
let moved = priority.update_mouse_position(x, y);

// Set keyboard priority
priority.set_keyboard_priority();

// Check current priority
match priority.get_priority() {
    InputPriority::None => println!("No priority"),
    InputPriority::Mouse => println!("Mouse has priority"),
    InputPriority::Keyboard => println!("Keyboard has priority"),
}

// Set cursor visibility callback
priority.set_cursor_visibility_callback(Box::new(|visible| {
    println!("Cursor visible: {}", visible);
}));
```

### UnifiedInputManager

```rust
// Create a new unified input manager
let mut input = UnifiedInputManager::new();

// Handle window event
let action = input.handle_event(&window_event);

// Update state
input.update(delta_time);

// Get focus manager
let focus = input.focus();

// Get current priority
let priority = input.get_priority();
```

## Conclusion

The unified input system provides a consistent, single-focus experience across the entire Neothesia application. By following this guide and ensuring only ONE focus indicator exists at any time, you'll create a better user experience with clear visual feedback about which element is currently focused.

Remember: **Only ONE focus indicator at any time!**
