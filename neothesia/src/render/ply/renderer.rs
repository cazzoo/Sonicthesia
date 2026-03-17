//! PLY-based rendering coordinator for Neothesia
//! 
//! This module provides a bridge between Neothesia's existing rendering architecture
//! and the PLY engine, allowing for gradual migration.

use ply_engine::prelude::*;
use ply_engine::math::Dimensions;
use neothesia_core::config::Config;
use piano_layout::KeyboardLayout;
use std::sync::Arc;
use std::time::Duration;

use super::{PlyWaterfallRenderer, PlyKeyboardRenderer, PlyGuidelineRenderer, PlyNoteLabelsRenderer};
use neothesia_core::render::waterfall::TrackChannelConfig;
use midi_file::MidiTrack;

/// Main PLY rendering coordinator for Neothesia
/// 
/// This coordinator manages all PLY-based rendering components and provides
/// a unified interface that matches Neothesia's existing architecture.
pub struct PlyRendererCoordinator {
    /// Waterfall renderer
    waterfall: Option<PlyWaterfallRenderer>,
    /// Keyboard renderer
    keyboard: Option<PlyKeyboardRenderer>,
    /// Guidelines renderer
    guidelines: Option<PlyGuidelineRenderer>,
    /// Note labels renderer
    note_labels: Option<PlyNoteLabelsRenderer>,
    /// Whether the coordinator is initialized
    initialized: bool,
}

impl PlyRendererCoordinator {
    /// Create a new PLY renderer coordinator
    pub fn new() -> Self {
        Self {
            waterfall: None,
            keyboard: None,
            guidelines: None,
            note_labels: None,
            initialized: false,
        }
    }

    /// Initialize the coordinator with MIDI data and configuration
    pub fn initialize(
        &mut self,
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        track_channel_configs: &[TrackChannelConfig],
        config: &Config,
        layout: &KeyboardLayout,
        measures: Arc<[Duration]>,
        vertical_guidelines: bool,
        horizontal_guidelines: bool,
    ) {
        // Initialize waterfall renderer
        let mut waterfall = PlyWaterfallRenderer::new();
        waterfall.initialize(tracks, hidden_tracks, track_channel_configs, config, layout);
        self.waterfall = Some(waterfall);

        // Initialize keyboard renderer
        let mut keyboard = PlyKeyboardRenderer::new(layout.clone());
        keyboard.initialize((0.0, layout.height));
        self.keyboard = Some(keyboard);

        // Initialize guidelines renderer
        let mut guidelines = PlyGuidelineRenderer::new(
            layout.clone(),
            (0.0, 0.0),
            vertical_guidelines,
            horizontal_guidelines,
            measures,
        );
        guidelines.initialize();
        self.guidelines = Some(guidelines);

        // Initialize note labels renderer if enabled
        if config.note_labels() {
            let notes = self.waterfall.as_ref().unwrap().notes();
            let mut note_labels = PlyNoteLabelsRenderer::new((0.0, 0.0), notes);
            note_labels.initialize();
            self.note_labels = Some(note_labels);
        }

        self.initialized = true;
    }

    /// Update all renderers with current time
    pub fn update(&mut self, time: f32, animation_speed: f32, scale: f32, keyboard_y: f32) {
        if !self.initialized {
            return;
        }

        // Update waterfall
        if let Some(waterfall) = &mut self.waterfall {
            waterfall.update(time);
        }

        // Update keyboard
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.update();
        }

        // Update guidelines
        if let Some(guidelines) = &mut self.guidelines {
            guidelines.update(time, animation_speed, scale);
        }

        // Update note labels
        if let Some(note_labels) = &mut self.note_labels {
            if let Some(keyboard) = &self.keyboard {
                note_labels.update(
                    keyboard.layout(),
                    animation_speed,
                    time,
                    scale,
                    keyboard_y,
                );
            }
        }
    }

    /// Get mutable reference to waterfall renderer
    pub fn waterfall_mut(&mut self) -> Option<&mut PlyWaterfallRenderer> {
        self.waterfall.as_mut()
    }

    /// Get mutable reference to keyboard renderer
    pub fn keyboard_mut(&mut self) -> Option<&mut PlyKeyboardRenderer> {
        self.keyboard.as_mut()
    }

    /// Get mutable reference to guidelines renderer
    pub fn guidelines_mut(&mut self) -> Option<&mut PlyGuidelineRenderer> {
        self.guidelines.as_mut()
    }

    /// Get mutable reference to note labels renderer
    pub fn note_labels_mut(&mut self) -> Option<&mut PlyNoteLabelsRenderer> {
        self.note_labels.as_mut()
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Handle keyboard layout changes
    pub fn set_keyboard_layout(&mut self, layout: KeyboardLayout) {
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.set_layout(layout.clone());
        }
        if let Some(guidelines) = &mut self.guidelines {
            guidelines.set_layout(layout.clone());
        }
    }

    /// Handle keyboard position changes
    pub fn set_keyboard_position(&mut self, pos: (f32, f32)) {
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.set_pos(pos);
        }
        if let Some(guidelines) = &mut self.guidelines {
            guidelines.set_pos(pos);
        }
        if let Some(note_labels) = &mut self.note_labels {
            note_labels.set_pos(pos);
        }
    }

    /// Reset all keyboard notes
    pub fn reset_keyboard_notes(&mut self) {
        if let Some(keyboard) = &mut self.keyboard {
            keyboard.reset_notes();
        }
    }
}

impl Default for PlyRendererCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
