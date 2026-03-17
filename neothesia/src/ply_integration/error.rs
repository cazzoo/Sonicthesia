//! PLY Integration Error Handling
//!
//! This module provides comprehensive error types and handling for the PLY engine integration.
//! All PLY integration errors are centralized here for consistent error reporting and debugging.

use std::fmt;

/// Comprehensive error type for PLY integration
#[derive(Debug, Clone, PartialEq)]
pub enum PlyIntegrationError {
    /// Audio-related errors
    Audio {
        message: String,
        source: AudioErrorSource,
    },

    /// Input handling errors
    Input {
        message: String,
        source: InputErrorSource,
    },

    /// Game logic errors
    GameLogic {
        message: String,
        source: GameLogicErrorSource,
    },

    /// UI framework errors
    Ui {
        message: String,
        source: UiErrorSource,
    },

    /// Song library errors
    SongLibrary {
        message: String,
        source: SongLibraryErrorSource,
    },

    /// Rendering errors
    Rendering {
        message: String,
        source: RenderingErrorSource,
    },
}

/// Audio error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioErrorSource {
    /// MIDI output connection failed
    MidiConnection,
    /// Audio device error
    AudioDevice,
    /// Gain control error
    GainControl,
    /// Audio event processing error
    EventProcessing,
    /// Synthesizer error
    Synthesizer,
}

/// Input handling error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputErrorSource {
    /// Keyboard input error
    Keyboard,
    /// Mouse input error
    Mouse,
    /// Gamepad input error
    Gamepad,
    /// Input binding error
    Binding,
    /// Keyboard-to-MIDI conversion error
    KeyboardToMidi,
}

/// Game logic error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameLogicErrorSource {
    /// Play-along system error
    PlayAlong,
    /// Rewind controller error
    RewindController,
    /// LUMI controller error
    LumiController,
    /// Timing statistics error
    TimingStats,
}

/// UI framework error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiErrorSource {
    /// UI initialization error
    Initialization,
    /// Layout error
    Layout,
    /// Widget state error
    WidgetState,
    /// Render command error
    RenderCommand,
    /// Layer stack error
    LayerStack,
}

/// Song library error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongLibraryErrorSource {
    /// Database error
    Database,
    /// File scanning error
    Scanning,
    /// Statistics calculation error
    Statistics,
    /// Cache error
    Cache,
}

/// Rendering error sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingErrorSource {
    /// Waterfall renderer error
    Waterfall,
    /// Keyboard renderer error
    Keyboard,
    /// Guideline renderer error
    Guideline,
    /// Note labels renderer error
    NoteLabels,
    /// Effects renderer error
    Effects,
    /// Renderer coordinator error
    Coordinator,
}

impl fmt::Display for PlyIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlyIntegrationError::Audio { message, source } => {
                write!(f, "PLY Audio Error [{}]: {}", source, message)
            }
            PlyIntegrationError::Input { message, source } => {
                write!(f, "PLY Input Error [{}]: {}", source, message)
            }
            PlyIntegrationError::GameLogic { message, source } => {
                write!(f, "PLY Game Logic Error [{}]: {}", source, message)
            }
            PlyIntegrationError::Ui { message, source } => {
                write!(f, "PLY UI Error [{}]: {}", source, message)
            }
            PlyIntegrationError::SongLibrary { message, source } => {
                write!(f, "PLY Song Library Error [{}]: {}", source, message)
            }
            PlyIntegrationError::Rendering { message, source } => {
                write!(f, "PLY Rendering Error [{}]: {}", source, message)
            }
        }
    }
}

impl fmt::Display for AudioErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioErrorSource::MidiConnection => write!(f, "MIDI Connection"),
            AudioErrorSource::AudioDevice => write!(f, "Audio Device"),
            AudioErrorSource::GainControl => write!(f, "Gain Control"),
            AudioErrorSource::EventProcessing => write!(f, "Event Processing"),
            AudioErrorSource::Synthesizer => write!(f, "Synthesizer"),
        }
    }
}

impl fmt::Display for InputErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputErrorSource::Keyboard => write!(f, "Keyboard"),
            InputErrorSource::Mouse => write!(f, "Mouse"),
            InputErrorSource::Gamepad => write!(f, "Gamepad"),
            InputErrorSource::Binding => write!(f, "Binding"),
            InputErrorSource::KeyboardToMidi => write!(f, "Keyboard-to-MIDI"),
        }
    }
}

impl fmt::Display for GameLogicErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameLogicErrorSource::PlayAlong => write!(f, "Play Along"),
            GameLogicErrorSource::RewindController => write!(f, "Rewind Controller"),
            GameLogicErrorSource::LumiController => write!(f, "LUMI Controller"),
            GameLogicErrorSource::TimingStats => write!(f, "Timing Statistics"),
        }
    }
}

impl fmt::Display for UiErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiErrorSource::Initialization => write!(f, "Initialization"),
            UiErrorSource::Layout => write!(f, "Layout"),
            UiErrorSource::WidgetState => write!(f, "Widget State"),
            UiErrorSource::RenderCommand => write!(f, "Render Command"),
            UiErrorSource::LayerStack => write!(f, "Layer Stack"),
        }
    }
}

impl fmt::Display for SongLibraryErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongLibraryErrorSource::Database => write!(f, "Database"),
            SongLibraryErrorSource::Scanning => write!(f, "Scanning"),
            SongLibraryErrorSource::Statistics => write!(f, "Statistics"),
            SongLibraryErrorSource::Cache => write!(f, "Cache"),
        }
    }
}

impl fmt::Display for RenderingErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderingErrorSource::Waterfall => write!(f, "Waterfall Renderer"),
            RenderingErrorSource::Keyboard => write!(f, "Keyboard Renderer"),
            RenderingErrorSource::Guideline => write!(f, "Guideline Renderer"),
            RenderingErrorSource::NoteLabels => write!(f, "Note Labels Renderer"),
            RenderingErrorSource::Effects => write!(f, "Effects Renderer"),
            RenderingErrorSource::Coordinator => write!(f, "Renderer Coordinator"),
        }
    }
}

impl std::error::Error for PlyIntegrationError {}

/// Result type alias for PLY integration operations
pub type PlyResult<T> = Result<T, PlyIntegrationError>;

/// Helper macro for creating audio errors
#[macro_export]
macro_rules! ply_audio_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::Audio {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

/// Helper macro for creating input errors
#[macro_export]
macro_rules! ply_input_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::Input {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

/// Helper macro for creating game logic errors
#[macro_export]
macro_rules! ply_game_logic_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::GameLogic {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

/// Helper macro for creating UI errors
#[macro_export]
macro_rules! ply_ui_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::Ui {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

/// Helper macro for creating song library errors
#[macro_export]
macro_rules! ply_song_library_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::SongLibrary {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

/// Helper macro for creating rendering errors
#[macro_export]
macro_rules! ply_rendering_error {
    ($source:expr, $($arg:tt)*) => {
        $crate::ply_integration::error::PlyIntegrationError::Rendering {
            message: format!($($arg)*),
            source: $source,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PlyIntegrationError::Audio {
            message: "Failed to connect to MIDI device".to_string(),
            source: AudioErrorSource::MidiConnection,
        };

        let display_str = format!("{}", err);
        assert!(display_str.contains("MIDI Connection"));
        assert!(display_str.contains("Failed to connect to MIDI device"));
    }

    #[test]
    fn test_all_error_types() {
        // Test that all error types can be created and displayed
        let errors = vec![
            PlyIntegrationError::Audio {
                message: "test".to_string(),
                source: AudioErrorSource::MidiConnection,
            },
            PlyIntegrationError::Input {
                message: "test".to_string(),
                source: InputErrorSource::Keyboard,
            },
            PlyIntegrationError::GameLogic {
                message: "test".to_string(),
                source: GameLogicErrorSource::PlayAlong,
            },
            PlyIntegrationError::Ui {
                message: "test".to_string(),
                source: UiErrorSource::Initialization,
            },
            PlyIntegrationError::SongLibrary {
                message: "test".to_string(),
                source: SongLibraryErrorSource::Database,
            },
            PlyIntegrationError::Rendering {
                message: "test".to_string(),
                source: RenderingErrorSource::Waterfall,
            },
        ];

        for err in errors {
            let _ = format!("{}", err); // Should not panic
        }
    }
}
