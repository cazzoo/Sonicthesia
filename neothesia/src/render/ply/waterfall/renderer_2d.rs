use macroquad::prelude::*;
use piano_layout::KeyboardLayout;

use super::{NoteVisual, WaterfallConfig, WaterfallNote, WaterfallRenderer};

pub struct Waterfall2D {
    notes: Vec<WaterfallNote>,
    config: WaterfallConfig,
    layout: KeyboardLayout,
    current_time: f32,
    visuals: Vec<NoteVisual>,
    pressed_keys: Vec<u8>,
    keyboard_top: f32,
    keyboard_height: f32,
}

impl Waterfall2D {
    pub fn new(notes: &[WaterfallNote], config: &WaterfallConfig, layout: &KeyboardLayout) -> Self {
        let screen_h = screen_height();
        let keyboard_height = screen_h * 0.2;
        let keyboard_top = screen_h - keyboard_height - 20.0;

        Self {
            notes: notes.to_vec(),
            config: config.clone(),
            layout: layout.clone(),
            current_time: 0.0,
            visuals: Vec::new(),
            pressed_keys: Vec::new(),
            keyboard_top,
            keyboard_height,
        }
    }

    fn calculate_note_visual(&self, note: &WaterfallNote) -> Option<NoteVisual> {
        let range_start = self.layout.range.start();
        if note.note < range_start || note.note > self.layout.range.end() {
            return None;
        }

        let key_idx = (note.note - range_start) as usize;
        if key_idx >= self.layout.keys.len() {
            return None;
        }

        let key = &self.layout.keys[key_idx];
        let is_sharp = key.kind().is_sharp();

        let pixels_per_second = 200.0 * self.config.animation_speed;
        let note_duration = note.end - note.start;
        let time_until_start = note.start - self.current_time;
        let y = self.keyboard_top - (time_until_start * pixels_per_second);
        let height = note_duration * pixels_per_second;

        if y + height < 0.0 || y > self.keyboard_top {
            return None;
        }

        let color_idx = note.track_color_id % self.config.colors.len();
        let color = self.config.colors[color_idx];

        Some(NoteVisual {
            x: key.x(),
            y,
            width: key.width() - 1.0,
            height: height.max(4.0),
            color: (color.0, color.1, color.2, 255),
            is_sharp,
            progress: 0.0,
        })
    }
}

impl WaterfallRenderer for Waterfall2D {
    fn update(&mut self, time: f32, layout: &KeyboardLayout) {
        self.current_time = time;
        self.layout = layout.clone();

        let screen_h = screen_height();
        self.keyboard_height = screen_h * 0.2;
        self.keyboard_top = screen_h - self.keyboard_height - 20.0;

        self.visuals.clear();
        self.pressed_keys.clear();

        let range_start = self.layout.range.start();

        for note in &self.notes {
            if let Some(visual) = self.calculate_note_visual(note) {
                self.visuals.push(visual);
            }

            if time >= note.start && time < note.end {
                if note.note >= range_start && note.note <= self.layout.range.end() {
                    let key_idx = (note.note - range_start) as u8;
                    if !self.pressed_keys.contains(&key_idx) {
                        self.pressed_keys.push(key_idx);
                    }
                }
            }
        }
    }

    fn render(&self) {
        for visual in &self.visuals {
            let color = Color::from_rgba(
                visual.color.0,
                visual.color.1,
                visual.color.2,
                visual.color.3,
            );
            draw_rectangle(visual.x, visual.y, visual.width, visual.height, color);
        }
    }

    fn get_active_notes(&self) -> Vec<&WaterfallNote> {
        self.notes
            .iter()
            .filter(|n| self.current_time >= n.start && self.current_time < n.end)
            .collect()
    }

    fn should_be_pressed(&self, note: u8) -> bool {
        self.pressed_keys.contains(&note)
    }

    fn get_pressed_keys(&self) -> Vec<u8> {
        self.pressed_keys.clone()
    }
}
