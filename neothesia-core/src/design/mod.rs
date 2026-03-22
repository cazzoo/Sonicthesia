//! Design tokens for the Sonic Obsidian design system
//!
//! This module provides all design tokens (colors, typography, spacing, effects)
//! used throughout the Neothesia UI, following the "Sonic Obsidian" design system.

/// Color palette for the Sonic Obsidian theme
pub mod colors {
    /// Background color (deep black void)
    pub const BACKGROUND: (u8, u8, u8) = (14, 14, 19);

    /// Surface color (same as background)
    pub const SURFACE: (u8, u8, u8) = (14, 14, 19);

    /// Surface container lowest
    pub const SURFACE_CONTAINER_LOWEST: (u8, u8, u8) = (0, 0, 0);

    /// Surface container low
    pub const SURFACE_CONTAINER_LOW: (u8, u8, u8) = (19, 19, 24);

    /// Surface container (mid-level)
    pub const SURFACE_CONTAINER: (u8, u8, u8) = (25, 25, 31);

    /// Surface container high
    pub const SURFACE_CONTAINER_HIGH: (u8, u8, u8) = (31, 31, 38);

    /// Surface container highest (closest to user)
    pub const SURFACE_CONTAINER_HIGHEST: (u8, u8, u8) = (37, 37, 44);

    /// Primary neon purple
    pub const PRIMARY: (u8, u8, u8) = (219, 144, 255);

    /// Primary dimmed
    pub const PRIMARY_DIM: (u8, u8, u8) = (210, 119, 255);

    /// Primary container
    pub const PRIMARY_CONTAINER: (u8, u8, u8) = (211, 123, 255);

    /// Primary fixed
    pub const PRIMARY_FIXED: (u8, u8, u8) = (211, 123, 255);

    /// Primary fixed dim
    pub const PRIMARY_FIXED_DIM: (u8, u8, u8) = (203, 102, 254);

    /// Secondary neon blue
    pub const SECONDARY: (u8, u8, u8) = (95, 158, 255);

    /// Secondary dimmed
    pub const SECONDARY_DIM: (u8, u8, u8) = (0, 115, 224);

    /// Secondary container
    pub const SECONDARY_CONTAINER: (u8, u8, u8) = (0, 93, 184);

    /// Tertiary neon pink
    pub const TERTIARY: (u8, u8, u8) = (255, 110, 128);

    /// Tertiary dimmed
    pub const TERTIARY_DIM: (u8, u8, u8) = (226, 29, 77);

    /// Tertiary container
    pub const TERTIARY_CONTAINER: (u8, u8, u8) = (252, 52, 93);

    /// On surface text (near-white, NOT pure white)
    pub const ON_SURFACE: (u8, u8, u8) = (248, 245, 253);

    /// On surface variant (muted text)
    pub const ON_SURFACE_VARIANT: (u8, u8, u8) = (172, 170, 177);

    /// On background
    pub const ON_BACKGROUND: (u8, u8, u8) = (248, 245, 253);

    /// Error neon red
    pub const ERROR: (u8, u8, u8) = (255, 110, 132);

    /// Error dimmed
    pub const ERROR_DIM: (u8, u8, u8) = (215, 51, 87);

    /// Error container
    pub const ERROR_CONTAINER: (u8, u8, u8) = (167, 1, 56);

    /// Outline (ghost borders)
    pub const OUTLINE: (u8, u8, u8) = (118, 116, 123);

    /// Outline variant (for ghost borders)
    pub const OUTLINE_VARIANT: (u8, u8, u8) = (72, 71, 77);

    /// Black (for primary button text)
    pub const BLACK: (u8, u8, u8) = (0, 0, 0);

    /// Convert RGB tuple to normalized floats (0.0-1.0)
    pub fn to_normalized(color: (u8, u8, u8)) -> (f32, f32, f32) {
        (
            color.0 as f32 / 255.0,
            color.1 as f32 / 255.0,
            color.2 as f32 / 255.0,
        )
    }

    /// Convert RGB tuple to hex string
    pub fn to_hex(color: (u8, u8, u8)) -> String {
        format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2)
    }

    /// Parse hex string to RGB tuple
    pub fn from_hex(hex: &str) -> Option<(u8, u8, u8)> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    }
}

/// Typography constants
pub mod typography {
    /// Display font (headlines, posters)
    pub const FONT_DISPLAY: &str = "Space Grotesk";

    /// Body font (paragraphs, labels)
    pub const FONT_BODY: &str = "Inter";

    /// Label font (buttons, small text)
    pub const FONT_LABEL: &str = "Inter";

    /// Monospace font (technical readouts, hex values)
    pub const FONT_MONOSPACE: &str = "JetBrains Mono";

    /// Display large size (48px)
    pub const DISPLAY_LG: u16 = 48;

    /// Display medium size (36px)
    pub const DISPLAY_MD: u16 = 36;

    /// Display small size (24px)
    pub const DISPLAY_SM: u16 = 24;

    /// Headline large size (32px)
    pub const HEADLINE_LG: u16 = 32;

    /// Headline medium size (24px)
    pub const HEADLINE_MD: u16 = 24;

    /// Headline small size (20px)
    pub const HEADLINE_SM: u16 = 20;

    /// Body large size (18px)
    pub const BODY_LG: u16 = 18;

    /// Body medium size (16px)
    pub const BODY_MD: u16 = 16;

    /// Body small size (14px)
    pub const BODY_SM: u16 = 14;

    /// Label large size (16px)
    pub const LABEL_LG: u16 = 16;

    /// Label medium size (14px)
    pub const LABEL_MD: u16 = 14;

    /// Label small size (12px)
    pub const LABEL_SM: u16 = 12;

    /// Label extra small size (10px)
    pub const LABEL_XS: u16 = 10;
}

/// Spacing constants (4px base unit)
pub mod spacing {
    /// Extra small spacing (4px)
    pub const XS: f32 = 4.0;

    /// Small spacing (8px)
    pub const SM: f32 = 8.0;

    /// Medium spacing (12px)
    pub const MD: f32 = 12.0;

    /// Large spacing (16px)
    pub const LG: f32 = 16.0;

    /// Extra large spacing (24px)
    pub const XL: f32 = 24.0;

    /// 2x extra large spacing (32px)
    pub const XXL: f32 = 32.0;

    /// 3x extra large spacing (48px)
    pub const XXXL: f32 = 48.0;

    /// 4x extra large spacing (64px)
    pub const XXXXL: f32 = 64.0;

    /// Standard card padding
    pub const CARD_PADDING: f32 = 24.0;

    /// Section spacing
    pub const SECTION_SPACING: f32 = 32.0;

    /// Component gap
    pub const COMPONENT_GAP: f32 = 16.0;

    /// Small component gap
    pub const COMPONENT_GAP_SM: f32 = 8.0;
}

/// Component size constants
pub mod sizes {
    /// Standard button height
    pub const BUTTON_HEIGHT: f32 = 48.0;

    /// Icon button size
    pub const ICON_BUTTON_SIZE: f32 = 48.0;

    /// Input field height
    pub const INPUT_HEIGHT: f32 = 48.0;

    /// Slider track height
    pub const SLIDER_TRACK_HEIGHT: f32 = 4.0;

    /// Slider thumb size
    pub const SLIDER_THUMB_SIZE: f32 = 16.0;

    /// Toggle track width
    pub const TOGGLE_WIDTH: f32 = 40.0;

    /// Toggle track height
    pub const TOGGLE_HEIGHT: f32 = 20.0;

    /// Toggle thumb size
    pub const TOGGLE_THUMB_SIZE: f32 = 16.0;

    /// Color swatch size
    pub const COLOR_SWATCH_SIZE: f32 = 48.0;

    /// Status chip height
    pub const CHIP_HEIGHT: f32 = 24.0;

    /// List item height
    pub const LIST_ITEM_HEIGHT: f32 = 64.0;

    /// Navigation bar height
    pub const NAV_BAR_HEIGHT: f32 = 64.0;

    /// Sidebar width
    pub const SIDEBAR_WIDTH: f32 = 256.0;

    /// Progress bar height
    pub const PROGRESS_BAR_HEIGHT: f32 = 4.0;

    /// Theme card aspect ratio (width:height)
    pub const THEME_CARD_ASPECT_RATIO: f32 = 4.0 / 3.0;
}

/// Border radius constants
pub mod radius {
    /// Small radius (4px)
    pub const SM: f32 = 4.0;

    /// Medium radius (8px)
    pub const MD: f32 = 8.0;

    /// Large radius (12px)
    pub const LG: f32 = 12.0;

    /// Full rounded (9999px for pills)
    pub const FULL: f32 = 9999.0;
}

/// Effect constants
pub mod effects {
    /// Glass panel opacity
    pub const GLASS_OPACITY: f32 = 0.6;

    /// Glass panel blur radius (in pixels)
    pub const GLASS_BLUR: f32 = 20.0;

    /// Ghost border opacity
    pub const GHOST_BORDER_OPACITY: f32 = 0.2;

    /// Ghost border width
    pub const GHOST_BORDER_WIDTH: f32 = 1.0;

    /// Glow opacity (ambient)
    pub const GLOW_OPACITY: f32 = 0.06;

    /// Glow opacity (active)
    pub const GLOW_OPACITY_ACTIVE: f32 = 0.15;

    /// Glow blur radius
    pub const GLOW_BLUR: f32 = 32.0;

    /// Hover scale factor
    pub const HOVER_SCALE: f32 = 1.02;

    /// Press scale factor
    pub const PRESS_SCALE: f32 = 0.95;

    /// Transition duration (milliseconds)
    pub const TRANSITION_MS: u32 = 200;

    /// Complex transition duration (milliseconds)
    pub const TRANSITION_COMPLEX_MS: u32 = 300;
}

/// Theme preset definitions
pub mod themes {
    use super::colors;

    /// Theme preset definition
    pub struct ThemePreset {
        pub id: &'static str,
        pub name: &'static str,
        pub description: &'static str,
        pub primary: (u8, u8, u8),
        pub secondary: (u8, u8, u8),
        pub tertiary: (u8, u8, u8),
        pub background: (u8, u8, u8),
    }

    /// Sonic Obsidian theme (default)
    pub const SONIC_OBSIDIAN: ThemePreset = ThemePreset {
        id: "sonic_obsidian",
        name: "Sonic Obsidian",
        description: "The original neon dark theme",
        primary: colors::PRIMARY,
        secondary: colors::SECONDARY,
        tertiary: colors::TERTIARY,
        background: colors::BACKGROUND,
    };

    /// Classic Light theme
    pub const CLASSIC_LIGHT: ThemePreset = ThemePreset {
        id: "classic_light",
        name: "Classic Light",
        description: "Clean light theme for daytime use",
        primary: (103, 58, 183),
        secondary: (33, 150, 243),
        tertiary: (255, 87, 34),
        background: (250, 250, 250),
    };

    /// Dark Pro theme
    pub const DARK_PRO: ThemePreset = ThemePreset {
        id: "dark_pro",
        name: "Dark Pro",
        description: "Professional dark theme with less saturation",
        primary: (149, 117, 205),
        secondary: (100, 181, 246),
        tertiary: (255, 138, 101),
        background: (18, 18, 24),
    };

    /// High Contrast theme
    pub const HIGH_CONTRAST: ThemePreset = ThemePreset {
        id: "high_contrast",
        name: "High Contrast",
        description: "Accessibility-focused theme",
        primary: (255, 255, 255),
        secondary: (0, 255, 255),
        tertiary: (255, 255, 0),
        background: (0, 0, 0),
    };

    /// All system themes
    pub const ALL_THEMES: &[&ThemePreset] =
        &[&SONIC_OBSIDIAN, &CLASSIC_LIGHT, &DARK_PRO, &HIGH_CONTRAST];
}

/// Re-export commonly used items
pub use colors::{BACKGROUND, ON_SURFACE, ON_SURFACE_VARIANT, PRIMARY, SECONDARY, TERTIARY};
pub use colors::{SURFACE_CONTAINER, SURFACE_CONTAINER_HIGH, SURFACE_CONTAINER_HIGHEST};
pub use spacing::{LG, MD, SM, XL, XS};
