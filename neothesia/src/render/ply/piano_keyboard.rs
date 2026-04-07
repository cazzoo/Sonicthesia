//! Enhanced piano keyboard renderer for PLY mode
//!
//! Provides an interactive, animated piano keyboard with:
//! - Visual feedback on key press
//! - Mouse click and drag support
//! - Responsive sizing and positioning
//! - Theme/style customization
//! - Keyboard input synchronization

use crate::virtual_resolution::{vh, vw};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use piano_layout::KeyboardLayout;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Input source for key press events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputSource {
    Mouse,
    Keyboard,
    Midi,
    Gamepad,
}

/// Color configuration for a single note
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NoteColor {
    /// Normal state color (RGB)
    pub normal: (u8, u8, u8),
    /// Pressed state color (RGB)
    pub pressed: (u8, u8, u8),
    /// Glow color (RGB) - optional, defaults to pressed color
    #[serde(default)]
    pub glow: Option<(u8, u8, u8)>,
    /// Glow intensity multiplier (0.0 - 2.0)
    #[serde(default = "default_glow_intensity")]
    pub glow_intensity: f32,
}

fn default_glow_intensity() -> f32 {
    1.0
}

impl Default for NoteColor {
    fn default() -> Self {
        Self {
            normal: (255, 255, 255),
            pressed: (100, 200, 255),
            glow: None,
            glow_intensity: 1.0,
        }
    }
}

impl NoteColor {
    /// Get the effective glow color (pressed color or custom)
    pub fn glow_color(&self) -> (u8, u8, u8) {
        self.glow.unwrap_or(self.pressed)
    }
}

/// Global theme settings that apply to all keys
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSettings {
    /// Border color for all keys
    #[serde(default)]
    pub border_color: (u8, u8, u8),

    /// Rounded corner radius
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,

    /// 2.5D effect depth (0.0 = flat, higher = more depth)
    #[serde(default = "default_depth_2d5")]
    pub depth_2d5: f32,

    /// Whether to use per-note colors or fall back to simple white/black
    #[serde(default = "default_use_per_note_colors")]
    pub use_per_note_colors: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            border_color: (0, 0, 0),
            corner_radius: 4.0,
            depth_2d5: 3.0,
            use_per_note_colors: true,
        }
    }
}

fn default_corner_radius() -> f32 {
    4.0
}
fn default_depth_2d5() -> f32 {
    3.0
}
fn default_use_per_note_colors() -> bool {
    true
}

/// Theme colors for all 12 notes in an octave
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OctaveTheme {
    /// Colors for each note in the octave (0=C, 1=C#, ..., 11=B)
    #[serde(default)]
    pub notes: [NoteColor; 12],

    /// Global theme settings
    #[serde(default)]
    pub settings: ThemeSettings,
}

impl Default for OctaveTheme {
    fn default() -> Self {
        Self {
            notes: Self::default_notes(),
            settings: ThemeSettings::default(),
        }
    }
}

impl OctaveTheme {
    /// Create default note colors (white/black key pattern)
    fn default_notes() -> [NoteColor; 12] {
        // White keys: C, D, E, F, G, A, B (indices 0, 2, 4, 5, 7, 9, 11)
        // Black keys: C#, D#, F#, G#, A# (indices 1, 3, 6, 8, 10)
        let white = NoteColor {
            normal: (255, 255, 255),
            pressed: (76, 175, 80),
            glow: None,
            glow_intensity: 1.0,
        };
        let black = NoteColor {
            normal: (26, 26, 26),
            pressed: (46, 125, 50),
            glow: None,
            glow_intensity: 1.0,
        };

        let is_sharp = [
            false, true, false, true, false, false, true, false, true, false, true, false,
        ];
        is_sharp.map(|sharp| if sharp { black.clone() } else { white.clone() })
    }

    /// Get color for a specific note (0-11)
    pub fn note_color(&self, note_index: usize) -> &NoteColor {
        &self.notes[note_index % 12]
    }
}

/// Theme style variants
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ThemeVariant {
    /// Classic piano appearance
    Classic,
    /// Modern with vibrant colors
    Modern,
    /// Flat design without 3D effects
    Flat,
    /// Enhanced 2.5D depth effect
    Depth2D5,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Modern
    }
}

/// A complete keyboard theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyboardTheme {
    /// Theme name
    pub name: String,

    /// Per-octave color scheme
    pub octave_theme: OctaveTheme,

    /// Theme variant for different styles
    #[serde(default)]
    pub variant: ThemeVariant,
}

/// Theme name enum for predefined themes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ThemeName {
    Classic,
    Modern,
    Rainbow,
    Neon,
    Pastel,
}

impl ThemeName {
    /// Get the theme name as a string
    pub fn as_str(&self) -> &str {
        match self {
            ThemeName::Classic => "Classic",
            ThemeName::Modern => "Modern",
            ThemeName::Rainbow => "Rainbow",
            ThemeName::Neon => "Neon",
            ThemeName::Pastel => "Pastel",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Classic" => Some(ThemeName::Classic),
            "Modern" => Some(ThemeName::Modern),
            "Rainbow" => Some(ThemeName::Rainbow),
            "Neon" => Some(ThemeName::Neon),
            "Pastel" => Some(ThemeName::Pastel),
            _ => None,
        }
    }
}

impl KeyboardTheme {
    /// Classic piano theme (black and white)
    pub fn classic() -> Self {
        let white = NoteColor {
            normal: (255, 255, 255),
            pressed: (200, 200, 200),
            glow: None,
            glow_intensity: 0.0,
        };
        let black = NoteColor {
            normal: (26, 26, 26),
            pressed: (60, 60, 60),
            glow: None,
            glow_intensity: 0.0,
        };

        let is_sharp = [
            false, true, false, true, false, false, true, false, true, false, true, false,
        ];
        let notes: [NoteColor; 12] =
            is_sharp.map(|sharp| if sharp { black.clone() } else { white.clone() });

        Self {
            name: "Classic".to_string(),
            octave_theme: OctaveTheme {
                notes,
                settings: ThemeSettings {
                    border_color: (0, 0, 0),
                    corner_radius: 2.0,
                    depth_2d5: 0.0,
                    use_per_note_colors: true,
                },
            },
            variant: ThemeVariant::Classic,
        }
    }

    /// Modern theme with green highlights
    pub fn modern() -> Self {
        Self {
            name: "Modern".to_string(),
            octave_theme: OctaveTheme::default(),
            variant: ThemeVariant::Modern,
        }
    }

    pub fn classic_colors() -> Self {
        let white = NoteColor {
            normal: (255, 255, 255),
            pressed: (100, 180, 255),
            glow: None,
            glow_intensity: 0.5,
        };
        let black = NoteColor {
            normal: (26, 26, 26),
            pressed: (60, 120, 200),
            glow: None,
            glow_intensity: 0.5,
        };

        let is_sharp = [
            false, true, false, true, false, false, true, false, true, false, true, false,
        ];
        let notes: [NoteColor; 12] =
            is_sharp.map(|sharp| if sharp { black.clone() } else { white.clone() });

        Self {
            name: "Classic Colors".to_string(),
            octave_theme: OctaveTheme {
                notes,
                settings: ThemeSettings {
                    border_color: (0, 0, 0),
                    corner_radius: 2.0,
                    depth_2d5: 1.0,
                    use_per_note_colors: true,
                },
            },
            variant: ThemeVariant::Classic,
        }
    }

    /// Rainbow theme - each note has a unique color
    pub fn rainbow() -> Self {
        let notes = [
            // C - Red
            NoteColor {
                normal: (255, 200, 200),
                pressed: (255, 50, 50),
                glow: None,
                glow_intensity: 1.2,
            },
            // C# - Red-Orange
            NoteColor {
                normal: (200, 150, 150),
                pressed: (255, 100, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // D - Orange
            NoteColor {
                normal: (255, 220, 180),
                pressed: (255, 140, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // D# - Yellow-Orange
            NoteColor {
                normal: (200, 160, 130),
                pressed: (255, 180, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // E - Yellow
            NoteColor {
                normal: (255, 255, 200),
                pressed: (255, 220, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // F - Yellow-Green
            NoteColor {
                normal: (220, 255, 200),
                pressed: (180, 255, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // F# - Green
            NoteColor {
                normal: (160, 200, 150),
                pressed: (0, 200, 0),
                glow: None,
                glow_intensity: 1.2,
            },
            // G - Cyan-Green
            NoteColor {
                normal: (180, 255, 220),
                pressed: (0, 255, 128),
                glow: None,
                glow_intensity: 1.2,
            },
            // G# - Cyan
            NoteColor {
                normal: (150, 200, 200),
                pressed: (0, 200, 200),
                glow: None,
                glow_intensity: 1.2,
            },
            // A - Blue
            NoteColor {
                normal: (200, 220, 255),
                pressed: (50, 100, 255),
                glow: None,
                glow_intensity: 1.2,
            },
            // A# - Blue-Purple
            NoteColor {
                normal: (180, 160, 200),
                pressed: (100, 0, 200),
                glow: None,
                glow_intensity: 1.2,
            },
            // B - Purple
            NoteColor {
                normal: (230, 200, 255),
                pressed: (180, 0, 255),
                glow: None,
                glow_intensity: 1.2,
            },
        ];

        Self {
            name: "Rainbow".to_string(),
            octave_theme: OctaveTheme {
                notes,
                settings: ThemeSettings {
                    border_color: (50, 50, 50),
                    corner_radius: 6.0,
                    depth_2d5: 4.0,
                    use_per_note_colors: true,
                },
            },
            variant: ThemeVariant::Modern,
        }
    }

    /// Neon theme with bright glowing colors
    pub fn neon() -> Self {
        let notes = [
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 0, 85),
                glow: Some((255, 0, 85)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (0, 255, 255),
                glow: Some((0, 255, 255)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 255, 0),
                glow: Some((255, 255, 0)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (0, 255, 0),
                glow: Some((0, 255, 0)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 0, 255),
                glow: Some((255, 0, 255)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (0, 128, 255),
                glow: Some((0, 128, 255)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 128, 0),
                glow: Some((255, 128, 0)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (128, 0, 255),
                glow: Some((128, 0, 255)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (0, 255, 128),
                glow: Some((0, 255, 128)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 0, 128),
                glow: Some((255, 0, 128)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (128, 255, 0),
                glow: Some((128, 255, 0)),
                glow_intensity: 1.5,
            },
            NoteColor {
                normal: (30, 30, 30),
                pressed: (255, 85, 0),
                glow: Some((255, 85, 0)),
                glow_intensity: 1.5,
            },
        ];

        Self {
            name: "Neon".to_string(),
            octave_theme: OctaveTheme {
                notes,
                settings: ThemeSettings {
                    border_color: (20, 20, 20),
                    corner_radius: 8.0,
                    depth_2d5: 2.0,
                    use_per_note_colors: true,
                },
            },
            variant: ThemeVariant::Modern,
        }
    }

    /// Pastel theme with soft colors
    pub fn pastel() -> Self {
        let notes = [
            NoteColor {
                normal: (255, 240, 245),
                pressed: (255, 182, 193),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (240, 248, 255),
                pressed: (173, 216, 230),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (255, 250, 240),
                pressed: (255, 228, 181),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (245, 255, 250),
                pressed: (152, 251, 152),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (255, 240, 245),
                pressed: (255, 192, 203),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (240, 255, 240),
                pressed: (144, 238, 144),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (230, 230, 250),
                pressed: (147, 112, 219),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (255, 245, 238),
                pressed: (255, 218, 185),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (240, 255, 255),
                pressed: (175, 238, 238),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (255, 250, 250),
                pressed: (255, 160, 122),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (245, 255, 245),
                pressed: (154, 205, 154),
                glow: None,
                glow_intensity: 0.8,
            },
            NoteColor {
                normal: (250, 240, 230),
                pressed: (255, 200, 150),
                glow: None,
                glow_intensity: 0.8,
            },
        ];

        Self {
            name: "Pastel".to_string(),
            octave_theme: OctaveTheme {
                notes,
                settings: ThemeSettings {
                    border_color: (200, 200, 200),
                    corner_radius: 5.0,
                    depth_2d5: 2.0,
                    use_per_note_colors: true,
                },
            },
            variant: ThemeVariant::Flat,
        }
    }

    /// Get all available predefined themes
    pub fn predefined_themes() -> Vec<Self> {
        vec![
            Self::classic(),
            Self::modern(),
            Self::rainbow(),
            Self::neon(),
            Self::pastel(),
        ]
    }

    /// Get theme by name
    pub fn get_theme(name: &str) -> Option<Self> {
        match name {
            "Classic" => Some(Self::classic()),
            "Modern" => Some(Self::modern()),
            "Classic Colors" => Some(Self::classic_colors()),
            "Rainbow" => Some(Self::rainbow()),
            "Neon" => Some(Self::neon()),
            "Pastel" => Some(Self::pastel()),
            _ => None,
        }
    }
}

/// Visual theme for the piano keyboard (legacy, kept for compatibility)
#[derive(Clone, Debug)]
pub struct PianoTheme {
    /// White key color (normal)
    pub white_key_normal: Color,
    /// White key color (pressed)
    pub white_key_pressed: Color,
    /// Black key color (normal)
    pub black_key_normal: Color,
    /// Black key color (pressed)
    pub black_key_pressed: Color,
    /// Border color
    pub border_color: Color,
    /// Glow effect intensity (0.0 - 1.0)
    pub glow_intensity: f32,
    /// Rounded corner radius
    pub corner_radius: f32,
    /// 2.5D effect depth (0.0 = flat, higher = more depth)
    pub depth_2d5: f32,
}

impl Default for PianoTheme {
    fn default() -> Self {
        Self {
            white_key_normal: Color::from_hex(0xffffff),
            white_key_pressed: Color::from_hex(0x4CAF50),
            black_key_normal: Color::from_hex(0x1a1a1a),
            black_key_pressed: Color::from_hex(0x2E7D32),
            border_color: Color::from_hex(0x000000),
            glow_intensity: 0.3,
            corner_radius: 4.0,
            depth_2d5: 3.0,
        }
    }
}

/// Preset themes
impl PianoTheme {
    /// Classic piano look (black and white)
    pub fn classic() -> Self {
        Self {
            white_key_normal: Color::from_hex(0xffffff),
            white_key_pressed: Color::from_hex(0xcccccc),
            black_key_normal: Color::from_hex(0x1a1a1a),
            black_key_pressed: Color::from_hex(0x333333),
            border_color: Color::from_hex(0x000000),
            glow_intensity: 0.0,
            corner_radius: 2.0,
            depth_2d5: 0.0,
        }
    }

    /// Modern look with green highlights
    pub fn modern() -> Self {
        Self::default()
    }

    /// 2.5D effect with depth
    pub fn depth_2d5() -> Self {
        Self {
            white_key_normal: Color::from_hex(0xf5f5f5),
            white_key_pressed: Color::from_hex(0x66BB6A),
            black_key_normal: Color::from_hex(0x212121),
            black_key_pressed: Color::from_hex(0x43A047),
            border_color: Color::from_hex(0x424242),
            glow_intensity: 0.4,
            corner_radius: 6.0,
            depth_2d5: 8.0,
        }
    }

    /// Flat design (no 3D effects)
    pub fn flat() -> Self {
        Self {
            white_key_normal: Color::from_hex(0xffffff),
            white_key_pressed: Color::from_hex(0xFF9800),
            black_key_normal: Color::from_hex(0x263238),
            black_key_pressed: Color::from_hex(0xFFB74D),
            border_color: Color::from_hex(0xECEFF1),
            glow_intensity: 0.2,
            corner_radius: 0.0,
            depth_2d5: 0.0,
        }
    }
}

/// Animation state for a key
#[derive(Clone, Debug)]
struct KeyAnimation {
    /// Animation value (0.0 = released, 1.0 = fully pressed)
    value: f32,
    /// Target value
    target: f32,
    /// Animation speed (higher = faster)
    speed: f32,
}

impl KeyAnimation {
    fn new() -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            speed: 8.0, // Animation speed
        }
    }

    fn update(&mut self, dt: f32, is_actually_pressed: bool) {
        // DEBUG: Log animation update entry
        log::debug!(
            "[DEBUG] [KeyAnimation::update] Entry\n  - Context: dt={:.4}, is_actually_pressed={}, current_value={:.3}, current_target={:.3}",
            dt, is_actually_pressed, self.value, self.target
        );

        // If the key is actually pressed by any input source, ensure target is 1.0
        // This prevents the animation from fading out while still pressed
        if is_actually_pressed {
            self.target = 1.0;
            // Immediately set to pressed state for instant visual feedback
            // This matches the behavior of the press() method
            self.value = 1.0;
            log::debug!(
                "[DEBUG] [KeyAnimation::update] Action: Setting value to 1.0 (key pressed)\n  - New state: value={:.3}, target={:.3}",
                self.value, self.target
            );
        } else {
            self.target = 0.0;
            // Interpolate toward 0.0 for smooth release animation
            let diff = self.target - self.value;
            self.value += diff * self.speed * dt;
            log::debug!(
                "[DEBUG] [KeyAnimation::update] Action: Interpolating toward 0.0 (key released)\n  - diff={:.3}, new_value={:.3}, target={:.3}",
                diff, self.value, self.target
            );
        }

        self.value = self.value.clamp(0.0, 1.0);

        // DEBUG: Log animation state changes
        if self.value < 0.99 && is_actually_pressed {
            log::warn!(
                "[DEBUG] [KeyAnimation::update] WARNING: Key is pressed but animation value is {:.3} (target: {:.3})",
                self.value, self.target
            );
        }

        log::debug!(
            "[DEBUG] [KeyAnimation::update] Exit\n  - Final state: value={:.3}, target={:.3}",
            self.value,
            self.target
        );
    }

    fn press(&mut self) {
        self.target = 1.0;
        // Immediately set to pressed state for instant visual feedback
        self.value = 1.0;
    }

    fn release(&mut self) {
        self.target = 0.0;
        // Don't immediately reset value - let it interpolate naturally
        // This ensures smooth transitions and prevents visual glitches
    }

    fn is_pressed(&self) -> bool {
        self.target > 0.5
    }
}

/// Visual state of a piano key
#[derive(Clone, Debug)]
struct VisualKey {
    note: u8,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    is_sharp: bool,
    animation: KeyAnimation,
}

/// Enhanced piano keyboard renderer
pub struct PianoKeyboardRenderer {
    layout: KeyboardLayout,
    keys: Vec<VisualKey>,
    theme: KeyboardTheme,
    position: (f32, f32),
    size: (f32, f32),
    window_width: f32,
    window_height: f32,

    // Mouse interaction state
    mouse_pressed_keys: HashMap<u8, bool>,
    mouse_is_down: bool,

    // Keyboard input state (sync with MIDI events)
    keyboard_pressed_notes: HashMap<u8, bool>,

    // Unified pressed state tracking
    // Maps note -> set of input sources that have this key pressed
    pressed_keys_sources: HashMap<u8, HashSet<InputSource>>,
}

impl PianoKeyboardRenderer {
    /// Create a new piano keyboard renderer
    pub fn new(layout: KeyboardLayout, config: &Config) -> Self {
        let window_width = vw();
        let window_height = vh();

        let (width, height) = Self::calculate_keyboard_size(window_width, window_height);
        let (x, y) = Self::calculate_keyboard_position(width, height, window_height);

        let keys = Self::build_visual_keys(&layout, x, y, width, height);

        // Load theme from config
        let theme_name = config.piano_theme_name();
        let theme = KeyboardTheme::get_theme(theme_name).unwrap_or_else(|| {
            log::warn!(
                "Unknown theme name '{}', falling back to Modern",
                theme_name
            );
            KeyboardTheme::modern()
        });

        Self {
            layout,
            keys,
            theme,
            position: (x, y),
            size: (width, height),
            window_width,
            window_height,
            mouse_pressed_keys: HashMap::new(),
            mouse_is_down: false,
            keyboard_pressed_notes: HashMap::new(),
            pressed_keys_sources: HashMap::new(),
        }
    }

    /// Set the visual theme
    pub fn set_theme(&mut self, theme: KeyboardTheme) {
        self.theme = theme;
    }

    /// Get note color for a specific MIDI note
    fn get_note_color(&self, note: u8) -> &NoteColor {
        let note_index = (note % 12) as usize;
        self.theme.octave_theme.note_color(note_index)
    }

    /// Update keyboard layout and recalculate key positions
    pub fn set_layout(&mut self, layout: KeyboardLayout) {
        let (width, height) = self.size;
        let (x, y) = self.position;

        self.keys = Self::build_visual_keys(&layout, x, y, width, height);
        self.layout = layout;
    }

    /// Rebuild keyboard layout from config range
    pub fn rebuild_from_config(&mut self, config: &Config) {
        let range = config.piano_range();
        let keyboard_range = piano_layout::KeyboardRange::new(range.clone());
        let sizing = piano_layout::Sizing::new(40.0, 120.0);
        let layout = KeyboardLayout::from_range(sizing, keyboard_range);
        self.set_layout(layout);
    }

    /// Update window size and recalculate positions
    pub fn update_window_size(&mut self) {
        let new_width = vw();
        let new_height = vh();

        if new_width != self.window_width || new_height != self.window_height {
            self.window_width = new_width;
            self.window_height = new_height;

            let (width, height) = Self::calculate_keyboard_size(new_width, new_height);
            let (x, y) = Self::calculate_keyboard_position(width, height, new_height);

            self.size = (width, height);
            self.position = (x, y);

            self.keys = Self::build_visual_keys(&self.layout, x, y, width, height);
        }
    }

    pub fn set_position_and_size(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.window_width = vw();
        self.window_height = vh();
        self.size = (w, h);
        self.position = (x, y);
        self.keys = Self::build_visual_keys(&self.layout, x, y, w, h);
    }

    /// Calculate keyboard size based on window dimensions
    fn calculate_keyboard_size(window_width: f32, window_height: f32) -> (f32, f32) {
        let width = window_width * 0.95; // 95% of window width
        let height = window_height * 0.2; // 20% of window height
        (width, height)
    }

    /// Calculate keyboard position (bottom center)
    fn calculate_keyboard_position(width: f32, height: f32, window_height: f32) -> (f32, f32) {
        let x = (vw() - width) / 2.0;
        let y = window_height - height - 20.0; // 20px from bottom
        (x, y)
    }

    fn build_visual_keys(
        layout: &KeyboardLayout,
        offset_x: f32,
        offset_y: f32,
        total_width: f32,
        total_height: f32,
    ) -> Vec<VisualKey> {
        let scale_x = total_width / layout.width;

        let range_start = layout.range.start();

        layout
            .keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let x = offset_x + key.x() * scale_x;
                let y = offset_y;
                let width = key.width() * scale_x;
                let height = if key.kind().is_sharp() {
                    total_height * 0.65
                } else {
                    total_height
                };
                let note_id = key.note_id();

                // Calculate the actual MIDI note number
                // The note_id is the position within the octave (0-11)
                // We need to add the octave offset to get the actual MIDI note number
                let midi_note = range_start + i as u8;

                VisualKey {
                    note: midi_note,
                    x,
                    y,
                    width,
                    height,
                    is_sharp: key.kind().is_sharp(),
                    animation: KeyAnimation::new(),
                }
            })
            .collect()
    }

    /// Handle mouse input
    pub fn handle_mouse_input(
        &mut self,
        mouse_pos: Vec2,
        mouse_button: MouseButton,
        is_pressed: bool,
    ) -> Option<Vec<u8>> {
        log::debug!(
            "[DEBUG] [handle_mouse_input] Entry\n  - Context: mouse_pos=({:.1},{:.1}), mouse_button={:?}, is_pressed={}",
            mouse_pos.x, mouse_pos.y, mouse_button, is_pressed
        );

        if mouse_button != MouseButton::Left {
            log::debug!("[DEBUG] [handle_mouse_input] Exit: Not left mouse button, ignoring");
            return None;
        }

        if is_pressed {
            self.mouse_is_down = true;
            if let Some(note) = self.get_key_at_position(mouse_pos) {
                log::debug!(
                    "[DEBUG] [handle_mouse_input] Action: Key pressed at position, note={}",
                    note
                );
                self.mouse_pressed_keys.insert(note, true);
                // Add to unified pressed state
                self.add_input_source(note, InputSource::Mouse);
                return Some(vec![note]);
            } else {
                log::debug!("[DEBUG] [handle_mouse_input] No key at position");
            }
        } else {
            self.mouse_is_down = false;
            let released_notes: Vec<u8> = self.mouse_pressed_keys.keys().copied().collect();
            log::debug!(
                "[DEBUG] [handle_mouse_input] Action: Releasing {} notes: {:?}",
                released_notes.len(),
                released_notes
            );
            for note in &released_notes {
                // Remove from unified pressed state
                self.remove_input_source(*note, InputSource::Mouse);
            }
            self.mouse_pressed_keys.clear();
            return Some(released_notes);
        }

        log::debug!("[DEBUG] [handle_mouse_input] Exit: No action taken");
        None
    }

    /// Handle mouse drag (return notes that started being pressed)
    pub fn handle_mouse_drag(&mut self, mouse_pos: Vec2) -> Option<Vec<u8>> {
        log::debug!(
            "[DEBUG] [handle_mouse_drag] Entry\n  - Context: mouse_pos=({:.1},{:.1}), mouse_is_down={}",
            mouse_pos.x, mouse_pos.y, self.mouse_is_down
        );

        if !self.mouse_is_down {
            log::debug!("[DEBUG] [handle_mouse_drag] Exit: Mouse not down, ignoring");
            return None;
        }

        let mut new_pressed_notes = Vec::new();

        if let Some(note) = self.get_key_at_position(mouse_pos) {
            if !self.mouse_pressed_keys.contains_key(&note) {
                // New key pressed during drag
                log::debug!(
                    "[DEBUG] [handle_mouse_drag] Action: New key pressed during drag, note={}",
                    note
                );
                self.mouse_pressed_keys.insert(note, true);
                // Add to unified pressed state
                self.add_input_source(note, InputSource::Mouse);
                new_pressed_notes.push(note);
            }
        }

        // Check for released keys (mouse moved off a key)
        let keys_to_release: Vec<u8> = self
            .mouse_pressed_keys
            .iter()
            .filter(|(note, _)| {
                if let Some(key) = self.get_key_key(**note) {
                    !self.is_point_in_key(mouse_pos, &key)
                } else {
                    false
                }
            })
            .map(|(note, _)| *note)
            .collect();

        if !keys_to_release.is_empty() {
            log::debug!(
                "[DEBUG] [handle_mouse_drag] Action: Releasing {} keys during drag: {:?}",
                keys_to_release.len(),
                keys_to_release
            );
        }

        for note in keys_to_release {
            self.mouse_pressed_keys.remove(&note);
            // Remove from unified pressed state
            self.remove_input_source(note, InputSource::Mouse);
        }

        if new_pressed_notes.is_empty() {
            log::debug!("[DEBUG] [handle_mouse_drag] Exit: No new keys pressed");
            None
        } else {
            log::debug!(
                "[DEBUG] [handle_mouse_drag] Exit: Returning {} new pressed notes: {:?}",
                new_pressed_notes.len(),
                new_pressed_notes
            );
            Some(new_pressed_notes)
        }
    }

    fn get_key_at_position(&self, pos: Vec2) -> Option<u8> {
        // Check black keys first (they're on top)
        for key in self.keys.iter().filter(|k| k.is_sharp) {
            if self.is_point_in_key(pos, key) {
                return Some(key.note);
            }
        }

        // Then check white keys
        for key in self.keys.iter().filter(|k| !k.is_sharp) {
            if self.is_point_in_key(pos, key) {
                return Some(key.note);
            }
        }

        None
    }

    /// Check if point is within key bounds
    fn is_point_in_key(&self, pos: Vec2, key: &VisualKey) -> bool {
        pos.x >= key.x
            && pos.x <= key.x + key.width
            && pos.y >= key.y
            && pos.y <= key.y + key.height
    }

    /// Get visual key by note number
    fn get_render_key(&self, note: u8) -> Option<&VisualKey> {
        self.keys.iter().find(|k| k.note == note)
    }

    /// Get mutable visual key by note number
    fn get_render_key_mut(&mut self, note: u8) -> Option<&mut VisualKey> {
        self.keys.iter_mut().find(|k| k.note == note)
    }

    /// Get key struct (helper for drag detection)
    fn get_key_key(&self, note: u8) -> Option<VisualKey> {
        self.get_render_key(note).cloned()
    }

    fn set_key_pressed(&mut self, note: u8, pressed: bool) {
        // DEBUG: Log when set_key_pressed is called
        let is_actually_pressed = self.is_key_pressed(note);
        log::debug!(
            "🎹 set_key_pressed(note={}, pressed={}, is_actually_pressed={})",
            note,
            pressed,
            is_actually_pressed
        );

        if let Some(key) = self.get_render_key_mut(note) {
            if pressed {
                log::debug!(
                    "🎹   → Calling animation.press(), current value={:.3}",
                    key.animation.value
                );
                key.animation.press();
            } else {
                log::debug!(
                    "🎹   → Calling animation.release(), current value={:.3}",
                    key.animation.value
                );
                key.animation.release();
            }
        }
    }

    /// Handle keyboard MIDI note events
    pub fn handle_note_event(&mut self, note: u8, velocity: u8) {
        let pressed = velocity > 0;

        log::debug!(
            "[DEBUG] [handle_note_event] Entry\n  - Context: note={}, velocity={}, pressed={}",
            note,
            velocity,
            pressed
        );

        // Update keyboard state tracking
        if pressed {
            self.keyboard_pressed_notes.insert(note, true);
            // Add to unified pressed state
            log::debug!(
                "[DEBUG] [handle_note_event] Action: Calling add_input_source(note={}, source=Midi)",
                note
            );
            self.add_input_source(note, InputSource::Midi);
        } else {
            self.keyboard_pressed_notes.remove(&note);
            // Remove from unified pressed state
            log::debug!(
                "[DEBUG] [handle_note_event] Action: Calling remove_input_source(note={}, source=Midi)",
                note
            );
            self.remove_input_source(note, InputSource::Midi);
        }

        log::debug!(
            "[DEBUG] [handle_note_event] Exit\n  - Result: pressed_keys_sources={:?}",
            self.pressed_keys_sources
        );
    }

    /// Highlight a key visually without triggering audio (for learn mode)
    pub fn highlight_key(&mut self, note: u8, highlight: bool) {
        self.set_key_pressed(note, highlight);
    }

    /// Add an input source for a pressed key
    fn add_input_source(&mut self, note: u8, source: InputSource) {
        log::debug!(
            "[DEBUG] [add_input_source] Entry\n  - Context: note={}, source={:?}",
            note,
            source
        );

        self.pressed_keys_sources
            .entry(note)
            .or_insert_with(HashSet::new)
            .insert(source);

        // Log the updated state (clone to avoid borrow checker issues)
        let sources = self.pressed_keys_sources.get(&note).cloned();
        let all_keys = self.pressed_keys_sources.clone();
        log::debug!(
            "[DEBUG] [add_input_source] Action: Added source\n  - Result: note={} now has {:?} sources\n  - All pressed keys: {:?}",
            note, sources, all_keys
        );
    }

    /// Remove an input source for a pressed key
    fn remove_input_source(&mut self, note: u8, source: InputSource) {
        log::debug!(
            "[DEBUG] [remove_input_source] Entry\n  - Context: note={}, source={:?}",
            note,
            source
        );

        if let Some(sources) = self.pressed_keys_sources.get_mut(&note) {
            let before_count = sources.len();
            sources.remove(&source);
            let after_count = sources.len();

            log::debug!(
                "[DEBUG] [remove_input_source] Action: Removed source\n  - Result: source count went from {} to {}",
                before_count, after_count
            );

            if sources.is_empty() {
                self.pressed_keys_sources.remove(&note);
                let all_keys = self.pressed_keys_sources.clone();
                log::debug!(
                    "[DEBUG] [remove_input_source] Action: No more sources for note={}, removing from pressed_keys_sources\n  - Remaining pressed keys: {:?}",
                    note, all_keys
                );
            } else {
                let sources_clone = sources.clone();
                let all_keys = self.pressed_keys_sources.clone();
                log::debug!(
                    "[DEBUG] [remove_input_source] Result: note={} still has {:?} sources\n  - All pressed keys: {:?}",
                    note, sources_clone, all_keys
                );
            }
        } else {
            log::debug!(
                "[DEBUG] [remove_input_source] Warning: note={} not found in pressed_keys_sources",
                note
            );
        }
    }

    /// Check if a key is pressed by any input source
    fn is_key_pressed(&self, note: u8) -> bool {
        let is_pressed = self.pressed_keys_sources.contains_key(&note);
        let sources = self.pressed_keys_sources.get(&note);

        log::debug!(
            "[DEBUG] [is_key_pressed] Called\n  - Context: note={}\n  - Result: is_pressed={}, sources={:?}",
            note, is_pressed, sources
        );

        is_pressed
    }

    /// Update animations
    pub fn update(&mut self, dt: f32) {
        log::debug!(
            "[DEBUG] [PianoKeyboardRenderer::update] Entry\n  - Context: dt={:.4}, total_keys={}",
            dt,
            self.keys.len()
        );

        self.update_window_size();

        // Collect pressed states first to avoid borrow checker issues
        let pressed_notes: std::collections::HashSet<u8> =
            self.pressed_keys_sources.keys().copied().collect();

        // DEBUG: Log pressed state
        if !pressed_notes.is_empty() {
            log::debug!(
                "[DEBUG] [PianoKeyboardRenderer::update] Render Loop\n  - Action: Updating {} keys\n  - Pressed notes: {:?}\n  - Pressed keys sources: {:?}",
                pressed_notes.len(), pressed_notes, self.pressed_keys_sources
            );
        } else {
            log::debug!(
                "[DEBUG] [PianoKeyboardRenderer::update] Render Loop\n  - Action: Updating {} keys\n  - No keys currently pressed",
                self.keys.len()
            );
        }

        for key in &mut self.keys {
            let is_actually_pressed = pressed_notes.contains(&key.note);
            log::debug!(
                "[DEBUG] [PianoKeyboardRenderer::update] Calling animation.update()\n  - Context: note={}, is_actually_pressed={}, current_animation_value={:.3}",
                key.note, is_actually_pressed, key.animation.value
            );
            key.animation.update(dt, is_actually_pressed);
        }

        log::debug!(
            "[DEBUG] [PianoKeyboardRenderer::update] Exit\n  - Action: Completed update for {} keys",
            self.keys.len()
        );
    }

    pub fn render(&self) {
        for key in self.keys.iter().filter(|k| !k.is_sharp) {
            self.render_key(key);
        }

        for key in self.keys.iter().filter(|k| k.is_sharp) {
            self.render_key(key);
        }
    }

    /// Render a single key
    fn render_key(&self, key: &VisualKey) {
        let note_color = self.get_note_color(key.note);
        let settings = &self.theme.octave_theme.settings;

        // Use per-note colors if enabled, otherwise fall back to simple white/black
        let (base_rgb, pressed_rgb) = if settings.use_per_note_colors {
            (note_color.normal, note_color.pressed)
        } else {
            // Fallback to simple white/black based on key type
            if key.is_sharp {
                ((26, 26, 26), (46, 125, 50))
            } else {
                ((255, 255, 255), (76, 175, 80))
            }
        };

        let base_color = Color::from_rgba(base_rgb.0, base_rgb.1, base_rgb.2, 255);
        let pressed_color = Color::from_rgba(pressed_rgb.0, pressed_rgb.1, pressed_rgb.2, 255);

        let anim = key.animation.value;

        // Enhanced color interpolation for more prominent pressed state
        // When pressed (anim > 0.5), use more aggressive interpolation
        let effective_anim = if anim > 0.5 {
            // For pressed keys, use a more aggressive curve
            // This makes the pressed state more visually distinct
            anim * anim // Square the value for more aggressive transition
        } else {
            anim
        };

        let r = base_color.r + (pressed_color.r - base_color.r) * effective_anim;
        let g = base_color.g + (pressed_color.g - base_color.g) * effective_anim;
        let b = base_color.b + (pressed_color.b - base_color.b) * effective_anim;
        let color = Color { r, g, b, a: 1.0 };

        // 2.5D depth effect - enhanced for pressed keys
        if settings.depth_2d5 > 0.0 {
            let depth = settings.depth_2d5;
            // For pressed keys, reduce the depth effect to simulate being pressed down
            let effective_depth = if anim > 0.5 { depth * 0.3 } else { depth };

            let shadow_color = Color {
                r: base_color.r * 0.5,
                g: base_color.g * 0.5,
                b: base_color.b * 0.5,
                a: 1.0,
            };

            // Draw shadow/depth
            draw_rectangle(
                key.x + effective_depth,
                key.y + effective_depth,
                key.width,
                key.height,
                shadow_color,
            );
        }

        // Draw main key
        draw_rectangle(key.x, key.y, key.width, key.height, color);

        // Draw border - enhanced for pressed keys
        let border_rgb = settings.border_color;
        let border_color = Color::from_rgba(border_rgb.0, border_rgb.1, border_rgb.2, 255);
        let border_width = if anim > 0.5 { 2.0 } else { 1.0 };

        draw_rectangle_lines(
            key.x,
            key.y,
            key.width,
            key.height,
            border_width,
            border_color,
        );

        // Glow effect
        if anim > 0.1 {
            let glow_rgb = note_color.glow_color();
            let glow_intensity = note_color.glow_intensity * anim;

            let glow_color = Color {
                r: glow_rgb.0 as f32 / 255.0,
                g: glow_rgb.1 as f32 / 255.0,
                b: glow_rgb.2 as f32 / 255.0,
                a: glow_intensity * 0.3,
            };

            draw_rectangle(key.x, key.y, key.width, key.height, glow_color);
        }

        // Inner highlight for pressed keys
        if anim > 0.5 {
            let highlight_color = Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15 * anim,
            };

            let highlight_height = key.height * 0.1;
            draw_rectangle(
                key.x + 2.0,
                key.y + 2.0,
                key.width - 4.0,
                highlight_height,
                highlight_color,
            );
        }
    }

    /// Get keyboard layout (for compatibility)
    pub fn layout(&self) -> &KeyboardLayout {
        &self.layout
    }

    /// Get current position
    pub fn position(&self) -> (f32, f32) {
        self.position
    }
}
