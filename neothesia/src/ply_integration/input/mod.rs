//! PLY Input Integration Module
//!
//! This module provides comprehensive input handling for the PLY engine integration,
//! bridging PLY input events with Neothesia's action system.

use std::collections::HashMap;

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoopProxy,
    keyboard::{Key, NamedKey},
};

use crate::NeothesiaEvent;

mod keyboard;
mod mouse;
mod gamepad;
mod keyboard_to_midi;
mod priority_manager;
mod focus_manager;
mod unified_input;
mod cursor;

pub use keyboard::PlyKeyboardHandler;
pub use mouse::PlyMouseHandler;
pub use gamepad::PlyGamepadHandler;
pub use keyboard_to_midi::KeyboardToMidiConverter;
pub use priority_manager::{InputPriority, InputPriorityManager};
pub use focus_manager::{FocusManager, FocusableElement, ElementType, KeyboardInput, FocusAction};
pub use unified_input::{UnifiedInputManager, InputAction};
pub use cursor::{init_cursor_with_callback, set_cursor_visibility, is_cursor_visible};

/// Neothesia input actions that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeothesiaAction {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Confirm,
    Cancel,
    Back,

    // Playback control
    PlayPause,
    Stop,
    Restart,
    FastForward,
    Rewind,

    // Settings
    OpenSettings,
    ToggleFullscreen,

    // Song selection
    NextSong,
    PreviousSong,

    // View control
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,

    // Practice mode
    ToggleWaitMode,
    ToggleLoopMode,

    // Recording
    StartRecording,
    StopRecording,

    // Misc
    ShowHelp,
    Quit,
}

/// Input binding configuration
#[derive(Debug, Clone)]
pub struct InputBinding {
    /// Keyboard key that triggers this action
    pub key: Option<Key>,
    /// Gamepad button that triggers this action
    pub gamepad_button: Option<GamepadButton>,
    /// Mouse button that triggers this action
    pub mouse_button: Option<MouseButton>,
}

/// Gamepad button enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LeftTrigger,
    RightTrigger,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Select,
    LeftStick,
    RightStick,
}

/// Main PLY input handler
pub struct PlyInputHandler {
    proxy: EventLoopProxy<NeothesiaEvent>,
    keyboard: PlyKeyboardHandler,
    mouse: PlyMouseHandler,
    gamepad: PlyGamepadHandler,
    keyboard_to_midi: KeyboardToMidiConverter,
    bindings: HashMap<NeothesiaAction, InputBinding>,
    reverse_bindings: HashMap<String, NeothesiaAction>,
}

impl PlyInputHandler {
    /// Create a new PLY input handler
    pub fn new(proxy: EventLoopProxy<NeothesiaEvent>) -> Self {
        log::info!("🎯 PLY Input Handler initialized - Using PLY input system");
        let bindings = Self::create_default_bindings();
        let reverse_bindings = Self::create_reverse_bindings(&bindings);

        Self {
            proxy,
            keyboard: PlyKeyboardHandler::new(),
            mouse: PlyMouseHandler::new(),
            gamepad: PlyGamepadHandler::new(),
            keyboard_to_midi: KeyboardToMidiConverter::new(),
            bindings,
            reverse_bindings,
        }
    }

    /// Create default input bindings
    fn create_default_bindings() -> HashMap<NeothesiaAction, InputBinding> {
        let mut bindings = HashMap::new();

        // Navigation
        bindings.insert(NeothesiaAction::NavigateUp, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowUp)),
            gamepad_button: Some(GamepadButton::DPadUp),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::NavigateDown, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowDown)),
            gamepad_button: Some(GamepadButton::DPadDown),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::NavigateLeft, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowLeft)),
            gamepad_button: Some(GamepadButton::DPadLeft),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::NavigateRight, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowRight)),
            gamepad_button: Some(GamepadButton::DPadRight),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Confirm, InputBinding {
            key: Some(Key::Named(NamedKey::Enter)),
            gamepad_button: Some(GamepadButton::A),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Cancel, InputBinding {
            key: Some(Key::Named(NamedKey::Escape)),
            gamepad_button: Some(GamepadButton::B),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Back, InputBinding {
            key: Some(Key::Named(NamedKey::Escape)),
            gamepad_button: Some(GamepadButton::B),
            mouse_button: Some(MouseButton::Back),
        });

        // Playback control
        bindings.insert(NeothesiaAction::PlayPause, InputBinding {
            key: Some(Key::Character(" ".into())),
            gamepad_button: Some(GamepadButton::Start),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Stop, InputBinding {
            key: None,
            gamepad_button: Some(GamepadButton::Select),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Restart, InputBinding {
            key: Some(Key::Character("r".into())),
            gamepad_button: None,
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::FastForward, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowRight)),
            gamepad_button: None,
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Rewind, InputBinding {
            key: Some(Key::Named(NamedKey::ArrowLeft)),
            gamepad_button: None,
            mouse_button: None,
        });

        // Settings
        bindings.insert(NeothesiaAction::OpenSettings, InputBinding {
            key: Some(Key::Character("s".into())),
            gamepad_button: Some(GamepadButton::X),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::ToggleFullscreen, InputBinding {
            key: Some(Key::Named(NamedKey::F11)),
            gamepad_button: None,
            mouse_button: None,
        });

        // Song selection
        bindings.insert(NeothesiaAction::NextSong, InputBinding {
            key: Some(Key::Named(NamedKey::PageDown)),
            gamepad_button: Some(GamepadButton::RightShoulder),
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::PreviousSong, InputBinding {
            key: Some(Key::Named(NamedKey::PageUp)),
            gamepad_button: Some(GamepadButton::LeftShoulder),
            mouse_button: None,
        });

        // View control
        bindings.insert(NeothesiaAction::ZoomIn, InputBinding {
            key: Some(Key::Character("+".into())),
            gamepad_button: None,
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::ZoomOut, InputBinding {
            key: Some(Key::Character("-".into())),
            gamepad_button: None,
            mouse_button: None,
        });

        // Practice mode
        bindings.insert(NeothesiaAction::ToggleWaitMode, InputBinding {
            key: Some(Key::Character("w".into())),
            gamepad_button: Some(GamepadButton::Y),
            mouse_button: None,
        });

        // Misc
        bindings.insert(NeothesiaAction::ShowHelp, InputBinding {
            key: Some(Key::Named(NamedKey::F1)),
            gamepad_button: None,
            mouse_button: None,
        });
        bindings.insert(NeothesiaAction::Quit, InputBinding {
            key: Some(Key::Character("q".into())),
            gamepad_button: None,
            mouse_button: None,
        });

        bindings
    }

    /// Create reverse lookup map for bindings
    fn create_reverse_bindings(
        bindings: &HashMap<NeothesiaAction, InputBinding>,
    ) -> HashMap<String, NeothesiaAction> {
        let mut reverse = HashMap::new();

        for (action, binding) in bindings {
            if let Some(key) = &binding.key {
                reverse.insert(format!("key:{}", key_to_string(key)), *action);
            }
            if let Some(btn) = binding.gamepad_button {
                reverse.insert(format!("gamepad:{:?}", btn), *action);
            }
            if let Some(btn) = binding.mouse_button {
                reverse.insert(format!("mouse:{:?}", btn), *action);
            }
        }

        reverse
    }

    /// Handle a window event and process input
    pub fn handle_event(&mut self, event: &WindowEvent) {
        // Log PLY input handling (only for significant events to avoid spam)
        match event {
            WindowEvent::KeyboardInput { .. } => {
                log::debug!("🎹 PLY Input: Processing keyboard event");
            }
            WindowEvent::MouseInput { .. } => {
                log::debug!("🖱️  PLY Input: Processing mouse event");
            }
            _ => {}
        }

        // Handle keyboard-to-MIDI conversion for piano keyboard
        self.keyboard_to_midi.handle_keyboard_event(event, &self.proxy);

        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                self.keyboard.handle_key_event(key_event, &self.reverse_bindings, &self.proxy);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse.handle_button_event(*state, *button, &self.reverse_bindings, &self.proxy);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.handle_move_physical(position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse.handle_wheel_event(delta, &self.reverse_bindings, &self.proxy);
            }
            _ => {}
        }
    }

    /// Update input state (call each frame)
    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
        self.gamepad.update(&self.reverse_bindings, &self.proxy);
    }

    /// Get keyboard handler
    pub fn keyboard(&self) -> &PlyKeyboardHandler {
        &self.keyboard
    }

    /// Get mouse handler
    pub fn mouse(&self) -> &PlyMouseHandler {
        &self.mouse
    }

    /// Get gamepad handler
    pub fn gamepad(&self) -> &PlyGamepadHandler {
        &self.gamepad
    }

    /// Get keyboard-to-MIDI converter
    pub fn keyboard_to_midi(&self) -> &KeyboardToMidiConverter {
        &self.keyboard_to_midi
    }

    /// Get mutable keyboard-to-MIDI converter
    pub fn keyboard_to_midi_mut(&mut self) -> &mut KeyboardToMidiConverter {
        &mut self.keyboard_to_midi
    }

    /// Check if an action is currently active
    pub fn is_action_active(&self, action: NeothesiaAction) -> bool {
        if let Some(binding) = self.bindings.get(&action) {
            if let Some(key) = &binding.key {
                if self.keyboard.is_key_pressed(key) {
                    return true;
                }
            }
            if let Some(btn) = binding.gamepad_button {
                if self.gamepad.is_button_pressed(btn) {
                    return true;
                }
            }
            if let Some(btn) = binding.mouse_button {
                if self.mouse.is_button_pressed(btn) {
                    return true;
                }
            }
        }
        false
    }
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
