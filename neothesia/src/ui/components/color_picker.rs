use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes};

pub struct ColorPicker {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub current_color: (u8, u8, u8),
    pub is_open: bool,
    pub is_hovered: bool,
    pub presets: Vec<(u8, u8, u8)>,
}

impl ColorPicker {
    pub fn new(x: f32, y: f32, current_color: (u8, u8, u8)) -> Self {
        Self {
            x,
            y,
            width: 48.0,
            height: 48.0,
            current_color,
            is_open: false,
            is_hovered: false,
            presets: vec![
                colors::PRIMARY,
                colors::SECONDARY,
                colors::TERTIARY,
                (255, 100, 100),
                (100, 255, 100),
                (100, 100, 255),
                (255, 255, 100),
                (255, 100, 255),
            ],
        }
    }

    pub fn presets(mut self, presets: Vec<(u8, u8, u8)>) -> Self {
        self.presets = presets;
        self
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) -> Option<(u8, u8, u8)> {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (r, g, b) = colors::to_normalized(self.current_color);
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(r, g, b, 1.0),
        );

        let (outline_r, outline_g, outline_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle_lines(
            self.x,
            self.y,
            self.width,
            self.height,
            effects::GHOST_BORDER_WIDTH,
            Color::new(
                outline_r,
                outline_g,
                outline_b,
                effects::GHOST_BORDER_OPACITY,
            ),
        );

        if self.is_hovered && mouse_pressed {
            self.is_open = !self.is_open;
        }

        if self.is_open {
            let popup_width = self.presets.len() as f32 * 36.0 + 16.0;
            let popup_height = 52.0;
            let popup_x = self.x;
            let popup_y = self.y + self.height + 8.0;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            draw_rectangle(
                popup_x,
                popup_y,
                popup_width,
                popup_height,
                Color::new(bg_r, bg_g, bg_b, 0.95),
            );

            let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle_lines(
                popup_x,
                popup_y,
                popup_width,
                popup_height,
                effects::GHOST_BORDER_WIDTH,
                Color::new(border_r, border_g, border_b, effects::GHOST_BORDER_OPACITY),
            );

            for (i, &color) in self.presets.iter().enumerate() {
                let swatch_x = popup_x + 8.0 + i as f32 * 36.0;
                let swatch_y = popup_y + 8.0;
                let swatch_size = 36.0;

                let (sr, sg, sb) = colors::to_normalized(color);
                draw_rectangle(
                    swatch_x,
                    swatch_y,
                    swatch_size,
                    swatch_size,
                    Color::new(sr, sg, sb, 1.0),
                );

                let swatch_hovered = mx >= swatch_x
                    && mx <= swatch_x + swatch_size
                    && my >= swatch_y
                    && my <= swatch_y + swatch_size;
                if swatch_hovered && mouse_pressed {
                    self.current_color = color;
                    self.is_open = false;
                    return Some(color);
                }
            }
        }

        None
    }

    pub fn hex_value(&self) -> String {
        colors::to_hex(self.current_color)
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
