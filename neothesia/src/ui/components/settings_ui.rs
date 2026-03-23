use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

pub struct SectionHeader {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub title: String,
}

impl SectionHeader {
    pub fn new(x: f32, y: f32, width: f32, title: &str) -> Self {
        Self {
            x,
            y,
            width,
            title: title.to_string(),
        }
    }

    pub fn render(&self) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &self.title,
            self.x,
            self.y + 24.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let line_x = self.x + measure_text(&self.title, None, 24, 1.0).width + spacing::LG;
        let line_w = self.width - (line_x - self.x);
        let (line_r, line_g, line_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            line_x,
            self.y + 18.0,
            line_w,
            1.0,
            Color::new(line_r, line_g, line_b, 0.2),
        );

        self.y + 48.0
    }
}

pub struct SettingCard {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub subtitle: String,
    pub has_accent: bool,
    pub accent_color: (u8, u8, u8),
}

impl SettingCard {
    pub fn new(x: f32, y: f32, width: f32, title: &str, subtitle: &str) -> Self {
        Self {
            x,
            y,
            width,
            height: 80.0,
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            has_accent: false,
            accent_color: colors::TERTIARY,
        }
    }

    pub fn with_accent(mut self, color: (u8, u8, u8)) -> Self {
        self.has_accent = true;
        self.accent_color = color;
        self
    }

    pub fn render(&self) {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        if self.has_accent {
            let (acc_r, acc_g, acc_b) = colors::to_normalized(self.accent_color);
            draw_rectangle(
                self.x,
                self.y,
                2.0,
                self.height,
                Color::new(acc_r, acc_g, acc_b, 1.0),
            );
        }

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &self.title,
            self.x + spacing::LG,
            self.y + 28.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &self.subtitle,
            self.x + spacing::LG,
            self.y + 48.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );
    }
}

pub struct PrimaryButton {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
}

impl PrimaryButton {
    pub fn new(x: f32, y: f32, width: f32, label: &str) -> Self {
        Self {
            x,
            y,
            width,
            height: 48.0,
            label: label.to_string(),
        }
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        let is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, if is_hovered { 1.0 } else { 0.9 }),
        );

        if is_hovered {
            let (glow_r, glow_g, glow_b) = colors::to_normalized(colors::PRIMARY);
            draw_rectangle(
                self.x - 2.0,
                self.y - 2.0,
                self.width + 4.0,
                self.height + 4.0,
                Color::new(glow_r, glow_g, glow_b, 0.1),
            );
        }

        let label_w = measure_text(&self.label, None, 16, 1.0).width;
        let (text_r, text_g, text_b) = colors::to_normalized(colors::BLACK);
        draw_text(
            &self.label,
            self.x + (self.width - label_w) / 2.0,
            self.y + self.height / 2.0 + 5.0,
            16.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        is_hovered && mouse_pressed
    }
}

pub struct SecondaryButton {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
}

impl SecondaryButton {
    pub fn new(x: f32, y: f32, width: f32, label: &str) -> Self {
        Self {
            x,
            y,
            width,
            height: 48.0,
            label: label.to_string(),
        }
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        let is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        if is_hovered {
            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::PRIMARY);
            draw_rectangle(
                self.x,
                self.y,
                self.width,
                self.height,
                Color::new(bg_r, bg_g, bg_b, 0.1),
            );
        }

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle_lines(
            self.x,
            self.y,
            self.width,
            self.height,
            1.0,
            Color::new(border_r, border_g, border_b, 0.2),
        );

        let label_w = measure_text(&self.label, None, 16, 1.0).width;
        let (text_r, text_g, text_b) = colors::to_normalized(if is_hovered {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE
        });
        draw_text(
            &self.label,
            self.x + (self.width - label_w) / 2.0,
            self.y + self.height / 2.0 + 5.0,
            16.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        is_hovered && mouse_pressed
    }
}

pub struct ToggleSwitch {
    pub x: f32,
    pub y: f32,
    pub label: String,
    pub value: bool,
}

impl ToggleSwitch {
    pub fn new(x: f32, y: f32, label: &str, value: bool) -> Self {
        Self {
            x,
            y,
            label: label.to_string(),
            value,
        }
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool) -> Option<bool> {
        let toggle_w = 40.0;
        let toggle_h = 20.0;
        let thumb_size = 16.0;

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &self.label,
            self.x,
            self.y + 15.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let toggle_x = self.x + 200.0;
        let toggle_y = self.y;
        let is_hovered = mx >= toggle_x
            && mx <= toggle_x + toggle_w
            && my >= toggle_y
            && my <= toggle_y + toggle_h;

        if self.value {
            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::PRIMARY);
            draw_rectangle(
                toggle_x,
                toggle_y,
                toggle_w,
                toggle_h,
                Color::new(bg_r, bg_g, bg_b, 0.2),
            );
            draw_rectangle_lines(
                toggle_x,
                toggle_y,
                toggle_w,
                toggle_h,
                1.0,
                Color::new(bg_r, bg_g, bg_b, 0.3),
            );

            let thumb_x = toggle_x + toggle_w - thumb_size - 2.0;
            let thumb_y = toggle_y + (toggle_h - thumb_size) / 2.0;
            draw_circle(
                thumb_x + thumb_size / 2.0,
                thumb_y + thumb_size / 2.0,
                thumb_size / 2.0,
                Color::new(bg_r, bg_g, bg_b, 1.0),
            );

            let (glow_r, glow_g, glow_b) = colors::to_normalized(colors::PRIMARY);
            draw_circle(
                thumb_x + thumb_size / 2.0,
                thumb_y + thumb_size / 2.0,
                thumb_size / 2.0 + 4.0,
                Color::new(glow_r, glow_g, glow_b, 0.15),
            );
        } else {
            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            draw_rectangle(
                toggle_x,
                toggle_y,
                toggle_w,
                toggle_h,
                Color::new(bg_r, bg_g, bg_b, 1.0),
            );
            let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle_lines(
                toggle_x,
                toggle_y,
                toggle_w,
                toggle_h,
                1.0,
                Color::new(border_r, border_g, border_b, 1.0),
            );

            let thumb_x = toggle_x + 2.0;
            let thumb_y = toggle_y + (toggle_h - thumb_size) / 2.0;
            let (thumb_r, thumb_g, thumb_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_circle(
                thumb_x + thumb_size / 2.0,
                thumb_y + thumb_size / 2.0,
                thumb_size / 2.0,
                Color::new(thumb_r, thumb_g, thumb_b, 1.0),
            );
        }

        if is_hovered && mouse_pressed {
            Some(!self.value)
        } else {
            None
        }
    }
}

pub struct Slider {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub label: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub unit: String,
}

impl Slider {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        label: &str,
        value: f32,
        min: f32,
        max: f32,
        unit: &str,
    ) -> Self {
        Self {
            x,
            y,
            width,
            label: label.to_string(),
            value,
            min,
            max,
            unit: unit.to_string(),
        }
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool, mouse_down: bool) -> Option<f32> {
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &self.label,
            self.x,
            self.y,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let track_h = 4.0;
        let track_y = self.y + 24.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            self.x,
            track_y,
            self.width,
            track_h,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let normalized = (self.value - self.min) / (self.max - self.min);
        let fill_w = self.width * normalized;
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            self.x,
            track_y,
            fill_w,
            track_h,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let thumb_radius = 8.0;
        let thumb_x = self.x + fill_w;
        let thumb_y = track_y + track_h / 2.0;
        draw_circle(
            thumb_x,
            thumb_y,
            thumb_radius,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let is_over_slider = mx >= self.x
            && mx <= self.x + self.width
            && my >= track_y - 10.0
            && my <= track_y + track_h + 10.0;

        if is_over_slider && (mouse_pressed || mouse_down) {
            let new_normalized = ((mx - self.x) / self.width).clamp(0.0, 1.0);
            let new_value = self.min + new_normalized * (self.max - self.min);
            Some(new_value)
        } else {
            None
        }
    }
}

pub struct Dropdown {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    pub value: String,
    pub status: Option<String>,
}

impl Dropdown {
    pub fn new(x: f32, y: f32, width: f32, label: &str, value: &str) -> Self {
        Self {
            x,
            y,
            width,
            height: 64.0,
            label: label.to_string(),
            value: value.to_string(),
            status: None,
        }
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = Some(status.to_string());
        self
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        let is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        if is_hovered {
            let (border_r, border_g, border_b) = colors::to_normalized(colors::SECONDARY);
            draw_rectangle_lines(
                self.x,
                self.y,
                self.width,
                self.height,
                1.0,
                Color::new(border_r, border_g, border_b, 0.3),
            );
        }

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &self.label,
            self.x + spacing::MD,
            self.y + 22.0,
            16.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        if let Some(ref status) = self.status {
            let status_w = measure_text(status, None, 10, 1.0).width;
            let (status_bg_r, status_bg_g, status_bg_b) = colors::to_normalized(colors::TERTIARY);
            draw_rectangle(
                self.x + self.width - status_w - 24.0,
                self.y + 8.0,
                status_w + 16.0,
                20.0,
                Color::new(status_bg_r, status_bg_g, status_bg_b, 0.1),
            );
            let (status_r, status_g, status_b) = colors::to_normalized(colors::TERTIARY);
            draw_text(
                status,
                self.x + self.width - status_w - 16.0,
                self.y + 22.0,
                10.0,
                Color::new(status_r, status_g, status_b, 1.0),
            );
        }

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &self.value,
            self.x + spacing::MD,
            self.y + 44.0,
            14.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        let arrow_x = self.x + self.width - 24.0;
        let (arrow_r, arrow_g, arrow_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "▼",
            arrow_x,
            self.y + 36.0,
            12.0,
            Color::new(arrow_r, arrow_g, arrow_b, 0.5),
        );

        is_hovered && mouse_pressed
    }
}

pub struct ColorPickerRow {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub label: String,
    pub colors: Vec<(u8, u8, u8)>,
    pub selected_index: usize,
}

impl ColorPickerRow {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        label: &str,
        colors: Vec<(u8, u8, u8)>,
        selected: usize,
    ) -> Self {
        Self {
            x,
            y,
            width,
            label: label.to_string(),
            colors,
            selected_index: selected,
        }
    }

    pub fn render(&self, mx: f32, my: f32, mouse_pressed: bool) -> Option<usize> {
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &self.label,
            self.x,
            self.y,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let swatch_size = 32.0;
        let gap = 12.0;
        let mut clicked = None;

        for (i, &color) in self.colors.iter().enumerate() {
            let sx = self.x + i as f32 * (swatch_size + gap);
            let sy = self.y + 12.0;
            let is_hovered =
                mx >= sx && mx <= sx + swatch_size && my >= sy && my <= sy + swatch_size;

            let (r, g, b) = colors::to_normalized(color);
            draw_circle(
                sx + swatch_size / 2.0,
                sy + swatch_size / 2.0,
                swatch_size / 2.0,
                Color::new(r, g, b, 1.0),
            );

            if i == self.selected_index {
                draw_circle_lines(
                    sx + swatch_size / 2.0,
                    sy + swatch_size / 2.0,
                    swatch_size / 2.0 + 2.0,
                    2.0,
                    Color::new(r, g, b, 1.0),
                );
            }

            if is_hovered && mouse_pressed {
                clicked = Some(i);
            }
        }

        clicked
    }
}
