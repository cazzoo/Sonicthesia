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
use std::collections::HashMap;

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

    fn update(&mut self, dt: f32) {
        let old_value = self.value;
        let diff = self.target - self.value;

        if diff.abs() > 0.5 {
            self.value += diff * 0.5;
        } else {
            self.value += diff * self.speed * dt;
        }

        self.value = self.value.clamp(0.0, 1.0);

        if (old_value - self.value).abs() > 0.01 {
            log::trace!(
                "Animation update: {:.3} -> {:.3} (target: {:.3})",
                old_value,
                self.value,
                self.target
            );
        }
    }

    fn press(&mut self) {
        self.target = 1.0;
        if self.value < 0.5 {
            self.value = 0.5;
        }
    }

    fn release(&mut self) {
        self.target = 0.0;
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
}

impl PianoKeyboardRenderer {
    /// Create a new piano keyboard renderer
    pub fn new(layout: KeyboardLayout, config: &Config) -> Self {
        let window_width = screen_width();
        let window_height = screen_height();

        let (width, height) = Self::calculate_keyboard_size(window_width, window_height);
        let (x, y) = Self::calculate_keyboard_position(width, height, window_height);

        let keys = Self::build_visual_keys(&layout, x, y, width, height);

        log::info!(
            "🎹 PianoKeyboardRenderer created with {} keys, range {:?}",
            keys.len(),
            layout
                .keys
                .first()
                .map(|k| k.note_id())
                .zip(layout.keys.last().map(|k| k.note_id()))
        );

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

        layout
            .keys
            .iter()
            .map(|key| {
                let x = offset_x + key.x() * scale_x;
                let y = offset_y;
                let width = key.width() * scale_x;
                let height = total_height;

                VisualKey {
                    note: key.note_id(),
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
        if mouse_button != MouseButton::Left {
            return None;
        }

        if is_pressed {
            self.mouse_is_down = true;
            if let Some(note) = self.get_key_at_position(mouse_pos) {
                log::info!("🖱️  MOUSE DOWN on key {}", note);
                self.mouse_pressed_keys.insert(note, true);
                self.set_key_pressed(note, true);
                return Some(vec![note]);
            }
        } else {
            log::info!(
                "🖱️  MOUSE UP - releasing {} keys",
                self.mouse_pressed_keys.len()
            );
            self.mouse_is_down = false;
            let released_notes: Vec<u8> = self.mouse_pressed_keys.keys().copied().collect();
            for note in &released_notes {
                self.set_key_pressed(*note, false);
            }
            self.mouse_pressed_keys.clear();
            return Some(released_notes);
        }

        None
    }

    /// Handle mouse drag (return notes that started being pressed)
    pub fn handle_mouse_drag(&mut self, mouse_pos: Vec2) -> Option<Vec<u8>> {
        if !self.mouse_is_down {
            return None;
        }

        let mut new_pressed_notes = Vec::new();

        if let Some(note) = self.get_key_at_position(mouse_pos) {
            if !self.mouse_pressed_keys.contains_key(&note) {
                // New key pressed during drag
                self.mouse_pressed_keys.insert(note, true);
                self.set_key_pressed(note, true);
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

        for note in keys_to_release {
            self.mouse_pressed_keys.remove(&note);
            self.set_key_pressed(note, false);
        }

        if new_pressed_notes.is_empty() {
            None
        } else {
            Some(new_pressed_notes)
        }
    }

    fn get_key_at_position(&self, pos: Vec2) -> Option<u8> {
        // Check black keys first (they're on top)
        for key in self.keys.iter().filter(|k| k.is_sharp) {
            if self.is_point_in_key(pos, key) {
                log::debug!(
                    "🖱️  Click detected on BLACK key {} at ({:.1}, {:.1})",
                    key.note,
                    pos.x,
                    pos.y
                );
                return Some(key.note);
            }
        }

        // Then check white keys
        for key in self.keys.iter().filter(|k| !k.is_sharp) {
            if self.is_point_in_key(pos, key) {
                log::debug!(
                    "🖱️  Click detected on WHITE key {} at ({:.1}, {:.1})",
                    key.note,
                    pos.x,
                    pos.y
                );
                return Some(key.note);
            }
        }

        log::debug!("🖱️  No key detected at ({:.1}, {:.1})", pos.x, pos.y);
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
        log::debug!(
            "set_key_pressed: note={}, pressed={}, has_key={}",
            note,
            pressed,
            self.get_render_key(note).is_some()
        );
        if let Some(key) = self.get_render_key_mut(note) {
            if pressed {
                key.animation.press();
                log::info!(
                    "✅ Key {} pressed - anim set to {}",
                    note,
                    key.animation.value
                );
            } else {
                key.animation.release();
                log::info!(
                    "❌ Key {} released - anim set to {}",
                    note,
                    key.animation.value
                );
            }
        } else {
            log::warn!("⚠️  Key {} not in keyboard range", note);
        }
    }

    /// Handle keyboard MIDI note events
    pub fn handle_note_event(&mut self, note: u8, velocity: u8) {
        let pressed = velocity > 0;
        log::info!(
            "🎹 MIDI note event: note={}, velocity={}, pressed={}",
            note,
            velocity,
            pressed
        );

        // Update keyboard state tracking
        if pressed {
            self.keyboard_pressed_notes.insert(note, true);
            self.set_key_pressed(note, true);
        } else {
            self.keyboard_pressed_notes.remove(&note);
            self.set_key_pressed(note, false);
        }
    }

    /// Update animations
    pub fn update(&mut self, dt: f32) {
        self.update_window_size();

        for key in &mut self.keys {
            key.animation.update(dt);
        }
    }

    pub fn render(&self) {
        let pressed_count = self
            .keys
            .iter()
            .filter(|k| k.animation.value > 0.01)
            .count();
        if pressed_count > 0 {
            log::info!("🎨 Rendering {} pressed keys", pressed_count);
        }

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

        if anim > 0.01 {
            log::debug!(
                "Drawing key {} with anim {:.2}, color=({:.2},{:.2},{:.2})",
                key.note,
                anim,
                pressed_color.r,
                pressed_color.g,
                pressed_color.b
            );
        }

        let r = base_color.r + (pressed_color.r - base_color.r) * anim;
        let g = base_color.g + (pressed_color.g - base_color.g) * anim;
        let b = base_color.b + (pressed_color.b - base_color.b) * anim;
        let color = Color { r, g, b, a: 1.0 };

        // 2.5D depth effect
        if self.theme.depth_2d5 > 0.0 {
            let depth = self.theme.depth_2d5;
            let shadow_color = Color {
                r: base_color.r * 0.5,
                g: base_color.g * 0.5,
                b: base_color.b * 0.5,
                a: 1.0,
            };

            // Draw shadow/depth
            draw_rectangle(
                key.x + depth,
                key.y + depth,
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

        // Draw border
        draw_rectangle_lines(
            key.x,
            key.y,
            key.width,
            key.height,
            1.0,
            self.theme.border_color,
        );

        // Glow effect for pressed keys
        if anim > 0.1 && self.theme.glow_intensity > 0.0 {
            let glow_color = Color {
                r: pressed_color.r,
                g: pressed_color.g,
                b: pressed_color.b,
                a: anim * self.theme.glow_intensity,
            };

            // Draw glow (larger semi-transparent rectangle)
            draw_rectangle(
                key.x - 2.0,
                key.y - 2.0,
                key.width + 4.0,
                key.height + 4.0,
                glow_color,
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
