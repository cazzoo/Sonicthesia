use super::glass_panel::GlassPanel;
use crate::common::{DifficultyLevel, HandSelection, PlayMode};
use crate::scene::ply_fonts;
use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

pub struct SessionConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub difficulty: DifficultyLevel,
    pub speed: f32,
    pub fingering_enabled: bool,
    pub device_connected: bool,
    pub device_name: String,
    pub hand_selection: HandSelection,
    pub midi_gain: f32,
    pub active_mode: PlayMode,
}

impl SessionConfig {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: 800.0,
            difficulty: DifficultyLevel::Medium,
            speed: 0.75,
            fingering_enabled: true,
            device_connected: true,
            device_name: "Obsidian-88 MKII".to_string(),
            hand_selection: HandSelection::Both,
            midi_gain: 1.0,
            active_mode: PlayMode::Practice,
        }
    }

    pub fn height(&self) -> f32 {
        280.0
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        self.active_mode = mode;
        if mode == PlayMode::Play {
            self.hand_selection = HandSelection::Both;
        }
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) {
        let panel_height = 280.0;
        let panel = GlassPanel::new(self.x, self.y, self.width, panel_height);
        panel.render();

        let is_play_mode = self.active_mode == PlayMode::Play;
        let speed_alpha = if is_play_mode { 0.3 } else { 1.0 };
        let hand_alpha = if is_play_mode { 0.3 } else { 1.0 };

        let mut current_x = self.x + spacing::XL;
        let content_y = self.y + spacing::XL;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        let title_color = Color::new(title_r, title_g, title_b, 1.0);
        ply_fonts::draw_headline("⚙", current_x, content_y + 16.0, 18.0, title_color);
        ply_fonts::draw_headline(
            "Session Configuration",
            current_x + 28.0,
            content_y + 16.0,
            18.0,
            title_color,
        );

        current_x = self.x + spacing::XL;
        let section_y = content_y + 50.0;

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let label_color = Color::new(label_r, label_g, label_b, 1.0);
        ply_fonts::draw_label("Difficulty Level", current_x, section_y, 10.0, label_color);

        let btn_y = section_y + 12.0;
        let btn_h = 32.0;
        let btn_w = 64.0;
        let btn_gap = 8.0;

        for (i, level) in [
            DifficultyLevel::Easy,
            DifficultyLevel::Medium,
            DifficultyLevel::Hard,
        ]
        .iter()
        .enumerate()
        {
            let btn_x = current_x + i as f32 * (btn_w + btn_gap);
            let is_active = self.difficulty == *level;
            let is_hovered =
                mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;

            if is_active {
                let (active_r, active_g, active_b) = colors::to_normalized(colors::PRIMARY);
                draw_rectangle(
                    btn_x,
                    btn_y,
                    btn_w,
                    btn_h,
                    Color::new(active_r, active_g, active_b, 1.0),
                );
            } else {
                let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
                draw_rectangle_lines(
                    btn_x,
                    btn_y,
                    btn_w,
                    btn_h,
                    1.0,
                    Color::new(border_r, border_g, border_b, 0.3),
                );

                if is_hovered {
                    let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::PRIMARY);
                    draw_rectangle_lines(
                        btn_x,
                        btn_y,
                        btn_w,
                        btn_h,
                        1.0,
                        Color::new(hover_r, hover_g, hover_b, 0.5),
                    );
                }
            }

            let text_color = if is_active {
                colors::BLACK
            } else {
                colors::ON_SURFACE
            };
            let (text_r, text_g, text_b) = colors::to_normalized(text_color);
            let btn_text_color = Color::new(text_r, text_g, text_b, 1.0);
            let text_width = measure_text(level.label(), ply_fonts::body_font(), 12, 1.0).width;
            ply_fonts::draw_body(
                level.label(),
                btn_x + (btn_w - text_width) / 2.0,
                btn_y + 22.0,
                12.0,
                btn_text_color,
            );

            if is_hovered && mouse_pressed {
                self.difficulty = *level;
            }
        }

        let speed_x = self.x + 250.0;
        let speed_label_color =
            Color::new(label_color.r, label_color.g, label_color.b, speed_alpha);
        ply_fonts::draw_label(
            "Playback Speed",
            speed_x,
            section_y,
            10.0,
            speed_label_color,
        );

        let slider_x = speed_x;
        let slider_y = section_y + 16.0;
        let slider_w = 180.0;
        let slider_h = 4.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            slider_x,
            slider_y,
            slider_w,
            slider_h,
            Color::new(track_r, track_g, track_b, speed_alpha),
        );

        let fill_w = slider_w * ((self.speed - 0.5) / 1.0);
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            slider_x,
            slider_y,
            fill_w,
            slider_h,
            Color::new(fill_r, fill_g, fill_b, speed_alpha),
        );

        let thumb_x = slider_x + fill_w - 6.0;
        let thumb_y = slider_y - 4.0;
        let thumb_size = 12.0;
        draw_circle(
            thumb_x + thumb_size / 2.0,
            thumb_y + thumb_size / 2.0,
            thumb_size / 2.0,
            Color::new(fill_r, fill_g, fill_b, speed_alpha),
        );

        let marks = [0.5, 0.75, 1.0, 1.25, 1.5];
        for mark in marks.iter() {
            let mark_x = slider_x + slider_w * ((mark - 0.5) / 1.0);
            let (mark_r, mark_g, mark_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle(
                mark_x,
                slider_y + 8.0,
                1.0,
                4.0,
                Color::new(mark_r, mark_g, mark_b, speed_alpha),
            );
        }

        let labels = ["0.5x", "0.75x", "1.0x", "1.5x"];
        for (i, label) in labels.iter().enumerate() {
            let label_x = slider_x + (slider_w / 4.0) * i as f32;
            let is_current = (self.speed - [0.5, 0.75, 1.0, 1.5][i]).abs() < 0.01;
            let speed_color = if is_current {
                Color::new(fill_r, fill_g, fill_b, speed_alpha)
            } else {
                Color::new(label_color.r, label_color.g, label_color.b, speed_alpha)
            };
            ply_fonts::draw_mono(label, label_x, slider_y + 32.0, 10.0, speed_color);
        }

        let visual_x = speed_x + 230.0;
        ply_fonts::draw_label("Visual Assistance", visual_x, section_y, 10.0, label_color);

        let toggle_y = section_y + 12.0;
        let toggle_w = 48.0;
        let toggle_h = 24.0;

        let (toggle_bg_r, toggle_bg_g, toggle_bg_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            visual_x,
            toggle_y,
            toggle_w,
            toggle_h,
            Color::new(toggle_bg_r, toggle_bg_g, toggle_bg_b, 1.0),
        );

        let knob_x = if self.fingering_enabled {
            visual_x + toggle_w - toggle_h + 2.0
        } else {
            visual_x + 2.0
        };
        let knob_size = toggle_h - 4.0;
        draw_circle(
            knob_x + knob_size / 2.0,
            toggle_y + toggle_h / 2.0,
            knob_size / 2.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        );

        ply_fonts::draw_body(
            "Fingering Numbers",
            visual_x,
            toggle_y + toggle_h + 18.0,
            12.0,
            label_color,
        );

        let input_x = visual_x + 200.0;
        ply_fonts::draw_label("Input Device", input_x, section_y, 10.0, label_color);

        let (device_r, device_g, device_b) = colors::to_normalized(colors::SECONDARY);
        let device_color = Color::new(device_r, device_g, device_b, 1.0);
        ply_fonts::draw_body("🎹", input_x, section_y + 28.0, 14.0, device_color);

        ply_fonts::draw_body(
            &format!("{} Connected", self.device_name),
            input_x + 24.0,
            section_y + 28.0,
            12.0,
            device_color,
        );

        let row2_y = content_y + 130.0;

        let hand_label_color = Color::new(label_color.r, label_color.g, label_color.b, hand_alpha);
        ply_fonts::draw_label(
            "Hand Selection",
            current_x,
            row2_y - 12.0,
            10.0,
            hand_label_color,
        );

        let hand_btn_y = row2_y;
        let hand_btn_h = 32.0;
        let hand_btn_w = 72.0;
        let hand_btn_gap = 8.0;

        for (i, hand) in [
            HandSelection::Left,
            HandSelection::Right,
            HandSelection::Both,
        ]
        .iter()
        .enumerate()
        {
            let btn_x = current_x + i as f32 * (hand_btn_w + hand_btn_gap);
            let is_active = self.hand_selection == *hand;
            let is_hovered = mx >= btn_x
                && mx <= btn_x + hand_btn_w
                && my >= hand_btn_y
                && my <= hand_btn_y + hand_btn_h;

            if is_active {
                let (active_r, active_g, active_b) = colors::to_normalized(colors::PRIMARY);
                draw_rectangle(
                    btn_x,
                    hand_btn_y,
                    hand_btn_w,
                    hand_btn_h,
                    Color::new(active_r, active_g, active_b, hand_alpha),
                );
            } else {
                let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
                draw_rectangle_lines(
                    btn_x,
                    hand_btn_y,
                    hand_btn_w,
                    hand_btn_h,
                    1.0,
                    Color::new(border_r, border_g, border_b, 0.3 * hand_alpha),
                );

                if is_hovered {
                    let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::PRIMARY);
                    draw_rectangle_lines(
                        btn_x,
                        hand_btn_y,
                        hand_btn_w,
                        hand_btn_h,
                        1.0,
                        Color::new(hover_r, hover_g, hover_b, 0.5 * hand_alpha),
                    );
                }
            }

            let text_color = if is_active {
                colors::BLACK
            } else {
                colors::ON_SURFACE
            };
            let (text_r, text_g, text_b) = colors::to_normalized(text_color);
            let btn_text_color = Color::new(text_r, text_g, text_b, hand_alpha);
            let text_width = measure_text(hand.label(), ply_fonts::body_font(), 12, 1.0).width;
            ply_fonts::draw_body(
                hand.label(),
                btn_x + (hand_btn_w - text_width) / 2.0,
                hand_btn_y + 22.0,
                12.0,
                btn_text_color,
            );

            if is_hovered && mouse_pressed && !is_play_mode {
                self.hand_selection = *hand;
            }
        }

        let gain_x = self.x + 320.0;
        ply_fonts::draw_label("MIDI Gain", gain_x, row2_y - 12.0, 10.0, label_color);

        let gain_slider_y = row2_y + 4.0;
        let gain_slider_w = 180.0;
        let gain_slider_h = 4.0;

        let (gtrack_r, gtrack_g, gtrack_b) =
            colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            gain_x,
            gain_slider_y,
            gain_slider_w,
            gain_slider_h,
            Color::new(gtrack_r, gtrack_g, gtrack_b, 1.0),
        );

        let gain_fill_w = gain_slider_w * self.midi_gain;
        let (gfill_r, gfill_g, gfill_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            gain_x,
            gain_slider_y,
            gain_fill_w,
            gain_slider_h,
            Color::new(gfill_r, gfill_g, gfill_b, 1.0),
        );

        let gain_thumb_x = gain_x + gain_fill_w - 6.0;
        let gain_thumb_y = gain_slider_y - 4.0;
        let gain_thumb_size = 12.0;
        draw_circle(
            gain_thumb_x + gain_thumb_size / 2.0,
            gain_thumb_y + gain_thumb_size / 2.0,
            gain_thumb_size / 2.0,
            Color::new(gfill_r, gfill_g, gfill_b, 1.0),
        );

        let gain_marks = [0.0, 0.25, 0.5, 0.75, 1.0];
        for mark in gain_marks.iter() {
            let mark_x = gain_x + gain_slider_w * mark;
            let (mark_r, mark_g, mark_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle(
                mark_x,
                gain_slider_y + 8.0,
                1.0,
                4.0,
                Color::new(mark_r, mark_g, mark_b, 1.0),
            );
        }

        let gain_labels = ["0%", "25%", "50%", "75%", "100%"];
        for (i, label) in gain_labels.iter().enumerate() {
            let label_x = gain_x + (gain_slider_w / 4.0) * i as f32;
            let is_current = (self.midi_gain - [0.0, 0.25, 0.5, 0.75, 1.0][i]).abs() < 0.01;
            let gain_color = if is_current {
                Color::new(gfill_r, gfill_g, gfill_b, 1.0)
            } else {
                label_color
            };
            ply_fonts::draw_mono(label, label_x, gain_slider_y + 32.0, 10.0, gain_color);
        }
    }

    pub fn handle_speed_drag(&mut self, mx: f32, my: f32, mouse_down: bool) {
        if self.active_mode == PlayMode::Play {
            return;
        }

        let slider_x = self.x + 250.0;
        let slider_y = self.y + spacing::XL + 50.0 + 16.0;
        let slider_w = 180.0;

        if mouse_down && my >= slider_y - 10.0 && my <= slider_y + 14.0 {
            let ratio = ((mx - slider_x) / slider_w).clamp(0.0, 1.0);
            self.speed = 0.5 + ratio * 1.0;
        }
    }

    pub fn handle_gain_drag(&mut self, mx: f32, my: f32, mouse_down: bool) {
        let row2_y = self.y + spacing::XL + 50.0 + 80.0 + 4.0;
        let slider_x = self.x + 320.0;
        let slider_w = 180.0;

        if mouse_down && my >= row2_y - 10.0 && my <= row2_y + 14.0 {
            let ratio = ((mx - slider_x) / slider_w).clamp(0.0, 1.0);
            self.midi_gain = ratio;
        }
    }
}
