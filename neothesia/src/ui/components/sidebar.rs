use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    MidiLibrary,
    SongLists,
    Recordings,
}

#[derive(Debug, Clone)]
pub struct SmartPlaylist {
    pub id: String,
    pub label: String,
}

pub struct Sidebar {
    pub active_section: SidebarSection,
    pub smart_playlists: Vec<SmartPlaylist>,
    pub width: f32,
    pub top_offset: f32,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active_section: SidebarSection::MidiLibrary,
            smart_playlists: vec![
                SmartPlaylist {
                    id: "recent".to_string(),
                    label: "Recent".to_string(),
                },
                SmartPlaylist {
                    id: "favorites".to_string(),
                    label: "Favorites".to_string(),
                },
                SmartPlaylist {
                    id: "difficult".to_string(),
                    label: "Difficult Songs".to_string(),
                },
            ],
            width: 256.0,
            top_offset: 64.0,
        }
    }

    pub fn height(&self) -> f32 {
        screen_height() - self.top_offset
    }

    pub fn set_active_section(&mut self, section: SidebarSection) {
        self.active_section = section;
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) -> Option<SidebarSection> {
        let sh = self.height();

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            0.0,
            self.top_offset,
            self.width,
            sh,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            self.width - 1.0,
            self.top_offset,
            1.0,
            sh,
            Color::new(border_r, border_g, border_b, 0.1),
        );

        let mut clicked_section = None;

        let content_x = spacing::LG;
        let mut current_y = self.top_offset + spacing::XL + 20.0;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Library Explorer",
            content_x,
            current_y,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        current_y += 24.0;
        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "Manage MIDI Input",
            content_x,
            current_y,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        current_y += 36.0;

        let sections = [
            (SidebarSection::MidiLibrary, "MIDI Library"),
            (SidebarSection::SongLists, "Song Lists"),
            (SidebarSection::Recordings, "Recordings"),
        ];

        for (section, label) in sections.iter() {
            let is_active = self.active_section == *section;
            let is_hovered =
                mx >= 0.0 && mx <= self.width && my >= current_y && my <= current_y + 44.0;

            if is_active {
                let (active_bg_r, active_bg_g, active_bg_b) =
                    colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
                draw_rectangle(
                    spacing::SM,
                    current_y,
                    self.width - spacing::LG,
                    44.0,
                    Color::new(active_bg_r, active_bg_g, active_bg_b, 1.0),
                );

                let (accent_r, accent_g, accent_b) = colors::to_normalized(colors::TERTIARY);
                draw_rectangle(
                    0.0,
                    current_y,
                    3.0,
                    44.0,
                    Color::new(accent_r, accent_g, accent_b, 1.0),
                );
            } else if is_hovered {
                let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
                draw_rectangle(
                    spacing::SM,
                    current_y,
                    self.width - spacing::LG,
                    44.0,
                    Color::new(hover_r, hover_g, hover_b, 0.5),
                );
            }

            let text_color = if is_active {
                colors::PRIMARY
            } else if is_hovered {
                colors::ON_SURFACE
            } else {
                colors::ON_SURFACE_VARIANT
            };
            let (text_r, text_g, text_b) = colors::to_normalized(text_color);

            ply_fonts::draw_body(
                label,
                content_x + spacing::MD,
                current_y + 28.0,
                14.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            if is_hovered && mouse_pressed {
                clicked_section = Some(*section);
            }

            current_y += 48.0;
        }

        current_y += 24.0;

        let (header_r, header_g, header_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_label(
            "SMART PLAYLISTS",
            content_x,
            current_y,
            10.0,
            Color::new(header_r, header_g, header_b, 0.6),
        );

        current_y += 28.0;

        for playlist in &self.smart_playlists {
            let is_hovered = mx >= content_x
                && mx <= self.width - spacing::LG
                && my >= current_y
                && my <= current_y + 28.0;

            let (text_r, text_g, text_b) = if is_hovered {
                colors::to_normalized(colors::ON_SURFACE)
            } else {
                colors::to_normalized(colors::ON_SURFACE_VARIANT)
            };

            ply_fonts::draw_body(
                &playlist.label,
                content_x + spacing::MD,
                current_y + 18.0,
                13.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            current_y += 32.0;
        }

        clicked_section
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x <= self.width && y >= self.top_offset
    }
}
