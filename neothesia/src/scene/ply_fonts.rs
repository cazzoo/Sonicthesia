//! Shared font management for PLY scenes
//!
//! Provides Space Grotesk (headlines) and Inter (body) fonts
//! as specified by the Sonic Obsidian design system.

use macroquad::prelude::Font;
use std::sync::OnceLock;

/// Global font storage
static HEADLINE_FONT: OnceLock<Option<Font>> = OnceLock::new();
static BODY_FONT: OnceLock<Option<Font>> = OnceLock::new();

/// Initialize fonts (call once from async context)
pub async fn init_fonts() {
    HEADLINE_FONT
        .set(
            macroquad::prelude::load_ttf_font("assets/fonts/SpaceGrotesk.ttf")
                .await
                .ok(),
        )
        .ok();

    BODY_FONT
        .set(
            macroquad::prelude::load_ttf_font("assets/fonts/Inter-Regular.ttf")
                .await
                .ok(),
        )
        .ok();
}

/// Get headline font (Space Grotesk)
pub fn headline_font() -> Option<&'static Font> {
    HEADLINE_FONT.get().and_then(|f| f.as_ref())
}

/// Get body font (Inter)
pub fn body_font() -> Option<&'static Font> {
    BODY_FONT.get().and_then(|f| f.as_ref())
}

/// Draw text with headline font (Space Grotesk)
pub fn draw_headline(text: &str, x: f32, y: f32, size: f32, color: macroquad::prelude::Color) {
    use macroquad::prelude::*;

    if let Some(font) = headline_font() {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(text, x, y, size, color);
    }
}

/// Draw text with body font (Inter)
pub fn draw_body(text: &str, x: f32, y: f32, size: f32, color: macroquad::prelude::Color) {
    use macroquad::prelude::*;

    if let Some(font) = body_font() {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(text, x, y, size, color);
    }
}

/// Draw text with centered alignment using headline font
pub fn draw_headline_centered(
    text: &str,
    center_x: f32,
    y: f32,
    size: f32,
    color: macroquad::prelude::Color,
) {
    use macroquad::prelude::*;

    let text_size = measure_text(text, headline_font(), size as u16, 1.0);
    draw_headline(text, center_x - text_size.width / 2.0, y, size, color);
}

/// Draw text with centered alignment using body font
pub fn draw_body_centered(
    text: &str,
    center_x: f32,
    y: f32,
    size: f32,
    color: macroquad::prelude::Color,
) {
    use macroquad::prelude::*;

    let text_size = measure_text(text, body_font(), size as u16, 1.0);
    draw_body(text, center_x - text_size.width / 2.0, y, size, color);
}

/// Draw small uppercase label text (for UI labels like "CURRENT SCORE", "TEMPO", etc.)
pub fn draw_label(text: &str, x: f32, y: f32, size: f32, color: macroquad::prelude::Color) {
    use macroquad::prelude::*;

    let uppercase_text = text.to_uppercase();
    if let Some(font) = body_font() {
        draw_text_ex(
            &uppercase_text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(&uppercase_text, x, y, size, color);
    }
}

/// Draw monospace text (for MIDI events, timestamps, technical data)
pub fn draw_mono(text: &str, x: f32, y: f32, size: f32, color: macroquad::prelude::Color) {
    use macroquad::prelude::*;

    // Use body font with monospace feature for consistent alignment
    if let Some(font) = body_font() {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(text, x, y, size, color);
    }
}
