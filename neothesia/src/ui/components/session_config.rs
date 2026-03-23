use super::glass_panel::GlassPanel;
use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

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
        draw_text(
            "⚙",
            current_x,
            content_y + 16.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        draw_text(
            "Session Configuration",
            current_x + 28.0,
            content_y + 16.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        current_x = self.x + spacing::XL;
        let section_y = content_y + 50.0;

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "DIFFICULTY LEVEL",
            current_x,
            section_y,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

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
            let text_width = measure_text(level.label(), None, 12, 1.0).width;
            draw_text(
                level.label(),
                btn_x + (btn_w - text_width) / 2.0,
                btn_y + 22.0,
                12.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            if is_hovered && mouse_pressed {
                self.difficulty = *level;
            }
        }

        let speed_x = self.x + 250.0;
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "PLAYBACK SPEED",
            speed_x,
            section_y,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
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
            let (lr, lg, lb) = if is_current {
                colors::to_normalized(colors::PRIMARY)
            } else {
                colors::to_normalized(colors::ON_SURFACE_VARIANT)
            };
            draw_text(
                label,
                label_x,
                slider_y + 32.0,
                10.0,
                Color::new(lr, lg, lb, 1.0),
            );
        }

        let visual_x = speed_x + 230.0;
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "VISUAL ASSISTANCE",
            visual_x,
            section_y,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

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

        draw_text(
            "Fingering Numbers",
            visual_x,
            toggle_y + toggle_h + 18.0,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let input_x = visual_x + 200.0;
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "INPUT DEVICE",
            input_x,
            section_y,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (device_r, device_g, device_b) = colors::to_normalized(colors::SECONDARY);
        draw_text(
            "🎹",
            input_x,
            section_y + 28.0,
            14.0,
            Color::new(device_r, device_g, device_b, 1.0),
        );

        draw_text(
            &format!("{} Connected", self.device_name),
            input_x + 24.0,
            section_y + 28.0,
            12.0,
            Color::new(device_r, device_g, device_b, 1.0),
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
