use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes};

pub struct GlassPanel {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub opacity: f32,
    pub blur: f32,
}

impl GlassPanel {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: radius::LG,
            opacity: effects::GLASS_OPACITY,
            blur: effects::GLASS_BLUR,
        }
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn render(&self) {
        let surface = colors::SURFACE_CONTAINER_HIGHEST;
        let (r, g, b) = colors::to_normalized(surface);
        let color = Color::new(r, g, b, self.opacity);

        draw_rectangle(self.x, self.y, self.width, self.height, color);

        let outline = colors::OUTLINE_VARIANT;
        let (r, g, b) = colors::to_normalized(outline);
        let border_color = Color::new(r, g, b, effects::GHOST_BORDER_OPACITY);
        draw_rectangle_lines(
            self.x,
            self.y,
            self.width,
            self.height,
            effects::GHOST_BORDER_WIDTH,
            border_color,
        );
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
