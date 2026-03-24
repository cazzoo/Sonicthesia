use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;
use crate::song_library::{FilterState, SongEntry, SortPreference};
use crate::ui::components::{
    GlassPanel, Header, NavItem, ProgressBar, Sidebar, SidebarSection, SongCard,
};

pub struct SongLibraryPage {
    pub header: Header,
    pub sidebar: Sidebar,
    pub scroll_offset: f32,
    pub songs: Vec<SongEntry>,
    pub song_cards: Vec<SongCard>,
    pub selected_song_index: Option<usize>,
    pub hovered_song_index: Option<usize>,
    pub last_practiced_song: Option<SongEntry>,
}

impl SongLibraryPage {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            sidebar: Sidebar::new(),
            scroll_offset: 0.0,
            songs: Vec::new(),
            song_cards: Vec::new(),
            selected_song_index: None,
            hovered_song_index: None,
            last_practiced_song: None,
        }
    }

    pub fn load_songs(&mut self, songs: Vec<SongEntry>) {
        self.songs = songs;

        if let Some(most_recent) = self
            .songs
            .iter()
            .filter(|s| s.last_played_at.is_some())
            .max_by_key(|s| s.last_played_at)
        {
            self.last_practiced_song = Some(most_recent.clone());
        }

        self.rebuild_cards();
    }

    fn rebuild_cards(&mut self) {
        self.song_cards.clear();
        let card_w = 300.0;
        let card_h = 200.0;
        let gap = 24.0;
        let content_x = self.sidebar.width + spacing::XL;
        let start_y = 300.0;

        let columns = ((screen_width() - content_x - spacing::XL) / (card_w + gap))
            .floor()
            .max(1.0) as usize;

        for (idx, song) in self.songs.iter().enumerate() {
            let row = idx / columns;
            let col = idx % columns;
            let x = content_x + col as f32 * (card_w + gap);
            let y = start_y + row as f32 * (card_h + gap);

            let mut card = SongCard::new(x, y, song.clone());
            if Some(idx) == self.selected_song_index {
                card.is_selected = true;
            }
            self.song_cards.push(card);
        }
    }

    fn render_header_section(&self) {
        let content_x = self.sidebar.width + spacing::XL;
        let content_y = self.header.height + spacing::XL;
        let content_width = screen_width() - self.sidebar.width - spacing::XL * 2.0;

        let panel = GlassPanel::new(content_x, content_y, content_width, 140.0);
        panel.render();

        let text_x = content_x + spacing::XL;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Piano Repertoire",
            text_x,
            content_y + 40.0,
            36.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            &format!(
                "Master your performance through deliberate practice. {} songs available.",
                self.songs.len()
            ),
            text_x,
            content_y + 64.0,
            14.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );

        let btn_x = content_x + content_width - 180.0;
        let btn_y = content_y + 40.0;
        let btn_w = 160.0;
        let btn_h = 48.0;

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
        let (primary_c_r, primary_c_g, primary_c_b) =
            colors::to_normalized(colors::PRIMARY_CONTAINER);

        draw_rectangle(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        ply_fonts::draw_body(
            "▶ Practice Now",
            btn_x + 16.0,
            btn_y + 30.0,
            14.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );
    }

    fn render_bento_section(&self) {
        let content_x = self.sidebar.width + spacing::XL;
        let bento_y = self.header.height + spacing::XL + 160.0;
        let total_width = screen_width() - content_x - spacing::XL * 2.0;
        let panel_w = total_width * 0.7;
        let quick_w = total_width * 0.3 - 24.0;

        let current_panel = GlassPanel::new(content_x, bento_y, panel_w, 180.0);
        current_panel.render();

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
        ply_fonts::draw_label(
            "CURRENT PROGRESS",
            content_x + spacing::XL,
            bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);

        if let Some(song) = &self.last_practiced_song {
            ply_fonts::draw_headline(
                &song.name,
                content_x + spacing::XL,
                bento_y + spacing::XL + 36.0,
                20.0,
                Color::new(title_r, title_g, title_b, 1.0),
            );

            let progress = song.best_score.unwrap_or(0.0);
            let last_practiced = song
                .last_played_at
                .map(|dt| format!("Last practiced {}", format_relative_time(dt)))
                .unwrap_or_else(|| "Never practiced".to_string());

            ply_fonts::draw_body(
                &format!("{} • Accuracy {:.0}%", last_practiced, progress),
                content_x + spacing::XL,
                bento_y + spacing::XL + 56.0,
                12.0,
                Color::new(desc_r, desc_g, desc_b, 1.0),
            );

            let ring_x = content_x + panel_w - 80.0;
            let ring_y = bento_y + 90.0;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            draw_circle(ring_x, ring_y, 40.0, Color::new(bg_r, bg_g, bg_b, 1.0));

            let pct = (progress / 100.0).min(1.0);
            let (arc_r, arc_g, arc_b) = colors::to_normalized(colors::PRIMARY);
            draw_circle_lines(
                ring_x,
                ring_y,
                36.0,
                4.0,
                Color::new(arc_r, arc_g, arc_b, 0.3),
            );

            let angle = -std::f32::consts::FRAC_PI_2 + (std::f32::consts::PI * 2.0 * pct);
            let end_x = ring_x + 36.0 * angle.cos();
            let end_y = ring_y + 36.0 * angle.sin();
            draw_line(
                ring_x,
                ring_y,
                end_x,
                end_y,
                3.0,
                Color::new(arc_r, arc_g, arc_b, 1.0),
            );

            let pct_text = format!("{:.0}%", progress);
            let text_w = measure_text(&pct_text, ply_fonts::body_font(), 14, 1.0).width;
            ply_fonts::draw_body(
                &pct_text,
                ring_x - text_w / 2.0,
                ring_y + 5.0,
                14.0,
                Color::new(primary_r, primary_g, primary_b, 1.0),
            );
        } else {
            ply_fonts::draw_headline(
                "No songs practiced yet",
                content_x + spacing::XL,
                bento_y + spacing::XL + 36.0,
                20.0,
                Color::new(title_r, title_g, title_b, 1.0),
            );
            ply_fonts::draw_body(
                "Select a song to start practicing",
                content_x + spacing::XL,
                bento_y + spacing::XL + 56.0,
                12.0,
                Color::new(desc_r, desc_g, desc_b, 1.0),
            );
        }

        let quick_x = content_x + panel_w + 24.0;
        let quick_panel = GlassPanel::new(quick_x, bento_y, quick_w, 180.0);
        quick_panel.render();

        let (tert_r, tert_g, tert_b) = colors::to_normalized(colors::TERTIARY);
        ply_fonts::draw_label(
            "QUICK START",
            quick_x + spacing::XL,
            bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(tert_r, tert_g, tert_b, 1.0),
        );

        ply_fonts::draw_headline(
            "Daily Drill",
            quick_x + spacing::XL,
            bento_y + spacing::XL + 36.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let badge_x = quick_x + spacing::XL;
        let badge_y = bento_y + spacing::XL + 56.0;

        for (i, badge) in ["Scales", "Arpeggios"].iter().enumerate() {
            let bx = badge_x + i as f32 * 80.0;
            draw_rectangle(
                bx,
                badge_y,
                72.0,
                24.0,
                Color::new(tert_r, tert_g, tert_b, 0.1),
            );
            draw_rectangle_lines(
                bx,
                badge_y,
                72.0,
                24.0,
                1.0,
                Color::new(tert_r, tert_g, tert_b, 0.2),
            );
            ply_fonts::draw_body(
                badge,
                bx + 8.0,
                badge_y + 16.0,
                10.0,
                Color::new(tert_r, tert_g, tert_b, 1.0),
            );
        }

        ply_fonts::draw_body(
            "15-minute technical warmup generated for your skill level.",
            quick_x + spacing::XL,
            badge_y + 50.0,
            10.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );
    }

    fn render_footer(&self) {
        let footer_y = screen_height() - 50.0;
        let content_x = self.sidebar.width + spacing::XL;

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            content_x,
            footer_y,
            screen_width() - content_x,
            1.0,
            Color::new(border_r, border_g, border_b, 0.2),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let (value_r, value_g, value_b) = colors::to_normalized(colors::SECONDARY);

        ply_fonts::draw_label(
            "CONNECTED DEVICE",
            content_x + spacing::XL,
            footer_y + 18.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        ply_fonts::draw_body(
            "Yamaha P-125 [Channel 1]",
            content_x + spacing::XL,
            footer_y + 36.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        ply_fonts::draw_label(
            "SOUNDFONT",
            content_x + 250.0,
            footer_y + 18.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        ply_fonts::draw_body(
            "Obsidian Grand Piano v1.2",
            content_x + 250.0,
            footer_y + 36.0,
            12.0,
            Color::new(0.973, 0.961, 0.992, 1.0),
        );
    }

    pub fn render(
        &mut self,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        _mouse_down: bool,
    ) -> SongLibraryInteraction {
        clear_background(Color::new(0.055, 0.055, 0.075, 1.0));

        let header_nav = self.header.render(mx, my, mouse_pressed);
        if let Some(nav) = header_nav {
            match nav {
                NavItem::Library => {}
                NavItem::Practice => return SongLibraryInteraction::NavigateToPractice,
                NavItem::Settings => return SongLibraryInteraction::NavigateToSettings,
            }
        }

        let sidebar_section = self.sidebar.render(mx, my, mouse_pressed);
        if let Some(section) = sidebar_section {
            match section {
                SidebarSection::MidiLibrary => self.sidebar.set_active_section(section),
                SidebarSection::SongLists => self.sidebar.set_active_section(section),
                SidebarSection::Recordings => self.sidebar.set_active_section(section),
            }
        }

        self.render_header_section();
        self.render_bento_section();

        let mut clicked_song = None;
        self.hovered_song_index = None;

        for (idx, card) in self.song_cards.iter_mut().enumerate() {
            let adjusted_y = card.y - self.scroll_offset;

            if adjusted_y + card.height < self.header.height + 64.0
                || adjusted_y > screen_height() - 60.0
            {
                continue;
            }

            let original_y = card.y;
            card.y = adjusted_y;
            let is_hovered = card.render(mx, my);
            card.y = original_y;

            if is_hovered {
                self.hovered_song_index = Some(idx);
                if mouse_pressed {
                    clicked_song = Some(idx);
                    self.selected_song_index = Some(idx);
                }
            }
        }

        self.render_footer();

        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0
            && !self.sidebar.contains_point(mx, my)
            && !self.header.contains_point(mx, my)
        {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 30.0).max(0.0);
            self.rebuild_cards();
        }

        if let Some(idx) = clicked_song {
            if let Some(song) = self.songs.get(idx) {
                return SongLibraryInteraction::SelectSong(song.clone());
            }
        }

        SongLibraryInteraction::None
    }

    pub fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 30.0).max(0.0);
        self.rebuild_cards();
    }
}

fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else {
        format!("{} weeks ago", duration.num_weeks())
    }
}

pub enum SongLibraryInteraction {
    None,
    SelectSong(SongEntry),
    NavigateToPractice,
    NavigateToSettings,
}
