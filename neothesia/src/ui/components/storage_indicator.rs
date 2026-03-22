use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes};

pub struct StorageIndicator {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub is_hovered: bool,
}

impl StorageIndicator {
    pub fn new(x: f32, y: f32, width: f32, height: f32, used_bytes: u64, total_bytes: u64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            used_bytes,
            total_bytes,
            is_hovered: false,
        }
    }

    pub fn render(&mut self, mx: f32, my: f32) {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let track_height = sizes::PROGRESS_BAR_HEIGHT;
        let track_y = self.y + (self.height - track_height) / 2.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            self.x,
            track_y,
            self.width,
            track_height,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let usage_ratio = if self.total_bytes > 0 {
            self.used_bytes as f32 / self.total_bytes as f32
        } else {
            0.0
        };
        let fill_width = self.width * usage_ratio.clamp(0.0, 1.0);

        let fill_color = if usage_ratio > 0.9 {
            colors::ERROR
        } else if usage_ratio > 0.7 {
            colors::TERTIARY
        } else {
            colors::PRIMARY
        };

        let (fill_r, fill_g, fill_b) = colors::to_normalized(fill_color);
        draw_rectangle(
            self.x,
            track_y,
            fill_width,
            track_height,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let used_str = format_bytes(self.used_bytes);
        let total_str = format_bytes(self.total_bytes);
        let label = format!("{} / {}", used_str, total_str);
        let label_y = self.y + self.height - 8.0;
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &label,
            self.x,
            label_y,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
    }

    pub fn usage_ratio(&self) -> f32 {
        if self.total_bytes > 0 {
            self.used_bytes as f32 / self.total_bytes as f32
        } else {
            0.0
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
