use macroquad::prelude::*;
use neothesia_core::design::{colors, radius, sizes, spacing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Midi,
    Audio,
    Themes,
    Folders,
}

impl SettingsTab {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Midi => "MIDI Setup",
            SettingsTab::Audio => "Audio",
            SettingsTab::Themes => "Themes",
            SettingsTab::Folders => "Library Management",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsTab::General => "⚙",
            SettingsTab::Midi => "🎹",
            SettingsTab::Audio => "🔊",
            SettingsTab::Themes => "🎨",
            SettingsTab::Folders => "📚",
        }
    }

    pub fn all() -> Vec<SettingsTab> {
        vec![
            SettingsTab::General,
            SettingsTab::Midi,
            SettingsTab::Audio,
            SettingsTab::Themes,
            SettingsTab::Folders,
        ]
    }
}

pub struct SettingsNav {
    pub current_tab: SettingsTab,
    pub hovered_tab: Option<SettingsTab>,
    pub sidebar_width: f32,
}

impl SettingsNav {
    pub fn new() -> Self {
        Self {
            current_tab: SettingsTab::General,
            hovered_tab: None,
            sidebar_width: sizes::SIDEBAR_WIDTH,
        }
    }

    pub fn render(&mut self, x: f32, y: f32, height: f32, mx: f32, my: f32, mouse_pressed: bool) {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            x,
            y,
            self.sidebar_width,
            height,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle_lines(
            x,
            y,
            self.sidebar_width,
            height,
            1.0,
            Color::new(border_r, border_g, border_b, 0.1),
        );

        let title_y = y + 32.0;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Library Explorer",
            x + spacing::LG,
            title_y,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let subtitle_y = title_y + 20.0;
        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Configure MIDI Input",
            x + spacing::LG,
            subtitle_y,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let mut tab_y = subtitle_y + 32.0;
        let tab_height = 44.0;
        let tab_gap = 4.0;

        self.hovered_tab = None;

        for tab in SettingsTab::all() {
            let is_active = self.current_tab == tab;
            let is_hovered =
                mx >= x && mx <= x + self.sidebar_width && my >= tab_y && my <= tab_y + tab_height;

            if is_hovered {
                self.hovered_tab = Some(tab);
            }

            if is_active {
                let (active_r, active_g, active_b) =
                    colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
                draw_rectangle(
                    x + spacing::SM,
                    tab_y,
                    self.sidebar_width - spacing::LG,
                    tab_height,
                    Color::new(active_r, active_g, active_b, 1.0),
                );

                let (accent_r, accent_g, accent_b) = colors::to_normalized(colors::TERTIARY);
                draw_rectangle(
                    x + spacing::SM,
                    tab_y,
                    4.0,
                    tab_height,
                    Color::new(accent_r, accent_g, accent_b, 1.0),
                );
            } else if is_hovered {
                let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
                draw_rectangle(
                    x + spacing::SM,
                    tab_y,
                    self.sidebar_width - spacing::LG,
                    tab_height,
                    Color::new(hover_r, hover_g, hover_b, 0.5),
                );
            }

            let icon_x = x + spacing::LG + spacing::SM;
            let icon_y = tab_y + tab_height / 2.0 + 5.0;
            let text_color = if is_active {
                colors::PRIMARY
            } else {
                colors::ON_SURFACE_VARIANT
            };
            let (text_r, text_g, text_b) = colors::to_normalized(text_color);
            draw_text(
                tab.icon(),
                icon_x,
                icon_y,
                18.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            let label_x = icon_x + 32.0;
            let label_y = tab_y + tab_height / 2.0 + 5.0;
            draw_text(
                tab.label(),
                label_x,
                label_y,
                14.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            if is_active {
                let (dot_r, dot_g, dot_b) = colors::to_normalized(colors::PRIMARY);
                draw_circle(
                    x + self.sidebar_width - spacing::LG,
                    tab_y + tab_height / 2.0,
                    4.0,
                    Color::new(dot_r, dot_g, dot_b, 1.0),
                );
            }

            if is_hovered && mouse_pressed {
                self.current_tab = tab;
            }

            tab_y += tab_height + tab_gap;
        }
    }

    pub fn handle_click(&mut self) -> Option<SettingsTab> {
        if let Some(tab) = self.hovered_tab {
            self.current_tab = tab;
            Some(tab)
        } else {
            None
        }
    }
}
