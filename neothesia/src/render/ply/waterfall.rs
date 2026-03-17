//! PLY-based waterfall renderer for Neothesia
//! 
//! This module provides a bridge between Neothesia's existing waterfall rendering
//! and the PLY engine, allowing for gradual migration.

use midi_file::{MidiNote, MidiTrack};
use std::rc::Rc;
use neothesia_core::{config, render};
use neothesia_core::render::waterfall::{TrackChannelConfig, NoteList};
use piano_layout::KeyboardLayout;

/// PLY-based waterfall renderer for Neothesia
pub struct PlyWaterfallRenderer {
    /// Waterfall notes to render
    notes: NoteList,
    /// Configuration for rendering
    config: RenderConfig,
    /// Keyboard layout for positioning
    layout: Option<KeyboardLayout>,
    /// Current time for animation
    current_time: f32,
    /// Whether the renderer is initialized
    initialized: bool,
}

impl PlyWaterfallRenderer {
    /// Create a new PLY waterfall renderer
    pub fn new() -> Self {
        Self {
            notes: NoteList::empty(),
            config: RenderConfig::default(),
            layout: None,
            current_time: 0.0,
            initialized: false,
        }
    }

    /// Initialize the renderer with MIDI data and configuration
    pub fn initialize(
        &mut self,
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        track_channel_configs: &[TrackChannelConfig],
        config: &neothesia_core::config::Config,
        layout: &KeyboardLayout,
    ) {
        // Create note list from MIDI data
        self.notes = NoteList::new(tracks, hidden_tracks, track_channel_configs);
        
        // Configure renderer
        self.config = RenderConfig::from_neothesia_config(config, layout);
        self.layout = Some(layout.clone());
        self.initialized = true;
    }

    /// Update the renderer with new time
    pub fn update(&mut self, time: f32) {
        if !self.initialized {
            return;
        }
        self.current_time = time;
    }
    
    /// Get the notes list
    pub fn notes(&self) -> &NoteList {
        &self.notes
    }
    
    /// Get current time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }
    
    /// Get animation speed
    pub fn animation_speed(&self) -> f32 {
        self.config.animation_speed
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get layout
    pub fn layout(&self) -> Option<&KeyboardLayout> {
        self.layout.as_ref()
    }

    /// Render the waterfall using PLY's built-in rendering system
    /// This uses macroquad for actual rendering
    pub fn render_ply(&mut self) {
        if !self.initialized {
            return;
        }

        use macroquad::prelude::*;

        let layout = match &self.layout {
            Some(l) => l,
            None => return,
        };

        let range_start = layout.range.start() as usize;

        // Render each note using macroquad drawing functions
        for note in self.notes.inner().iter() {
            if layout.range.contains(note.note) && note.channel != 9 {
                let key = &layout.keys[note.note as usize - range_start];

                // Get color from config
                let color_idx = note.track_color_id % self.config.color_scheme.len();
                let ply_color = &self.config.color_scheme[color_idx];
                let color = Color {
                    r: ply_color.r / 255.0,
                    g: ply_color.g / 255.0,
                    b: ply_color.b / 255.0,
                    a: ply_color.a / 255.0,
                };

                // Calculate note position and size
                let x = key.x();
                let y = note.start.as_secs_f32();
                let w = key.width() - 1.0;
                let h = if note.duration.as_secs_f32() >= 0.1 {
                    note.duration.as_secs_f32()
                } else {
                    0.1
                };

                // Draw the note using macroquad
                draw_rectangle(x, y, w, h - 0.01, color);
            }
        }

        log::debug!("🎨 PLY Waterfall: Rendered {} notes using macroquad", self.notes.inner().len());
    }
}

/// Configuration for PLY waterfall rendering
#[derive(Clone, Debug)]
struct RenderConfig {
    /// Animation speed
    animation_speed: f32,
    /// Color scheme for tracks
    color_scheme: Vec<ply_engine::color::Color>,
    /// Background color
    background_color: ply_engine::color::Color,
}

impl RenderConfig {
    /// Create default configuration
    fn default() -> Self {
        Self {
            animation_speed: 1.0,
            color_scheme: vec![
                ply_engine::color::Color::rgb(255.0, 0.0, 0.0),   // RED
                ply_engine::color::Color::rgb(0.0, 0.0, 255.0),   // BLUE
                ply_engine::color::Color::rgb(0.0, 255.0, 0.0),   // GREEN
                ply_engine::color::Color::rgb(255.0, 255.0, 0.0), // YELLOW
            ],
            background_color: ply_engine::color::Color::rgb(0.0, 0.0, 0.0), // BLACK
        }
    }

    /// Create configuration from Neothesia config
    fn from_neothesia_config(
        config: &neothesia_core::config::Config,
        layout: &piano_layout::KeyboardLayout,
    ) -> Self {
        Self {
            animation_speed: config.animation_speed(),
            color_scheme: config.color_schema().iter()
                .map(|c| ply_engine::color::Color::u_rgba(c.base.0, c.base.1, c.base.2, 255))
                .collect(),
            background_color: {
                let bg = config.background_color();
                ply_engine::color::Color::u_rgba(bg.0, bg.1, bg.2, 255)
            },
        }
    }
}
