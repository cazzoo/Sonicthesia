use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;

use crate::common::PlayMode;

impl PlayMode {
    pub fn short_description(&self) -> &'static str {
        match self {
            PlayMode::Learn => "Note-by-note with guides",
            PlayMode::Practice => "Falling notes at your pace",
            PlayMode::Play => "Full performance scoring",
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            PlayMode::Practice => colors::to_normalized(colors::SECONDARY),
            PlayMode::Learn => colors::to_normalized(colors::PRIMARY),
            PlayMode::Play => colors::to_normalized(colors::TERTIARY),
        }
    }
}

pub struct ModeCard {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub mode: PlayMode,
    pub is_selected: bool,
    pub is_hovered: bool,
}

impl ModeCard {
    pub fn new(x: f32, y: f32, mode: PlayMode) -> Self {
        Self {
            x,
            y,
            width: 180.0,
            height: 56.0,
            mode,
            is_selected: false,
            is_hovered: false,
        }
    }

    pub fn render(&mut self, mx: f32, my: f32) -> bool {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (bg_r, bg_g, bg_b) = if self.is_selected {
            let (r, g, b) = self.mode.color();
            (r * 0.15, g * 0.15, b * 0.15)
        } else if self.is_hovered {
            colors::to_normalized(colors::SURFACE_CONTAINER_HIGH)
        } else {
            colors::to_normalized(colors::SURFACE_CONTAINER)
        };
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        if self.is_selected {
            let (pr, pg, pb) = self.mode.color();
            draw_rectangle(
                self.x,
                self.y,
                3.0,
                self.height,
                Color::new(pr, pg, pb, 1.0),
            );
            draw_rectangle_lines(
                self.x,
                self.y,
                self.width,
                self.height,
                1.0,
                Color::new(pr, pg, pb, 0.6),
            );
        }

        let cx = self.x + 14.0;
        let icon_y = self.y + self.height / 2.0 - 10.0;

        let (icon_r, icon_g, icon_b) = self.mode.color();
        ply_fonts::draw_body(
            self.mode.icon(),
            cx,
            icon_y + 10.0,
            20.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        let text_x = cx + 32.0;
        let (title_r, title_g, title_b) = if self.is_selected {
            self.mode.color()
        } else {
            colors::to_normalized(colors::ON_SURFACE)
        };
        ply_fonts::draw_body(
            self.mode.label(),
            text_x,
            self.y + 18.0,
            14.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let desc_alpha = if self.is_selected { 0.9 } else { 0.6 };
        ply_fonts::draw_body(
            self.mode.short_description(),
            text_x,
            self.y + 34.0,
            10.0,
            Color::new(desc_r, desc_g, desc_b, desc_alpha),
        );

        self.is_hovered
    }

    pub fn was_clicked(&self, _mx: f32, _my: f32, mouse_pressed: bool) -> bool {
        self.is_hovered && mouse_pressed
    }
}

pub struct ModeSelector {
    pub x: f32,
    pub y: f32,
    pub modes: Vec<ModeCard>,
    pub selected_mode: PlayMode,
    pub gap: f32,
}

impl ModeSelector {
    pub fn new(x: f32, y: f32) -> Self {
        let card_w = 180.0;
        let gap = 12.0;
        let modes = vec![
            ModeCard::new(x, y, PlayMode::Learn),
            ModeCard::new(x + card_w + gap, y, PlayMode::Practice),
            ModeCard::new(x + (card_w + gap) * 2.0, y, PlayMode::Play),
        ];

        Self {
            x,
            y,
            modes,
            selected_mode: PlayMode::Learn,
            gap,
        }
    }

    pub fn selected_mode(mut self, mode: PlayMode) -> Self {
        self.selected_mode = mode;
        self
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) -> Option<PlayMode> {
        let mut clicked_mode = None;

        for mode_card in &mut self.modes {
            mode_card.is_selected = mode_card.mode == self.selected_mode;
            if mode_card.render(mx, my) && mode_card.was_clicked(mx, my, mouse_pressed) {
                clicked_mode = Some(mode_card.mode);
                self.selected_mode = mode_card.mode;
            }
        }

        clicked_mode
    }

    pub fn width(&self) -> f32 {
        self.modes.len() as f32 * 180.0 + (self.modes.len() - 1) as f32 * self.gap
    }

    pub fn height(&self) -> f32 {
        56.0
    }
}
