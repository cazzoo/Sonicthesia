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

        // Debug: Log rendering state every 60 frames (approx 1 second)
        static mut FRAME_COUNT: u32 = 0;
        let should_log = unsafe {
            FRAME_COUNT += 1;
            FRAME_COUNT % 60 == 0
        };

        let mut visible_notes = 0;
        let mut culled_notes = 0;
        let mut total_notes = 0;
        let mut min_start_time = f32::MAX;
        let mut max_start_time = f32::MIN;

        for note in self.notes.inner().iter() {
            if layout.range.contains(note.note) && note.channel != 9 {
                total_notes += 1;

                let key = &layout.keys[note.note as usize - range_start];

                let color_idx = note.track_color_id % self.config.color_scheme.len();
                let ply_color = &self.config.color_scheme[color_idx];
                let color = Color {
                    r: ply_color.r / 255.0,
                    g: ply_color.g / 255.0,
                    b: ply_color.b / 255.0,
                    a: 1.0,
                };

                let note_start = note.start.as_secs_f32();
                let note_duration = note.duration.as_secs_f32();
                let time_until_start = note_start - self.current_time;
                let y = keyboard_top - (time_until_start * pixels_per_second);
                let height = note_duration * pixels_per_second;

                // Track note time range for debug
                if should_log {
                    min_start_time = min_start_time.min(note_start);
                    max_start_time = max_start_time.max(note_start);
                }

                // Culling: hide notes completely off-screen
                // Notes should be visible while falling from top toward keyboard
                let note_bottom = y + height;
                if note_bottom < 0.0 || y > screen_h {
                    culled_notes += 1;
                    continue;
                }

                visible_notes += 1;

                let x = keyboard_x + key.x() * scale_x;
                let w = key.width() * scale_x - 1.0;

                draw_rectangle(x, y, w, height.max(4.0), color);
            }
        }

        // Debug logging with println! for visibility
        if should_log {
            println!(
                "[WATERFALL] t={:.2}s | {} total, {} vis, {} culled | range: {:.1}-{:.1}s | kb_top={:.0} pps={:.0}",
                self.current_time,
                total_notes,
                visible_notes,
                culled_notes,
                min_start_time,
                max_start_time,
                keyboard_top,
                pixels_per_second
            );

            // Log first 3 notes with positions
            for (i, note) in self.notes.inner().iter().take(3).enumerate() {
                let note_start = note.start.as_secs_f32();
                let note_dur = note.duration.as_secs_f32();
                let time_until = note_start - self.current_time;
                let y_pos = keyboard_top - (time_until * pixels_per_second);
                let h = note_dur * pixels_per_second;
                println!(
                    "  [{}] midi={} start={:.2}s dur={:.2}s => y={:.0} h={:.0} bot={:.0} in={:.2}s",
                    i,
                    note.note,
                    note_start,
                    note_dur,
                    y_pos,
                    h,
                    y_pos + h,
                    time_until
                );
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
