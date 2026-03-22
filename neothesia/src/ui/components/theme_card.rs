use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes, themes::ThemePreset};

pub struct ThemeCard {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub theme_id: String,
    pub theme_name: String,
    pub primary: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
    pub tertiary: (u8, u8, u8),
    pub background: (u8, u8, u8),
    pub is_active: bool,
    pub is_hovered: bool,
}

impl ThemeCard {
    pub fn from_preset(preset: &ThemePreset, x: f32, y: f32, width: f32) -> Self {
        Self {
            x,
            y,
            width,
            height: width / sizes::THEME_CARD_ASPECT_RATIO,
            theme_id: preset.id.to_string(),
            theme_name: preset.name.to_string(),
            primary: preset.primary,
            secondary: preset.secondary,
            tertiary: preset.tertiary,
            background: preset.background,
            is_active: false,
            is_hovered: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn render(&mut self, mx: f32, my: f32) {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let scale = if self.is_hovered {
            effects::HOVER_SCALE
        } else {
            1.0
        };
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        let offset_x = (scaled_width - self.width) / 2.0;
        let offset_y = (scaled_height - self.height) / 2.0;
        let draw_x = self.x - offset_x;
        let draw_y = self.y - offset_y;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(self.background);
        draw_rectangle(
            draw_x,
            draw_y,
            scaled_width,
            scaled_height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        let preview_height = scaled_height * 0.6;
        let primary_height = preview_height * 0.4;
        let secondary_y = draw_y + primary_height;
        let tertiary_height = preview_height - primary_height;

        let (p_r, p_g, p_b) = colors::to_normalized(self.primary);
        draw_rectangle(
            draw_x,
            draw_y,
            scaled_width * 0.6,
            primary_height,
            Color::new(p_r, p_g, p_b, 0.8),
        );

        let (s_r, s_g, s_b) = colors::to_normalized(self.secondary);
        draw_rectangle(
            draw_x,
            secondary_y,
            scaled_width * 0.4,
            tertiary_height,
            Color::new(s_r, s_g, s_b, 0.6),
        );

        let (t_r, t_g, t_b) = colors::to_normalized(self.tertiary);
        draw_rectangle(
            draw_x + scaled_width * 0.4,
            secondary_y,
            scaled_width * 0.3,
            tertiary_height,
            Color::new(t_r, t_g, t_b, 0.5),
        );

        let label_y = draw_y + scaled_height - 24.0;
        let (on_r, on_g, on_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &self.theme_name,
            draw_x + 8.0,
            label_y + 16.0,
            14.0,
            Color::new(on_r, on_g, on_b, 1.0),
        );

        if self.is_active {
            let (a_r, a_g, a_b) = colors::to_normalized(colors::PRIMARY);
            draw_rectangle_lines(
                draw_x,
                draw_y,
                scaled_width,
                scaled_height,
                2.0,
                Color::new(a_r, a_g, a_b, 1.0),
            );

            draw_rectangle(
                draw_x + scaled_width - 24.0,
                draw_y + 4.0,
                20.0,
                20.0,
                Color::new(a_r, a_g, a_b, 1.0),
            );
            draw_text(
                "✓",
                draw_x + scaled_width - 20.0,
                draw_y + 18.0,
                14.0,
                Color::new(0.0, 0.0, 0.0, 1.0),
            );
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    pub fn was_clicked(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        self.contains_point(mx, my) && mouse_pressed
    }
}
