//! Enhanced piano keyboard renderer for PLY mode
//!
//! Provides an interactive, animated piano keyboard with:
//! - Visual feedback on key press
//! - Mouse click and drag support
//! - Responsive sizing and positioning
//! - Theme/style customization
//! - Keyboard input synchronization

use macroquad::prelude::*;
use neothesia_core::config::Config;
use piano_layout::KeyboardLayout;
use std::collections::{HashMap, HashSet};

/// Input source for key press events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputSource {
    Mouse,
    Keyboard,
    Midi,
    Gamepad,
}

/// Visual theme for the piano keyboard
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
            self.value, self.target
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
    theme: PianoTheme,
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
        let window_width = screen_width();
        let window_height = screen_height();

        let (width, height) = Self::calculate_keyboard_size(window_width, window_height);
        let (x, y) = Self::calculate_keyboard_position(width, height, window_height);

        let keys = Self::build_visual_keys(&layout, x, y, width, height);

        Self {
            layout,
            keys,
            theme: PianoTheme::default(),
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
    pub fn set_theme(&mut self, theme: PianoTheme) {
        self.theme = theme;
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
        let new_width = screen_width();
        let new_height = screen_height();

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

    /// Calculate keyboard size based on window dimensions
    fn calculate_keyboard_size(window_width: f32, window_height: f32) -> (f32, f32) {
        let width = window_width * 0.95; // 95% of window width
        let height = window_height * 0.2; // 20% of window height
        (width, height)
    }

    /// Calculate keyboard position (bottom center)
    fn calculate_keyboard_position(width: f32, height: f32, window_height: f32) -> (f32, f32) {
        let x = (screen_width() - width) / 2.0;
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
        let total_keys = layout.keys.len() as f32;
        let avg_key_width = total_width / total_keys;
        let scale_x = total_width / (total_keys * avg_key_width);

        // Get the MIDI note range from the layout
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
                released_notes.len(), released_notes
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
                keys_to_release.len(), keys_to_release
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
                new_pressed_notes.len(), new_pressed_notes
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
        log::debug!("🎹 set_key_pressed(note={}, pressed={}, is_actually_pressed={})", note, pressed, is_actually_pressed);
        
        if let Some(key) = self.get_render_key_mut(note) {
            if pressed {
                log::debug!("🎹   → Calling animation.press(), current value={:.3}", key.animation.value);
                key.animation.press();
            } else {
                log::debug!("🎹   → Calling animation.release(), current value={:.3}", key.animation.value);
                key.animation.release();
            }
        }
    }

    /// Handle keyboard MIDI note events
    pub fn handle_note_event(&mut self, note: u8, velocity: u8) {
        let pressed = velocity > 0;

        log::debug!(
            "[DEBUG] [handle_note_event] Entry\n  - Context: note={}, velocity={}, pressed={}",
            note, velocity, pressed
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

    /// Add an input source for a pressed key
    fn add_input_source(&mut self, note: u8, source: InputSource) {
        log::debug!(
            "[DEBUG] [add_input_source] Entry\n  - Context: note={}, source={:?}",
            note, source
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
            note, source
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
            dt, self.keys.len()
        );

        self.update_window_size();

        // Collect pressed states first to avoid borrow checker issues
        let pressed_notes: std::collections::HashSet<u8> = self.pressed_keys_sources.keys().copied().collect();

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
        let (base_color, pressed_color) = if key.is_sharp {
            (self.theme.black_key_normal, self.theme.black_key_pressed)
        } else {
            (self.theme.white_key_normal, self.theme.white_key_pressed)
        };

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
        if self.theme.depth_2d5 > 0.0 {
            let depth = self.theme.depth_2d5;
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
        if self.theme.corner_radius > 0.0 {
            // Rounded rectangle (simulated with regular rect for now)
            draw_rectangle(key.x, key.y, key.width, key.height, color);
        } else {
            draw_rectangle(key.x, key.y, key.width, key.height, color);
        }

        // Draw border - enhanced for pressed keys
        let border_width = if anim > 0.5 { 2.0 } else { 1.0 };
        let border_color = if anim > 0.5 {
            // Use a brighter border for pressed keys
            Color {
                r: pressed_color.r * 1.2,
                g: pressed_color.g * 1.2,
                b: pressed_color.b * 1.2,
                a: 1.0,
            }
        } else {
            self.theme.border_color
        };
        
        draw_rectangle_lines(
            key.x,
            key.y,
            key.width,
            key.height,
            border_width,
            border_color,
        );

        // Enhanced glow effect for pressed keys
        if anim > 0.1 && self.theme.glow_intensity > 0.0 {
            // Multi-layer glow for more prominent effect
            let glow_layers = if anim > 0.5 { 3 } else { 1 };
            
            for layer in 0..glow_layers {
                let offset = (layer + 1) as f32 * 1.5;
                let alpha = anim * self.theme.glow_intensity * (1.0 - layer as f32 * 0.2);
                
                let glow_color = Color {
                    r: pressed_color.r,
                    g: pressed_color.g,
                    b: pressed_color.b,
                    a: alpha,
                };

                // Draw glow (larger semi-transparent rectangle)
                draw_rectangle(
                    key.x - offset,
                    key.y - offset,
                    key.width + offset * 2.0,
                    key.height + offset * 2.0,
                    glow_color,
                );
            }
        }

        // Inner highlight for pressed keys (simulates light reflection)
        if anim > 0.5 {
            let highlight_color = Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15 * anim,
            };
            
            // Draw a small highlight rectangle at the top of the key
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
