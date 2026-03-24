use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Listen,
    Learn,
    Play,
}

impl PlayMode {
    pub fn label(&self) -> &'static str {
        match self {
            PlayMode::Listen => "Listen",
            PlayMode::Learn => "Learn",
            PlayMode::Play => "Play",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PlayMode::Listen => "Experience a cinematic 3D visualizer. Perfect for analysis or background immersion.",
            PlayMode::Learn => "Interactive pedagogy tools. Adjust speed, set loops, and toggle fingering guides.",
            PlayMode::Play => "Full performance tracking. Record your session, track accuracy, and build streaks.",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PlayMode::Listen => "▶",
            PlayMode::Learn => "🎓",
            PlayMode::Play => "🎮",
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            PlayMode::Listen => colors::to_normalized(colors::SECONDARY),
            PlayMode::Learn => colors::to_normalized(colors::PRIMARY),
            PlayMode::Play => colors::to_normalized(colors::TERTIARY),
        }
    }

    pub fn badges(&self) -> &[&str] {
        match self {
            PlayMode::Listen => &["4K Visuals", "Spatial Audio"],
            PlayMode::Learn => &["Falling Notes", "Looping"],
            PlayMode::Play => &["Score HUD", "Game Mode"],
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
            width: 280.0,
            height: 320.0,
            mode,
            is_selected: false,
            is_hovered: false,
        }
    }

    pub fn render(&mut self, mx: f32, my: f32) -> bool {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        if self.is_selected {
            let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
            draw_rectangle(
                self.x,
                self.y,
                self.width,
                self.height,
                Color::new(primary_r, primary_g, primary_b, 0.1),
            );
            draw_rectangle_lines(
                self.x,
                self.y,
                self.width,
                self.height,
                1.0,
                Color::new(primary_r, primary_g, primary_b, 1.0),
            );
        } else if self.is_hovered {
            let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            draw_rectangle(
                self.x,
                self.y,
                self.width,
                self.height,
                Color::new(hover_r, hover_g, hover_b, 1.0),
            );
        }

        let content_x = self.x + spacing::XL;
        let mut current_y = self.y + spacing::XL;

        let (icon_r, icon_g, icon_b) = self.mode.color();
        let icon_bg_size = 48.0;
        draw_rectangle(
            content_x,
            current_y,
            icon_bg_size,
            icon_bg_size,
            Color::new(icon_r, icon_g, icon_b, 0.1),
        );

        ply_fonts::draw_body(
            self.mode.icon(),
            content_x + 12.0,
            current_y + 34.0,
            24.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        current_y += icon_bg_size + spacing::LG;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            self.mode.label(),
            content_x,
            current_y + 18.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        current_y += 40.0;

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);

        let desc = self.mode.description();
        let max_chars = 60;
        let lines: Vec<&str> = if desc.len() > max_chars {
            let mid = desc[..max_chars].rfind(' ').unwrap_or(max_chars);
            vec![&desc[..mid], &desc[mid + 1..]]
        } else {
            vec![desc]
        };

        for (i, line) in lines.iter().enumerate() {
            ply_fonts::draw_body(
                line,
                content_x,
                current_y + i as f32 * 18.0,
                12.0,
                Color::new(desc_r, desc_g, desc_b, 1.0),
            );
        }

        current_y += lines.len() as f32 * 18.0 + 30.0;

        let badges = self.mode.badges();
        let mut badge_x = content_x;
        for badge in badges {
            let badge_width = measure_text(badge, ply_fonts::body_font(), 10, 1.0).width + 24.0;
            let badge_height = 24.0;

            let (badge_bg_r, badge_bg_g, badge_bg_b) =
                colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            draw_rectangle(
                badge_x,
                current_y,
                badge_width,
                badge_height,
                Color::new(badge_bg_r, badge_bg_g, badge_bg_b, 1.0),
            );

            let (badge_border_r, badge_border_g, badge_border_b) =
                colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle_lines(
                badge_x,
                current_y,
                badge_width,
                badge_height,
                1.0,
                Color::new(badge_border_r, badge_border_g, badge_border_b, 0.2),
            );

            let (badge_text_r, badge_text_g, badge_text_b) =
                colors::to_normalized(colors::ON_SURFACE_VARIANT);
            ply_fonts::draw_body(
                badge,
                badge_x + 12.0,
                current_y + 16.0,
                10.0,
                Color::new(badge_text_r, badge_text_g, badge_text_b, 1.0),
            );

            badge_x += badge_width + 8.0;
        }

        self.is_hovered
    }

    pub fn was_clicked(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
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
        let modes = vec![
            ModeCard::new(x, y, PlayMode::Listen),
            ModeCard::new(x + 280.0 + 24.0, y, PlayMode::Learn),
            ModeCard::new(x + (280.0 + 24.0) * 2.0, y, PlayMode::Play),
        ];

        Self {
            x,
            y,
            modes,
            selected_mode: PlayMode::Learn,
            gap: 24.0,
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
        self.modes.len() as f32 * 280.0 + (self.modes.len() - 1) as f32 * self.gap
    }

    pub fn height(&self) -> f32 {
        320.0
    }
}
