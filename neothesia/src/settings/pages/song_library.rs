use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;
use crate::song_library::{FilterState, SongEntry, SortPreference};
use crate::ui::components::{
    GlassPanel, Header, NavItem, ProgressBar, Sidebar, SidebarSection, SongCard,
};

struct SongLibraryLayout {
    header_h: f32,
    header_section_y: f32,
    header_section_h: f32,
    bento_y: f32,
    bento_h: f32,
    cards_y: f32,
    footer_h: f32,
}

struct LibraryStats {
    total_songs: u32,
    songs_played: u32,
    avg_accuracy: f32,
    total_practices: u32,
    best_song: Option<(String, f32)>,
}

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

    fn layout(&self) -> SongLibraryLayout {
        let header_h = self.header.height;
        let header_section_y = header_h + spacing::XL;
        let header_section_h = 140.0;
        let bento_y = header_section_y + header_section_h + spacing::LG;
        let bento_h = 180.0;
        let cards_y = bento_y + bento_h + spacing::XL;
        let footer_h = 50.0;
        SongLibraryLayout {
            header_h,
            header_section_y,
            header_section_h,
            bento_y,
            bento_h,
            cards_y,
            footer_h,
        }
    }

    fn rebuild_cards(&mut self) {
        self.song_cards.clear();
        let card_w = 300.0;
        let card_h = 200.0;
        let gap = 24.0;
        let layout = self.layout();
        let content_x = self.sidebar.width + spacing::XL;
        let start_y = layout.cards_y;

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

    fn compute_stats(&self) -> LibraryStats {
        let total_songs = self.songs.len() as u32;
        let songs_played = self.songs.iter().filter(|s| s.play_count > 0).count() as u32;
        let avg_accuracy = {
            let scores: Vec<f32> = self.songs.iter().filter_map(|s| s.best_score).collect();
            if scores.is_empty() {
                0.0
            } else {
                scores.iter().sum::<f32>() / scores.len() as f32
            }
        };
        let total_practices: u32 = self.songs.iter().map(|s| s.play_count).sum();
        let best_song = self
            .songs
            .iter()
            .filter_map(|s| s.best_score.map(|sc| (s, sc)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, sc)| (s.name.clone(), sc));

        LibraryStats {
            total_songs,
            songs_played,
            avg_accuracy,
            total_practices,
            best_song,
        }
    }

    fn render_header_section(&self, stats: &LibraryStats) {
        let layout = self.layout();
        let content_x = self.sidebar.width + spacing::XL;
        let content_width = screen_width() - self.sidebar.width - spacing::XL * 2.0;

        let panel = GlassPanel::new(
            content_x,
            layout.header_section_y,
            content_width,
            layout.header_section_h,
        );
        panel.render();

        let text_x = content_x + spacing::XL;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Piano Repertoire",
            text_x,
            layout.header_section_y + 40.0,
            36.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let played_pct = if stats.total_songs > 0 {
            (stats.songs_played as f32 / stats.total_songs as f32 * 100.0).round() as u32
        } else {
            0
        };
        ply_fonts::draw_body(
            &format!(
                "{} songs • {} practiced ({}%) • {} sessions • {:.0}% avg accuracy",
                stats.total_songs,
                stats.songs_played,
                played_pct,
                stats.total_practices,
                stats.avg_accuracy,
            ),
            text_x,
            layout.header_section_y + 64.0,
            14.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );

        let btn_x = content_x + content_width - 180.0;
        let btn_y = layout.header_section_y + 40.0;
        let btn_w = 160.0;
        let btn_h = 48.0;

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);

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

    fn render_bento_section(&self, stats: &LibraryStats) {
        let layout = self.layout();
        let content_x = self.sidebar.width + spacing::XL;
        let total_width = screen_width() - content_x - spacing::XL * 2.0;
        let panel_w = total_width * 0.7;
        let quick_w = total_width * 0.3 - 24.0;

        let current_panel = GlassPanel::new(content_x, layout.bento_y, panel_w, layout.bento_h);
        current_panel.render();

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
        ply_fonts::draw_label(
            "CURRENT PROGRESS",
            content_x + spacing::XL,
            layout.bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);

        if let Some(song) = &self.last_practiced_song {
            let display_name = if song.name.len() > 30 {
                format!("{}...", &song.name[..27])
            } else {
                song.name.clone()
            };
            ply_fonts::draw_headline(
                &display_name,
                content_x + spacing::XL,
                layout.bento_y + spacing::XL + 36.0,
                20.0,
                Color::new(title_r, title_g, title_b, 1.0),
            );

            let progress = song.best_score.unwrap_or(0.0);
            let play_info = if song.play_count > 0 {
                song.last_played_at
                    .map(|dt| {
                        format!(
                            "Played {} • {} times",
                            format_relative_time(dt),
                            song.play_count
                        )
                    })
                    .unwrap_or_else(|| format!("Played {} times", song.play_count))
            } else {
                "Not yet practiced".to_string()
            };

            ply_fonts::draw_body(
                &format!("{} • {:.0}% accuracy", play_info, progress),
                content_x + spacing::XL,
                layout.bento_y + spacing::XL + 56.0,
                12.0,
                Color::new(desc_r, desc_g, desc_b, 1.0),
            );

            let ring_x = content_x + panel_w - 80.0;
            let ring_y = layout.bento_y + 90.0;

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
                layout.bento_y + spacing::XL + 36.0,
                20.0,
                Color::new(title_r, title_g, title_b, 1.0),
            );
            ply_fonts::draw_body(
                "Select a song to start practicing",
                content_x + spacing::XL,
                layout.bento_y + spacing::XL + 56.0,
                12.0,
                Color::new(desc_r, desc_g, desc_b, 1.0),
            );
        }

        let quick_x = content_x + panel_w + 24.0;
        let quick_panel = GlassPanel::new(quick_x, layout.bento_y, quick_w, layout.bento_h);
        quick_panel.render();

        let (tert_r, tert_g, tert_b) = colors::to_normalized(colors::TERTIARY);
        ply_fonts::draw_label(
            "QUICK START",
            quick_x + spacing::XL,
            layout.bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(tert_r, tert_g, tert_b, 1.0),
        );

        ply_fonts::draw_headline(
            "Daily Drill",
            quick_x + spacing::XL,
            layout.bento_y + spacing::XL + 36.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let badge_x = quick_x + spacing::XL;
        let badge_y = layout.bento_y + spacing::XL + 56.0;

        let drill_type = if stats.songs_played < 3 {
            "Scales"
        } else if stats.avg_accuracy < 70.0 {
            "Accuracy"
        } else {
            "Sight-Read"
        };

        let level = if stats.total_practices < 10 {
            "Beginner"
        } else if stats.total_practices < 50 {
            "Intermediate"
        } else {
            "Advanced"
        };

        for (i, badge) in [drill_type, level].iter().enumerate() {
            let bx = badge_x + i as f32 * 90.0;
            draw_rectangle(
                bx,
                badge_y,
                82.0,
                24.0,
                Color::new(tert_r, tert_g, tert_b, 0.1),
            );
            draw_rectangle_lines(
                bx,
                badge_y,
                82.0,
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

        let drill_desc = if stats.songs_played == 0 {
            "Start with foundational exercises tailored to beginners.".to_string()
        } else {
            format!(
                "{}-min drill based on {} practiced songs ({:.0}% avg accuracy).",
                if stats.total_practices < 20 { 10 } else { 15 },
                stats.songs_played,
                stats.avg_accuracy,
            )
        };

        ply_fonts::draw_body(
            &drill_desc,
            quick_x + spacing::XL,
            badge_y + 50.0,
            10.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );
    }

    fn render_footer(&self) {
        let layout = self.layout();
        let footer_y = screen_height() - layout.footer_h;
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
        mouse_down: bool,
    ) -> SongLibraryInteraction {
        clear_background(Color::new(0.055, 0.055, 0.075, 1.0));

        let layout = self.layout();
        let stats = self.compute_stats();
        let content_top = layout.cards_y;
        let content_bottom = screen_height() - layout.footer_h;

        let mut clicked_song = None;
        self.hovered_song_index = None;

        for (idx, card) in self.song_cards.iter_mut().enumerate() {
            let adjusted_y = card.y - self.scroll_offset;

            if adjusted_y + card.height < content_top || adjusted_y > content_bottom {
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
        self.render_header_section(&stats);
        self.render_bento_section(&stats);

        let mut nav_event = None;
        let header_nav = self.header.render(mx, my, mouse_pressed);
        if let Some(nav) = header_nav {
            match nav {
                NavItem::Library => {}
                NavItem::Practice => nav_event = Some(SongLibraryInteraction::NavigateToPractice),
                NavItem::Settings => nav_event = Some(SongLibraryInteraction::NavigateToSettings),
            }
        }

        let sidebar_section = self.sidebar.render(mx, my, mouse_pressed);
        if let Some(section) = sidebar_section {
            self.sidebar.set_active_section(section);
        }

        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0
            && !self.sidebar.contains_point(mx, my)
            && !self.header.contains_point(mx, my)
        {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 30.0).max(0.0);
            self.rebuild_cards();
        }

        if let Some(ev) = nav_event {
            return ev;
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
