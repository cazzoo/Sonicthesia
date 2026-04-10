use macroquad::prelude::*;
use std::time::Instant;

use crate::scene::ply_fonts;
use crate::virtual_resolution::{vh, vw};
use neothesia_core::design::colors;

const TOAST_DURATION_SECS: f64 = 3.0;

pub struct Toast {
    message: String,
    created_at: Instant,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f64() > TOAST_DURATION_SECS
    }

    pub fn render(&self) {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        let alpha = if elapsed < 0.3 {
            elapsed as f32 / 0.3
        } else if elapsed > TOAST_DURATION_SECS - 0.5 {
            ((TOAST_DURATION_SECS - elapsed) as f32 / 0.5).max(0.0)
        } else {
            1.0
        };

        let screen_w = vw();
        let screen_h = vh();

        let text_w = measure_text(&self.message, ply_fonts::body_font(), 14, 1.0).width;
        let padding_x = 20.0;
        let padding_y = 12.0;
        let toast_w = text_w + padding_x * 2.0;
        let toast_h = 40.0;
        let toast_x = (screen_w - toast_w) / 2.0;
        let toast_y = screen_h - toast_h - 80.0;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            toast_x,
            toast_y,
            toast_w,
            toast_h,
            Color::new(bg_r, bg_g, bg_b, 0.9 * alpha),
        );

        let (accent_r, accent_g, accent_b) = colors::to_normalized(colors::TERTIARY);
        draw_rectangle(
            toast_x,
            toast_y,
            3.0,
            toast_h,
            Color::new(accent_r, accent_g, accent_b, alpha),
        );

        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_body(
            &self.message,
            toast_x + padding_x,
            toast_y + padding_y + 4.0,
            14.0,
            Color::new(text_r, text_g, text_b, alpha),
        );
    }
}

pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn show(&mut self, message: impl Into<String>) {
        self.toasts.push(Toast::new(message));
    }

    pub fn update(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn render(&self) {
        for toast in &self.toasts {
            toast.render();
        }
    }
}
