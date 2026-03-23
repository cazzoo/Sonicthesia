use macroquad::prelude::*;
use neothesia_core::design::{colors, radius, sizes};

pub struct ProgressBar {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub progress: f32,
    pub color: Color,
    pub show_percentage: bool,
}

impl ProgressBar {
    pub fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            x,
            y,
            width,
            height: 6.0,
            progress: 0.0,
            color: Color::new(0.373, 0.620, 1.0, 1.0),
            show_percentage: false,
        }
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn primary_color(mut self) -> Self {
        let (r, g, b) = colors::to_normalized(colors::PRIMARY);
        self.color = Color::new(r, g, b, 1.0);
        self
    }

    pub fn secondary_color(mut self) -> Self {
        let (r, g, b) = colors::to_normalized(colors::SECONDARY);
        self.color = Color::new(r, g, b, 1.0);
        self
    }

    pub fn tertiary_color(mut self) -> Self {
        let (r, g, b) = colors::to_normalized(colors::TERTIARY);
        self.color = Color::new(r, g, b, 1.0);
        self
    }

    pub fn error_color(mut self) -> Self {
        let (r, g, b) = colors::to_normalized(colors::ERROR);
        self.color = Color::new(r, g, b, 1.0);
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn render(&self) {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);

        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        let fill_width = self.width * self.progress;
        if fill_width > 0.0 {
            draw_rectangle(self.x, self.y, fill_width, self.height, self.color);
        }
    }

    pub fn render_with_label(&self, label: &str) {
        self.render();

        if self.show_percentage {
            let pct_text = format!("{:.0}%", self.progress * 100.0);
            draw_text(
                &pct_text,
                self.x + self.width + 8.0,
                self.y + self.height - 2.0,
                10.0,
                Color::new(0.667, 0.655, 0.694, 1.0),
            );
        }
    }
}

pub struct ProgressRing {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub progress: f32,
    pub stroke_width: f32,
    pub color: Color,
}

impl ProgressRing {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Self {
            x,
            y,
            radius,
            progress: 0.0,
            stroke_width: 4.0,
            color: Color::new(0.859, 0.565, 1.0, 1.0),
        }
    }

    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn render(&self) {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);

        let segments = 64;
        let angle_step = std::f32::consts::PI * 2.0 / segments as f32;
        let filled_segments = (self.progress * segments as f32) as usize;

        for i in 0..segments {
            let angle = i as f32 * angle_step - std::f32::consts::FRAC_PI_2;
            let next_angle = (i + 1) as f32 * angle_step - std::f32::consts::FRAC_PI_2;

            let color = if i < filled_segments {
                self.color
            } else {
                Color::new(bg_r, bg_g, bg_b, 1.0)
            };

            let inner_r = self.radius - self.stroke_width;
            let outer_r = self.radius;

            let points: Vec<(f32, f32)> = vec![
                (
                    self.x + inner_r * angle.cos(),
                    self.y + inner_r * angle.sin(),
                ),
                (
                    self.x + outer_r * angle.cos(),
                    self.y + outer_r * angle.sin(),
                ),
                (
                    self.x + outer_r * next_angle.cos(),
                    self.y + outer_r * next_angle.sin(),
                ),
                (
                    self.x + inner_r * next_angle.cos(),
                    self.y + inner_r * next_angle.sin(),
                ),
            ];

            for j in 0..points.len() {
                let (x1, y1) = points[j];
                let (x2, y2) = points[(j + 1) % points.len()];
                draw_line(x1, y1, x2, y2, 2.0, color);
            }
        }
    }

    pub fn render_with_text(&self, text: &str) {
        self.render();

        let text_width = measure_text(text, None, 12, 1.0).width;
        draw_text(
            text,
            self.x - text_width / 2.0,
            self.y + 4.0,
            12.0,
            Color::new(0.973, 0.961, 0.992, 1.0),
        );
    }
}
