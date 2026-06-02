//! Macroquad-based rendering system for PLY integration
//!
//! This module provides a complete rendering system using macroquad,
//! replacing the WGPU rendering pipeline with PLY's built-in rendering.

use macroquad::prelude::*;
use midi_file::MidiNote;
use neothesia_core::config::Config;
use piano_layout::KeyboardLayout;
use std::rc::Rc;

use super::piano_keyboard::{KeyboardTheme, PianoKeyboardRenderer};

/// Macroquad-based waterfall renderer
pub struct MacroquadWaterfallRenderer {
    notes: Rc<[MidiNote]>,
    layout: KeyboardLayout,
    config: WaterfallConfig,
    current_time: f32,
}

#[derive(Clone)]
struct WaterfallConfig {
    animation_speed: f32,
    colors: Vec<Color>,
}

impl MacroquadWaterfallRenderer {
    pub fn new(notes: Rc<[MidiNote]>, layout: KeyboardLayout, config: &Config) -> Self {
        let theme_name = config.piano_theme_name();
        let theme = KeyboardTheme::get_theme(theme_name).unwrap_or_else(|| {
            KeyboardTheme::modern()
        });
        let pressed_colors = theme.pressed_colors();

        let colors = pressed_colors
            .iter()
            .map(|(r, g, b)| Color {
                r: *r as f32 / 255.0,
                g: *g as f32 / 255.0,
                b: *b as f32 / 255.0,
                a: 1.0,
            })
            .collect();

        Self {
            notes,
            layout,
            config: WaterfallConfig {
                animation_speed: config.animation_speed(),
                colors,
            },
            current_time: 0.0,
        }
    }

    pub fn update(&mut self, time: f32) {
        self.current_time = time;
    }

    pub fn render(&self) {
        let range_start = self.layout.range.start() as usize;

        for note in self.notes.iter() {
            if self.layout.range.contains(note.note) && note.channel != 9 {
                let key = &self.layout.keys[note.note as usize - range_start];

                let color_idx = (note.note % 12) as usize % self.config.colors.len();
                let color = self.config.colors[color_idx];

                let x = key.x();
                let y = note.start.as_secs_f32();
                let w = key.width() - 1.0;
                let h = if note.duration.as_secs_f32() >= 0.1 {
                    note.duration.as_secs_f32()
                } else {
                    0.1
                };

                draw_rectangle(x, y, w, h - 0.01, color);
            }
        }

        log::trace!(
            "🎨 PLY RENDERING: Drew {} waterfall notes",
            self.notes.len()
        );
    }
}

/// Macroquad-based keyboard renderer
pub struct MacroquadKeyboardRenderer {
    layout: KeyboardLayout,
    keys: Vec<KeyState>,
}

struct KeyState {
    note: u8,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
    is_sharp: bool,
}

impl MacroquadKeyboardRenderer {
    pub fn new(layout: KeyboardLayout, config: &Config) -> Self {
        let keys = layout
            .keys
            .iter()
            .map(|key| {
                let color = if key.kind().is_sharp() {
                    Color::from_hex(0x1a1a1a)
                } else {
                    Color::from_hex(0xffffff)
                };

                KeyState {
                    note: key.note_id(),
                    x: key.x(),
                    y: layout.height,
                    width: key.width(),
                    height: 120.0, // Standard key height
                    color,
                    is_sharp: key.kind().is_sharp(),
                }
            })
            .collect();

        Self { layout, keys }
    }

    pub fn render(&self) {
        for key in &self.keys {
            draw_rectangle(key.x, key.y, key.width, key.height, key.color);
            draw_rectangle_lines(
                key.x,
                key.y,
                key.width,
                key.height,
                1.0,
                Color::from_hex(0x000000),
            );
        }

        log::trace!("🎨 PLY RENDERING: Drew {} keyboard keys", self.keys.len());
    }
}

/// Macroquad-based guidelines renderer
pub struct MacroquadGuidelineRenderer {
    layout: KeyboardLayout,
    vertical: bool,
    horizontal: bool,
    measures: std::sync::Arc<[std::time::Duration]>,
}

impl MacroquadGuidelineRenderer {
    pub fn new(
        layout: KeyboardLayout,
        vertical: bool,
        horizontal: bool,
        measures: std::sync::Arc<[std::time::Duration]>,
    ) -> Self {
        Self {
            layout,
            vertical,
            horizontal,
            measures,
        }
    }

    pub fn render(&self, time: f32, animation_speed: f32) {
        let line_color = Color::new(0.3, 0.3, 0.3, 0.5);

        // Vertical guidelines
        if self.vertical {
            for key in self.layout.keys.iter() {
                draw_line(
                    key.x() + key.width() / 2.0,
                    0.0,
                    key.x() + key.width() / 2.0,
                    10000.0,
                    1.0,
                    line_color,
                );
            }
        }

        // Horizontal guidelines (measures)
        if self.horizontal {
            for measure_time in self.measures.iter() {
                let y = measure_time.as_secs_f32() * animation_speed;
                draw_line(0.0, y, 10000.0, y, 1.0, line_color);
            }
        }

        log::trace!("🎨 PLY RENDERING: Drew guidelines");
    }
}

/// Main PLY rendering coordinator using macroquad
pub struct PlyMacroquadRenderer {
    waterfall: Option<MacroquadWaterfallRenderer>,
    keyboard: Option<MacroquadKeyboardRenderer>,
    enhanced_keyboard: Option<PianoKeyboardRenderer>,
    guidelines: Option<MacroquadGuidelineRenderer>,
}

impl PlyMacroquadRenderer {
    pub fn new() -> Self {
        Self {
            waterfall: None,
            keyboard: None,
            enhanced_keyboard: None,
            guidelines: None,
        }
    }

    pub fn set_waterfall(&mut self, renderer: MacroquadWaterfallRenderer) {
        self.waterfall = Some(renderer);
    }

    pub fn set_keyboard(&mut self, renderer: MacroquadKeyboardRenderer) {
        self.keyboard = Some(renderer);
    }

    pub fn set_enhanced_keyboard(&mut self, renderer: PianoKeyboardRenderer) {
        self.enhanced_keyboard = Some(renderer);
    }

    pub fn set_guidelines(&mut self, renderer: MacroquadGuidelineRenderer) {
        self.guidelines = Some(renderer);
    }

    pub fn render(&mut self, time: f32, animation_speed: f32, dt: f32) {
        clear_background(BLACK);

        if let Some(waterfall) = &self.waterfall {
            waterfall.render();
        }

        if let Some(guidelines) = &self.guidelines {
            guidelines.render(time, animation_speed);
        }

        if let Some(enhanced_keyboard) = &mut self.enhanced_keyboard {
            enhanced_keyboard.update(dt);
            enhanced_keyboard.render();
        } else if let Some(keyboard) = &self.keyboard {
            keyboard.render();
        }

        log::info!("🎨 PLY RENDERING ACTIVE: Frame rendered using macroquad");
    }

    pub fn enhanced_keyboard_mut(&mut self) -> Option<&mut PianoKeyboardRenderer> {
        self.enhanced_keyboard.as_mut()
    }

    pub fn enhanced_keyboard(&self) -> Option<&PianoKeyboardRenderer> {
        self.enhanced_keyboard.as_ref()
    }
}
