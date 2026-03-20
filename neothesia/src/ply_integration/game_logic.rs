//! PLY-based game logic integration for Neothesia
//!
//! This module provides integration between Neothesia's game logic systems
//! (wait mode, play along statistics, rewind controller, LUMI controller)
//! and the PLY engine architecture.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use midi_file::midly::{num::u4, MidiMessage};
use piano_layout::KeyboardRange;

/// Re-export game logic types for convenience
pub use crate::scene::playing_scene::midi_player::{
    MidiEventSource, PlayerStats, ScoreData, NotePress,
};

/// PLY-integrated play along system
/// 
/// This provides PLY-aware extensions for enhanced gameplay features
/// while working with the existing PlayAlong system.
pub struct PlyPlayAlong {
    /// PLY-specific extensions
    ply_extensions: PlyExtensions,
}

/// PLY-specific extensions for play along system
pub struct PlyExtensions {
    /// Visual feedback state for PLY rendering
    pub visual_feedback: HashMap<u8, VisualFeedbackState>,
    /// Timing statistics for PLY-based analytics
    pub timing_stats: TimingStats,
}

/// Visual feedback state for individual notes
#[derive(Debug, Clone)]
pub struct VisualFeedbackState {
    /// When the note became required
    pub required_since: Option<Instant>,
    /// Whether the note was played correctly
    pub played_correctly: bool,
    /// Timing delta (positive = late, negative = early)
    pub timing_delta: Option<Duration>,
}

/// Timing statistics for analytics
#[derive(Debug, Default, Clone)]
pub struct TimingStats {
    /// Notes played with perfect timing
    pub perfect_notes: usize,
    /// Notes played with good timing
    pub good_notes: usize,
    /// Notes played with okay timing
    pub okay_notes: usize,
    /// Notes played with bad timing
    pub bad_notes: usize,
}

impl PlyPlayAlong {
    /// Create a new PLY-integrated play along system
    pub fn new() -> Self {
        Self {
            ply_extensions: PlyExtensions {
                visual_feedback: HashMap::new(),
                timing_stats: TimingStats::default(),
            },
        }
    }

    /// Update visual feedback states based on required notes
    pub fn update_visual_feedback(&mut self, required_notes: &HashMap<u8, NotePress>) {
        let now = Instant::now();

        // Update feedback state for required notes
        for (&note_id, _) in required_notes.iter() {
            let feedback = self.ply_extensions.visual_feedback
                .entry(note_id)
                .or_insert_with(|| VisualFeedbackState {
                    required_since: Some(now),
                    played_correctly: false,
                    timing_delta: None,
                });

            // Calculate how long the note has been required
            if let Some(required_since) = feedback.required_since {
                let _waiting_duration = now.duration_since(required_since);
                // Note has been waiting for this long
            }
        }

        // Clean up feedback for notes that are no longer required
        self.ply_extensions.visual_feedback.retain(|&note_id, _| {
            required_notes.contains_key(&note_id)
        });
    }

    /// Handle a note press event for visual feedback
    pub fn handle_note_press(&mut self, note_id: u8, required_notes: &HashMap<u8, NotePress>) {
        let now = Instant::now();

        if required_notes.contains_key(&note_id) {
            // Note was required - update visual feedback
            if let Some(feedback) = self.ply_extensions.visual_feedback.get_mut(&note_id) {
                feedback.played_correctly = true;
                // We can't access timestamp directly, so we'll use the required_since time
                if let Some(required_since) = feedback.required_since {
                    feedback.timing_delta = Some(now.duration_since(required_since));
                }
            }
        }
    }

    /// Categorize a timing delta and update statistics
    pub fn categorize_timing(&mut self, delta: Duration) -> &'static str {
        const PERFECT_THRESHOLD: Duration = Duration::from_millis(50);
        const GOOD_THRESHOLD: Duration = Duration::from_millis(100);
        const OKAY_THRESHOLD: Duration = Duration::from_millis(200);

        if delta <= PERFECT_THRESHOLD {
            self.ply_extensions.timing_stats.perfect_notes += 1;
            "perfect"
        } else if delta <= GOOD_THRESHOLD {
            self.ply_extensions.timing_stats.good_notes += 1;
            "good"
        } else if delta <= OKAY_THRESHOLD {
            self.ply_extensions.timing_stats.okay_notes += 1;
            "okay"
        } else {
            self.ply_extensions.timing_stats.bad_notes += 1;
            "bad"
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.ply_extensions.visual_feedback.clear();
        self.ply_extensions.timing_stats = TimingStats::default();
    }

    /// Get visual feedback state for a note
    pub fn get_visual_feedback(&self, note_id: u8) -> Option<&VisualFeedbackState> {
        self.ply_extensions.visual_feedback.get(&note_id)
    }

    /// Get timing statistics
    pub fn get_timing_stats(&self) -> &TimingStats {
        &self.ply_extensions.timing_stats
    }

    /// Get mutable reference to extensions
    pub fn extensions_mut(&mut self) -> &mut PlyExtensions {
        &mut self.ply_extensions
    }
}

impl Default for PlyPlayAlong {
    fn default() -> Self {
        Self::new()
    }
}

/// PLY-integrated rewind controller
/// 
/// Provides enhanced rewind functionality with PLY-aware features
/// like smooth scrubbing and visual feedback.
pub struct PlyRewindController {
    /// Inner rewind controller state
    state: RewindState,
    /// Smooth scrubbing state
    scrubbing: ScrubbingState,
}

/// Rewind state
enum RewindState {
    /// Not rewinding
    None,
    /// Keyboard rewinding with speed
    Keyboard { speed: i64, was_paused: bool },
    /// Mouse rewinding
    Mouse { was_paused: bool },
}

/// Scrubbing state for smooth timeline navigation
struct ScrubbingState {
    /// Target time for smooth scrubbing
    target_time: Option<f32>,
    /// Current scrubbing position (0.0 - 1.0)
    scrub_position: f32,
    /// Whether scrubbing is active
    is_active: bool,
}

impl Default for ScrubbingState {
    fn default() -> Self {
        Self {
            target_time: None,
            scrub_position: 0.0,
            is_active: false,
        }
    }
}

impl PlyRewindController {
    /// Create a new PLY rewind controller
    pub fn new() -> Self {
        Self {
            state: RewindState::None,
            scrubbing: ScrubbingState::default(),
        }
    }

    /// Check if currently rewinding
    pub fn is_rewinding(&self) -> bool {
        !matches!(self.state, RewindState::None)
    }

    /// Check if keyboard rewinding
    pub fn is_keyboard_rewinding(&self) -> bool {
        matches!(self.state, RewindState::Keyboard { .. })
    }

    /// Check if mouse rewinding
    pub fn is_mouse_rewinding(&self) -> bool {
        matches!(self.state, RewindState::Mouse { .. })
    }

    /// Start mouse rewind
    pub fn start_mouse_rewind(&mut self, was_paused: bool) {
        self.state = RewindState::Mouse { was_paused };
        self.scrubbing.is_active = true;
    }

    /// Start keyboard rewind
    pub fn start_keyboard_rewind(&mut self, speed: i64, was_paused: bool) {
        self.state = RewindState::Keyboard { speed, was_paused };
    }

    /// Stop rewinding
    pub fn stop_rewind(&mut self) -> (bool, Option<i64>) {
        let (was_paused, speed) = match &self.state {
            RewindState::Keyboard { speed, was_paused } => (*was_paused, Some(*speed)),
            RewindState::Mouse { was_paused } => (*was_paused, None),
            RewindState::None => (false, None),
        };

        self.state = RewindState::None;
        self.scrubbing.is_active = false;

        (was_paused, speed)
    }

    /// Update rewind state
    pub fn update(&mut self, delta: Duration, modifiers: RewindModifiers) -> Option<i64> {
        if let RewindState::Keyboard { speed, .. } = self.state {
            let mut v = speed;
            if modifiers.shift {
                v *= 2;
            } else if modifiers.control {
                v /= 2;
            }

            Some((100.0 * v as f32 * delta.as_secs_f32()).round() as i64)
        } else {
            None
        }
    }

    /// Set scrub position (0.0 - 1.0)
    pub fn set_scrub_position(&mut self, position: f32) {
        self.scrubbing.scrub_position = position.clamp(0.0, 1.0);
        self.scrubbing.target_time = Some(position);
    }

    /// Get scrub position
    pub fn scrub_position(&self) -> f32 {
        self.scrubbing.scrub_position
    }

    /// Check if scrubbing is active
    pub fn is_scrubbing(&self) -> bool {
        self.scrubbing.is_active
    }
}

impl Default for PlyRewindController {
    fn default() -> Self {
        Self::new()
    }
}

/// Modifiers for rewind speed
#[derive(Debug, Clone, Copy)]
pub struct RewindModifiers {
    /// Shift key multiplier
    pub shift: bool,
    /// Control key divider
    pub control: bool,
}

/// PLY-integrated LUMI controller
/// 
/// Provides enhanced LUMI integration with PLY-aware features
/// like dynamic lighting effects and visual feedback synchronization.
pub struct PlyLumiController {
    /// Inner LUMI controller
    inner: crate::lumi_controller::LumiController,
    /// PLY-specific lighting effects
    lighting_effects: LightingEffects,
}

/// Lighting effects for PLY integration
struct LightingEffects {
    /// Current effect mode
    mode: LightingMode,
    /// Effect intensity (0.0 - 1.0)
    intensity: f32,
    /// Effect color override
    color_override: Option<(u8, u8, u8)>,
}

/// Lighting mode
enum LightingMode {
    /// Standard mode
    Standard,
    /// Wait mode highlighting
    WaitMode,
    /// Success feedback
    Success,
    /// Error feedback
    Error,
    /// Hinting mode
    Hinting,
}

impl Default for LightingEffects {
    fn default() -> Self {
        Self {
            mode: LightingMode::Standard,
            intensity: 1.0,
            color_override: None,
        }
    }
}

impl PlyLumiController {
    /// Create a new PLY-integrated LUMI controller
    pub fn new(connection: crate::output_manager::OutputConnection) -> Self {
        Self {
            inner: crate::lumi_controller::LumiController::new(connection),
            lighting_effects: LightingEffects::default(),
        }
    }

    /// Begin API mode
    pub fn begin_api_mode(&mut self) {
        self.inner.begin_api_mode();
    }

    /// End API mode
    pub fn end_api_mode(&mut self) {
        self.inner.end_api_mode();
    }

    /// Clear all keys
    pub fn clear_all(&mut self) {
        self.inner.clear_all();
    }

    /// Set key color with PLY-aware effects
    pub fn set_key_color(&mut self, note: u8, r: u8, g: u8, b: u8) {
        // Apply intensity based on current mode
        let (r, g, b) = self.apply_lighting_mode(r, g, b);
        self.inner.set_key_color(note, r, g, b);
    }

    /// Set key dim (hinting)
    pub fn set_key_dim(&mut self, note: u8, r: u8, g: u8, b: u8) {
        let (r, g, b) = self.apply_lighting_mode(r, g, b);
        self.inner.set_key_dim(note, r, g, b);
    }

    /// Clear key
    pub fn clear_key(&mut self, note: u8) {
        self.inner.clear_key(note);
    }

    /// Set brightness
    pub fn set_brightness(&mut self, value: u8) {
        self.inner.set_brightness(value);
    }

    /// Set color mode
    pub fn set_color_mode(&mut self, mode: u8) {
        self.inner.set_color_mode(mode);
    }

    /// Set lighting mode
    pub fn set_lighting_mode(&mut self, mode: LightingMode) {
        self.lighting_effects.mode = mode;
    }

    /// Set effect intensity
    pub fn set_intensity(&mut self, intensity: f32) {
        self.lighting_effects.intensity = intensity.clamp(0.0, 1.0);
    }

    /// Apply lighting mode to color
    fn apply_lighting_mode(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let intensity = self.lighting_effects.intensity;

        match self.lighting_effects.mode {
            LightingMode::Standard => {
                // Apply intensity
                (
                    (r as f32 * intensity).round() as u8,
                    (g as f32 * intensity).round() as u8,
                    (b as f32 * intensity).round() as u8,
                )
            }
            LightingMode::WaitMode => {
                // Pulse effect for wait mode
                let pulse = 0.7 + 0.3 * (intensity * 2.0 % 1.0);
                (
                    (r as f32 * pulse).round() as u8,
                    (g as f32 * pulse).round() as u8,
                    (b as f32 * pulse).round() as u8,
                )
            }
            LightingMode::Success => {
                // Bright green flash
                (
                    ((r as f32 * 0.3 + 0.0) * intensity).round() as u8,
                    ((g as f32 * 0.3 + 255.0) * intensity).round() as u8,
                    ((b as f32 * 0.3 + 0.0) * intensity).round() as u8,
                )
            }
            LightingMode::Error => {
                // Red flash
                (
                    ((r as f32 * 0.3 + 255.0) * intensity).round() as u8,
                    ((g as f32 * 0.3 + 0.0) * intensity).round() as u8,
                    ((b as f32 * 0.3 + 0.0) * intensity).round() as u8,
                )
            }
            LightingMode::Hinting => {
                // Dim blue for hinting
                (
                    (r as f32 * 0.4 * intensity).round() as u8,
                    (g as f32 * 0.4 * intensity).round() as u8,
                    ((b as f32 * 0.4 + 100.0) * intensity).round() as u8,
                )
            }
        }
    }

    /// Update wait mode lighting
    pub fn update_wait_mode_lighting(&mut self, required_notes: &HashMap<u8, crate::scene::playing_scene::midi_player::NotePress>) {
        self.set_lighting_mode(LightingMode::WaitMode);
        // The actual key lighting is handled in the main loop
    }

    /// Update success lighting
    pub fn update_success_lighting(&mut self, note_id: u8) {
        self.set_lighting_mode(LightingMode::Success);
        self.set_key_color(note_id, 0, 255, 0);
        
        // Reset to standard after a brief flash
        self.set_lighting_mode(LightingMode::Standard);
    }

    /// Update error lighting
    pub fn update_error_lighting(&mut self, note_id: u8) {
        self.set_lighting_mode(LightingMode::Error);
        self.set_key_color(note_id, 255, 0, 0);
        
        // Reset to standard after a brief flash
        self.set_lighting_mode(LightingMode::Standard);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ply_playalong_creation() {
        let playalong = PlyPlayAlong::new();
        assert!(playalong.get_visual_feedback(60).is_none());
    }

    #[test]
    fn test_ply_rewind_controller() {
        let mut controller = PlyRewindController::new();
        assert!(!controller.is_rewinding());
        
        controller.start_mouse_rewind(false);
        assert!(controller.is_rewinding());
        assert!(controller.is_mouse_rewinding());
        
        let (was_paused, speed) = controller.stop_rewind();
        assert!(!was_paused);
        assert!(speed.is_none());
        assert!(!controller.is_rewinding());
    }

    #[test]
    fn test_scrubbing() {
        let mut controller = PlyRewindController::new();
        controller.set_scrub_position(0.5);
        assert_eq!(controller.scrub_position(), 0.5);
        assert!(controller.is_scrubbing());
    }
}
