use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;
use crate::song_library::SongEntry;
use crate::ui::components::{
    GlassPanel, Header, ModeSelector, NavItem, SessionConfig, Sidebar, SidebarSection,
};
use crate::virtual_resolution::{vh, vw};
use crate::{DifficultyLevel, HandSelection, PlayMode, SessionConfig as SessionConfigData};

pub struct SongSelectedPage {
    pub header: Header,
    pub sidebar: Sidebar,
    pub song: Option<SongEntry>,
    pub mode_selector: ModeSelector,
    pub session_config: SessionConfig,
    pub scroll_offset: f32,
}

impl SongSelectedPage {
    pub fn new(song: SongEntry) -> Self {
        let content_x = 256.0 + spacing::XL;
        Self {
            header: Header::new(),
            sidebar: Sidebar::new(),
            song: Some(song),
            mode_selector: ModeSelector::new(content_x, 64.0 + 360.0 + spacing::LG + 56.0),
            session_config: SessionConfig::new(
                content_x,
                64.0 + 360.0 + spacing::LG + 56.0 + 56.0 + spacing::LG,
            ),
            scroll_offset: 0.0,
        }
    }

    fn build_session_config(&self) -> SessionConfigData {
        let mode = self.mode_selector.selected_mode;
        let mut config = SessionConfigData::for_mode(mode);
        config.speed = self.session_config.speed;
        config.difficulty = self.session_config.difficulty;
        config.fingering_enabled = self.session_config.fingering_enabled;
        config.hand_selection = self.session_config.hand_selection;
        config.midi_gain = self.session_config.midi_gain;
        config
    }

    fn render_hero_section(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        let content_x = self.sidebar.width;
        let hero_y = self.header.height;
        let hero_w = vw() - content_x;
        let hero_h = 360.0;

        draw_rectangle(
            content_x,
            hero_y,
            hero_w,
            hero_h,
            Color::new(0.1, 0.08, 0.15, 1.0),
        );

        let (gradient_r, gradient_g, gradient_b) = colors::to_normalized(colors::BACKGROUND);
        for i in 0..100 {
            let alpha = i as f32 / 100.0;
            let y = hero_y + hero_h - i as f32 * 3.0;
            draw_rectangle(
                content_x,
                y,
                hero_w,
                3.0,
                Color::new(gradient_r, gradient_g, gradient_b, alpha),
            );
        }

        let back_x = content_x + spacing::XL;
        let back_y = hero_y + spacing::XL + 20.0;
        let is_back_hovered =
            mx >= back_x && mx <= back_x + 140.0 && my >= back_y - 12.0 && my <= back_y + 12.0;

        let (back_r, back_g, back_b) = if is_back_hovered {
            colors::to_normalized(colors::PRIMARY)
        } else {
            colors::to_normalized(colors::ON_SURFACE_VARIANT)
        };

        ply_fonts::draw_body(
            "← Back to Library",
            back_x,
            back_y,
            12.0,
            Color::new(back_r, back_g, back_b, 1.0),
        );

        if is_back_hovered && mouse_pressed {
            return true;
        }

        let badge_y = back_y + 30.0;

        let (tert_r, tert_g, tert_b) = colors::to_normalized(colors::TERTIARY_CONTAINER);
        let badge_text = self
            .song
            .as_ref()
            .map(|s| {
                if s.difficulty > 0 && s.difficulty < 3 {
                    "EASY"
                } else if s.difficulty < 5 {
                    "Medium"
                } else if s.difficulty < 7 {
                    "Hard"
                } else {
                    "Expert"
                }
            })
            .unwrap_or("MASTERPIECE");
        let badge_width = measure_text(badge_text, ply_fonts::body_font(), 10, 1.0).width + 24.0;
        draw_rectangle(
            back_x,
            badge_y,
            badge_width,
            24.0,
            Color::new(tert_r, tert_g, tert_b, 1.0),
        );
        ply_fonts::draw_body(
            badge_text,
            back_x + 12.0,
            badge_y + 16.0,
            10.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );

        let (meta_r, meta_g, meta_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let genre_text = self
            .song
            .as_ref()
            .and_then(|s| s.genre.clone())
            .unwrap_or_else(|| "MIDI".to_string());
        let genre_display = self
            .song
            .as_ref()
            .map(|s| {
                let tracks = s.track_count;
                format!("{} • {} tracks", genre_text, tracks)
            })
            .unwrap_or_else(|| genre_text);
        ply_fonts::draw_body(
            &genre_display,
            back_x + badge_width + 12.0,
            badge_y + 16.0,
            12.0,
            Color::new(meta_r, meta_g, meta_b, 1.0),
        );

        let title_y = badge_y + 50.0;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);

        let song_name = self
            .song
            .as_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("Song");
        ply_fonts::draw_headline(
            song_name,
            back_x,
            title_y + 50.0,
            40.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (artist_r, artist_g, artist_b) = colors::to_normalized(colors::PRIMARY);
        let artist_text = self
            .song
            .as_ref()
            .map(|s| {
                let mut parts = Vec::new();
                if let Some(ref genre) = s.genre {
                    parts.push(genre.clone());
                }
                if s.difficulty > 0 {
                    let stars = (0..s.difficulty).map(|_| "★").collect::<String>();
                    parts.push(format!("Difficulty: {}", stars));
                }
                if parts.is_empty() {
                    "MIDI File".to_string()
                } else {
                    parts.join(" • ")
                }
            })
            .unwrap_or_else(|| "MIDI File".to_string());
        ply_fonts::draw_body(
            &artist_text,
            back_x,
            title_y + 80.0,
            16.0,
            Color::new(artist_r, artist_g, artist_b, 0.8),
        );

        let stats_x = content_x + hero_w - 260.0;
        let stats_y = hero_y + 120.0;

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            stats_x,
            stats_y - 20.0,
            1.0,
            180.0,
            Color::new(border_r, border_g, border_b, 0.2),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE);

        ply_fonts::draw_label(
            "TOTAL NOTES",
            stats_x + 20.0,
            stats_y,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let track_count = self.song.as_ref().map(|s| s.track_count).unwrap_or(0);
        ply_fonts::draw_headline(
            &track_count.to_string(),
            stats_x + 20.0,
            stats_y + 28.0,
            24.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        ply_fonts::draw_label(
            "TRACK COUNT",
            stats_x + 20.0,
            stats_y + 70.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (lh_r, lh_g, lh_b) = colors::to_normalized(colors::PRIMARY);
        let (rh_r, rh_g, rh_b) = colors::to_normalized(colors::SECONDARY);

        let play_count = self.song.as_ref().map(|s| s.play_count).unwrap_or(0);
        ply_fonts::draw_body(
            "Played",
            stats_x + 20.0,
            stats_y + 98.0,
            10.0,
            Color::new(lh_r, lh_g, lh_b, 1.0),
        );
        ply_fonts::draw_headline(
            &play_count.to_string(),
            stats_x + 20.0,
            stats_y + 118.0,
            20.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        let difficulty = self.song.as_ref().map(|s| s.difficulty).unwrap_or(0);
        let diff_stars = (0..difficulty).map(|_| "★").collect::<String>();
        ply_fonts::draw_body(
            "Level",
            stats_x + 80.0,
            stats_y + 98.0,
            10.0,
            Color::new(rh_r, rh_g, rh_b, 1.0),
        );
        ply_fonts::draw_headline(
            &diff_stars,
            stats_x + 80.0,
            stats_y + 118.0,
            20.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        ply_fonts::draw_label(
            "DURATION",
            stats_x + 20.0,
            stats_y + 150.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let duration = self
            .song
            .as_ref()
            .map(|s| {
                let mins = s.duration_secs / 60;
                let secs = s.duration_secs % 60;
                format!("{}:{:02}", mins, secs)
            })
            .unwrap_or_else(|| "5:42".to_string());

        ply_fonts::draw_headline(
            &duration,
            stats_x + 20.0,
            stats_y + 178.0,
            24.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        false
    }

    fn render_mode_title(&self) {
        let content_x = self.sidebar.width + spacing::XL;
        let title_y = self.header.height + 360.0 + spacing::LG;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Select Performance Mode",
            content_x,
            title_y,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "Choose how you want to interact with this composition.",
            content_x,
            title_y + 26.0,
            14.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );
    }

    fn render_cta_section(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        let cta_y = self.session_config.y + self.session_config.height() + spacing::XL;
        let btn_w = 280.0;
        let btn_h = 60.0;
        let btn_x = self.sidebar.width
            + spacing::XL
            + (vw() - self.sidebar.width - spacing::XL * 2.0 - btn_w) / 2.0;

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);

        let is_hovered = mx >= btn_x && mx <= btn_x + btn_w && my >= cta_y && my <= cta_y + btn_h;

        draw_rectangle(
            btn_x,
            cta_y,
            btn_w,
            btn_h,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        if is_hovered {
            draw_rectangle(btn_x, cta_y, btn_w, btn_h, Color::new(1.0, 1.0, 1.0, 0.2));
        }

        let text = "START SESSION  ▶";
        let text_width = measure_text(text, ply_fonts::headline_font(), 18, 1.0).width;
        ply_fonts::draw_headline(
            text,
            btn_x + (btn_w - text_width) / 2.0,
            cta_y + 38.0,
            18.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );

        let (fav_r, fav_g, fav_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let fav_y = cta_y + btn_h + 24.0;
        let fav_text = "❤ Add to Favorites";
        let fav_width = measure_text(fav_text, ply_fonts::body_font(), 14, 1.0).width;
        let fav_x = btn_x + (btn_w - fav_width) / 2.0;

        let is_fav_hovered =
            mx >= fav_x && mx <= fav_x + fav_width && my >= fav_y - 12.0 && my <= fav_y + 8.0;
        let (fav_color_r, fav_color_g, fav_color_b) = if is_fav_hovered {
            colors::to_normalized(colors::ON_SURFACE)
        } else {
            (fav_r, fav_g, fav_b)
        };

        ply_fonts::draw_body(
            fav_text,
            fav_x,
            fav_y,
            14.0,
            Color::new(fav_color_r, fav_color_g, fav_color_b, 1.0),
        );

        is_hovered && mouse_pressed
    }

    pub fn render(
        &mut self,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> SongSelectedInteraction {
        clear_background(Color::new(0.055, 0.055, 0.075, 1.0));

        if self.render_hero_section(mx, my, mouse_pressed) {
            return SongSelectedInteraction::NavigateBack;
        }

        self.render_mode_title();

        if let Some(mode) = self.mode_selector.render(mx, my, mouse_pressed) {
            return SongSelectedInteraction::ModeSelected(mode);
        }

        self.session_config
            .set_mode(self.mode_selector.selected_mode);
        self.session_config.render(mx, my, mouse_pressed);
        self.session_config.handle_speed_drag(mx, my, mouse_down);
        self.session_config.handle_gain_drag(mx, my, mouse_down);

        if self.render_cta_section(mx, my, mouse_pressed) {
            let config = self.build_session_config();
            return SongSelectedInteraction::StartSession { config };
        }

        let header_nav = self.header.render(mx, my, mouse_pressed);
        let sidebar_section = self.sidebar.render(mx, my, mouse_pressed);

        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0
            && !self.sidebar.contains_point(mx, my)
            && !self.header.contains_point(mx, my)
        {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 30.0).max(0.0);
        }

        if let Some(nav) = header_nav {
            match nav {
                NavItem::Back => return SongSelectedInteraction::NavigateBack,
                NavItem::Library => return SongSelectedInteraction::NavigateBack,
                NavItem::FreePlay => return SongSelectedInteraction::NavigateToFreePlay,
                NavItem::Practice => {}
                NavItem::Settings => return SongSelectedInteraction::NavigateToSettings,
            }
        }

        if let Some(section) = sidebar_section {
            match section {
                SidebarSection::MidiLibrary => return SongSelectedInteraction::NavigateBack,
                _ => {}
            }
        }

        SongSelectedInteraction::None
    }
}

pub enum SongSelectedInteraction {
    None,
    NavigateBack,
    NavigateToFreePlay,
    NavigateToSettings,
    ModeSelected(PlayMode),
    StartSession { config: SessionConfigData },
}
