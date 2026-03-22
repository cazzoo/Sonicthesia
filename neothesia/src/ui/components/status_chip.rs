use macroquad::prelude::*;
use neothesia_core::design::{colors, radius, sizes};

pub enum ChipVariant {
    Primary,
    Secondary,
    Tertiary,
    Success,
    Warning,
    Error,
}

pub struct StatusChip {
    pub x: f32,
    pub y: f32,
    pub label: String,
    pub variant: ChipVariant,
    pub is_hovered: bool,
}

impl StatusChip {
    pub fn new(x: f32, y: f32, label: &str, variant: ChipVariant) -> Self {
        Self {
            x,
            y,
            label: label.to_string(),
            variant,
            is_hovered: false,
        }
    }

    pub fn render(&mut self, mx: f32, my: f32) {
        let (bg_color, text_color) = match &self.variant {
            ChipVariant::Primary => (colors::PRIMARY, colors::BLACK),
            ChipVariant::Secondary => (colors::SECONDARY, colors::BLACK),
            ChipVariant::Tertiary => (colors::TERTIARY, colors::BLACK),
            ChipVariant::Success => ((76, 175, 80), colors::BLACK),
            ChipVariant::Warning => ((255, 193, 7), colors::BLACK),
            ChipVariant::Error => (colors::ERROR, colors::BLACK),
        };

        let text_width = measure_text(&self.label, None, 12, 1.0).width;
        let padding = 12.0;
        let chip_width = text_width + padding * 2.0;
        let chip_height = sizes::CHIP_HEIGHT;

        self.is_hovered =
            mx >= self.x && mx <= self.x + chip_width && my >= self.y && my <= self.y + chip_height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(bg_color);
        let opacity = if self.is_hovered { 0.9 } else { 0.15 };
        draw_rectangle(
            self.x,
            self.y,
            chip_width,
            chip_height,
            Color::new(bg_r, bg_g, bg_b, opacity),
        );

        draw_rectangle(
            self.x,
            self.y,
            chip_width,
            chip_height,
            Color::new(0.0, 0.0, 0.0, 0.0),
        );

        let (text_r, text_g, text_b) = colors::to_normalized(text_color);
        draw_text(
            &self.label,
            self.x + padding,
            self.y + chip_height / 2.0 + 4.0,
            12.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );
    }

    pub fn width(&self) -> f32 {
        let text_width = measure_text(&self.label, None, 12, 1.0).width;
        text_width + 24.0
    }

    pub fn height(&self) -> f32 {
        sizes::CHIP_HEIGHT
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let text_width = measure_text(&self.label, None, 12, 1.0).width;
        let chip_width = text_width + 24.0;
        x >= self.x && x <= self.x + chip_width && y >= self.y && y <= self.y + sizes::CHIP_HEIGHT
    }
}
