use super::glass_panel::GlassPanel;
use crate::scene::ply_fonts;
use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl DifficultyLevel {
    pub fn label(&self) -> &'static str {
        match self {
            DifficultyLevel::Easy => "EASY",
            DifficultyLevel::Medium => "MED",
            DifficultyLevel::Hard => "HARD",
        }
    }
}

pub struct SessionConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub difficulty: DifficultyLevel,
    pub speed: f32,
    pub fingering_enabled: bool,
    pub device_connected: bool,
    pub device_name: String,
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
        }
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) {
        let panel_height = 200.0;
        let panel = GlassPanel::new(self.x, self.y, self.width, panel_height);
        panel.render();

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
        ply_fonts::draw_label("Playback Speed", speed_x, section_y, 10.0, label_color);

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
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let fill_w = slider_w * ((self.speed - 0.5) / 1.0);
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            slider_x,
            slider_y,
            fill_w,
            slider_h,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let thumb_x = slider_x + fill_w - 6.0;
        let thumb_y = slider_y - 4.0;
        let thumb_size = 12.0;
        draw_circle(
            thumb_x + thumb_size / 2.0,
            thumb_y + thumb_size / 2.0,
            thumb_size / 2.0,
            Color::new(fill_r, fill_g, fill_b, 1.0),
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
                Color::new(mark_r, mark_g, mark_b, 1.0),
            );
        }

        let labels = ["0.5x", "0.75x", "1.0x", "1.5x"];
        for (i, label) in labels.iter().enumerate() {
            let label_x = slider_x + (slider_w / 4.0) * i as f32;
            let is_current = (self.speed - [0.5, 0.75, 1.0, 1.5][i]).abs() < 0.01;
            let speed_color = if is_current {
                Color::new(fill_r, fill_g, fill_b, 1.0)
            } else {
                label_color
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
    }

    pub fn handle_speed_drag(&mut self, mx: f32, my: f32, mouse_down: bool) {
        let slider_x = self.x + 250.0;
        let slider_y = self.y + spacing::XL + 50.0 + 16.0;
        let slider_w = 180.0;

        if mouse_down && my >= slider_y - 10.0 && my <= slider_y + 14.0 {
            let ratio = ((mx - slider_x) / slider_w).clamp(0.0, 1.0);
            self.speed = 0.5 + ratio * 1.0;
        }
    }
}
