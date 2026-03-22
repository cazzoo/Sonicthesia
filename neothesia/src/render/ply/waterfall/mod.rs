use midi_file::{MidiNote, MidiTrack};
use neothesia_core::config;
use neothesia_core::waterfall::{NoteList, TrackChannelConfig};
use piano_layout::KeyboardLayout;
use std::rc::Rc;
use std::time::Duration;

pub mod renderer_2d;

pub use renderer_2d::Waterfall2D;

/// Represents a note that should be rendered in the waterfall
#[derive(Clone, Debug)]
pub struct WaterfallNote {
    pub note: u8,
    pub start: f32,
    pub end: f32,
    pub velocity: u8,
    pub channel: u8,
    pub track_id: usize,
    pub track_color_id: usize,
}

impl From<&MidiNote> for WaterfallNote {
    fn from(n: &MidiNote) -> Self {
        Self {
            note: n.note,
            start: n.start.as_secs_f32(),
            end: n.end.as_secs_f32(),
            velocity: n.velocity,
            channel: n.channel,
            track_id: n.track_id,
            track_color_id: n.track_color_id,
        }
    }
}

/// Visual state of a note in the waterfall
#[derive(Clone, Debug)]
pub struct NoteVisual {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8, u8),
    pub is_sharp: bool,
    pub progress: f32,
}

/// Configuration for waterfall rendering
#[derive(Clone, Debug)]
pub struct WaterfallConfig {
    pub animation_speed: f32,
    pub note_spacing: f32,
    pub colors: Vec<(u8, u8, u8)>,
}

impl Default for WaterfallConfig {
    fn default() -> Self {
        Self {
            animation_speed: 1.0,
            note_spacing: 2.0,
            colors: vec![(255, 80, 80), (80, 80, 255), (80, 255, 80), (255, 255, 80)],
        }
    }
}

/// Generic trait for waterfall rendering techniques
pub trait WaterfallRenderer {
    fn update(&mut self, time: f32, layout: &KeyboardLayout);
    fn render(&self);
    fn get_active_notes(&self) -> Vec<&WaterfallNote>;
    fn should_be_pressed(&self, note: u8) -> bool;
    fn get_pressed_keys(&self) -> Vec<u8>;
}

/// Main waterfall manager that coordinates rendering
pub struct Waterfall {
    notes: Vec<WaterfallNote>,
    config: WaterfallConfig,
    layout: KeyboardLayout,
    current_time: f32,
    renderer: Box<dyn WaterfallRenderer>,
}

impl Waterfall {
    pub fn new(notes: &[MidiNote], config: WaterfallConfig, layout: KeyboardLayout) -> Self {
        let waterfall_notes: Vec<WaterfallNote> = notes.iter().map(|n| n.into()).collect();
        let renderer = Box::new(Waterfall2D::new(&waterfall_notes, &config, &layout));

        Self {
            notes: waterfall_notes,
            config,
            layout,
            current_time: 0.0,
            renderer,
        }
    }

    pub fn update(&mut self, time: f32) {
        self.current_time = time;
        self.renderer.update(time, &self.layout);
    }

    pub fn render(&self) {
        self.renderer.render();
    }

    pub fn should_be_pressed(&self, note: u8) -> bool {
        self.renderer.should_be_pressed(note)
    }

    pub fn get_pressed_keys(&self) -> Vec<u8> {
        self.renderer.get_pressed_keys()
    }
}

pub struct PlyWaterfallRenderer {
    notes: NoteList,
    config: RenderConfig,
    layout: Option<KeyboardLayout>,
    current_time: f32,
    initialized: bool,
}

impl PlyWaterfallRenderer {
    pub fn new() -> Self {
        Self {
            notes: NoteList::empty(),
            config: RenderConfig::default(),
            layout: None,
            current_time: 0.0,
            initialized: false,
        }
    }

    pub fn initialize(
        &mut self,
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        track_channel_configs: &[TrackChannelConfig],
        config: &neothesia_core::config::Config,
        layout: &KeyboardLayout,
    ) {
        log::info!(
            "[WATERFALL INIT] Initializing with {} tracks, range={:?}",
            tracks.len(),
            layout.range
        );

        self.notes = NoteList::new(tracks, hidden_tracks, track_channel_configs);

        log::info!(
            "[WATERFALL INIT] Loaded {} notes after filtering",
            self.notes.len()
        );

        // Log first few notes for debugging
        for (i, note) in self.notes.inner().iter().take(5).enumerate() {
            log::info!(
                "[WATERFALL INIT] Note {}: midi={}, start={:?}, duration={:?}, channel={}, track={}",
                i, note.note, note.start, note.duration, note.channel, note.track_id
            );
        }

        self.config = RenderConfig::from_neothesia_config(config, layout);
        self.layout = Some(layout.clone());
        self.initialized = true;

        log::info!(
            "[WATERFALL INIT] Initialized with animation_speed={}",
            self.config.animation_speed
        );
    }

    pub fn update(&mut self, time: f32) {
        if !self.initialized {
            return;
        }
        self.current_time = time;
    }

    pub fn notes(&self) -> &NoteList {
        &self.notes
    }

    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    pub fn animation_speed(&self) -> f32 {
        self.config.animation_speed
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn layout(&self) -> Option<&KeyboardLayout> {
        self.layout.as_ref()
    }

    /// Get notes that should be playing right now (current_time is between start and end)
    pub fn get_active_notes(&self) -> Vec<(u8, u8)> {
        if !self.initialized {
            return Vec::new();
        }

        self.notes
            .inner()
            .iter()
            .filter(|note| {
                let start = note.start.as_secs_f32();
                let end = start + note.duration.as_secs_f32();
                self.current_time >= start && self.current_time < end
            })
            .map(|note| (note.note, note.velocity))
            .collect()
    }

    /// Get notes that should be highlighted (about to be played within threshold)
    pub fn get_upcoming_notes(&self, threshold: f32) -> Vec<u8> {
        if !self.initialized {
            return Vec::new();
        }

        self.notes
            .inner()
            .iter()
            .filter(|note| {
                let start = note.start.as_secs_f32();
                let time_until = start - self.current_time;
                time_until > 0.0 && time_until <= threshold
            })
            .map(|note| note.note)
            .collect()
    }

    pub fn render_ply(&self) {
        if !self.initialized {
            return;
        }

        use macroquad::prelude::*;

        let layout = match &self.layout {
            Some(l) => l,
            None => return,
        };

        let screen_w = screen_width();
        let screen_h = screen_height();
        let keyboard_height = screen_h * 0.2;
        let keyboard_top = screen_h - keyboard_height - 20.0;
        let keyboard_width = screen_w * 0.95;
        let keyboard_x = (screen_w - keyboard_width) / 2.0;
        let pixels_per_second = self.config.animation_speed;
        let scale_x = keyboard_width / layout.width;

        let range_start = layout.range.start() as usize;

        // Color palettes from design spec
        // Right Hand (RH): Warm colors - Magenta, Purple, Pink
        let rh_colors = [
            Color::from_rgba(255, 0, 255, 255),   // Magenta #ff00ff
            Color::from_rgba(168, 85, 247, 255),  // Purple #a855f7
            Color::from_rgba(244, 114, 182, 255), // Pink #f472b6
        ];
        // Left Hand (LH): Cool colors - Blue, Cyan, Teal
        let lh_colors = [
            Color::from_rgba(37, 99, 235, 255),  // Blue #2563eb
            Color::from_rgba(34, 211, 238, 255), // Cyan #22d3ee
            Color::from_rgba(45, 212, 191, 255), // Teal #2dd4bf
        ];

        for note in self.notes.inner().iter() {
            if layout.range.contains(note.note) && note.channel != 9 {
                let key = &layout.keys[note.note as usize - range_start];

                // Determine left/right hand based on MIDI channel
                // Channel 0 = Right Hand, Channel 1 = Left Hand
                let is_right_hand = note.channel == 0;
                let color_palette = if is_right_hand {
                    &rh_colors
                } else {
                    &lh_colors
                };
                let color_idx = note.track_color_id % color_palette.len();
                let color = color_palette[color_idx];

                let note_start = note.start.as_secs_f32();
                let note_duration = note.duration.as_secs_f32();
                let time_until_start = note_start - self.current_time;
                let y = keyboard_top - (time_until_start * pixels_per_second);
                let height = note_duration * pixels_per_second;

                let x = keyboard_x + key.x() * scale_x;
                let w = key.width() * scale_x - 1.0;

                // Clip notes at keyboard boundary - don't show below keyboard
                let note_bottom = y + height;
                let clipped_bottom = note_bottom.min(keyboard_top);
                let clipped_height = (clipped_bottom - y).max(0.0);

                // Skip if note is completely off screen
                if clipped_height <= 0.0 || y > keyboard_top || y + clipped_height < 0.0 {
                    continue;
                }

                // Draw glow effect (larger, semi-transparent rectangle behind)
                let glow_color =
                    Color::from_rgba((color.r as u8), (color.g as u8), (color.b as u8), 60);
                draw_rectangle(x - 2.0, y - 2.0, w + 4.0, clipped_height + 4.0, glow_color);

                // Draw main note with rounded appearance (simulated via multiple rects)
                // Main body
                draw_rectangle(x, y, w, clipped_height.max(4.0), color);

                // Top highlight for depth
                let highlight_height = (clipped_height * 0.15).min(8.0);
                let highlight_color = Color::from_rgba(
                    ((color.r as f32) * 1.3).min(255.0) as u8,
                    ((color.g as f32) * 1.3).min(255.0) as u8,
                    ((color.b as f32) * 1.3).min(255.0) as u8,
                    180,
                );
                draw_rectangle(x, y, w, highlight_height, highlight_color);
            }
        }
    }
}

struct RenderConfig {
    animation_speed: f32,
    color_scheme: Vec<ply_engine::color::Color>,
    background_color: ply_engine::color::Color,
}

impl RenderConfig {
    fn default() -> Self {
        Self {
            animation_speed: 1.0,
            color_scheme: vec![
                ply_engine::color::Color::rgb(255.0, 0.0, 0.0),
                ply_engine::color::Color::rgb(0.0, 0.0, 255.0),
                ply_engine::color::Color::rgb(0.0, 255.0, 0.0),
                ply_engine::color::Color::rgb(255.0, 255.0, 0.0),
            ],
            background_color: ply_engine::color::Color::rgb(0.0, 0.0, 0.0),
        }
    }

    fn from_neothesia_config(
        config: &neothesia_core::config::Config,
        layout: &piano_layout::KeyboardLayout,
    ) -> Self {
        Self {
            animation_speed: config.animation_speed(),
            color_scheme: config
                .color_schema()
                .iter()
                .map(|c| ply_engine::color::Color::u_rgba(c.base.0, c.base.1, c.base.2, 255))
                .collect(),
            background_color: {
                let bg = config.background_color();
                ply_engine::color::Color::u_rgba(bg.0, bg.1, bg.2, 255)
            },
        }
    }
}
