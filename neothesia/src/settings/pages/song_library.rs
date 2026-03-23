use crate::song_library::{FilterState, SongEntry, SortPreference};
use crate::ui::components::{
    GlassPanel, Header, NavItem, ProgressBar, Sidebar, SidebarSection, SongCard,
};
use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

pub struct SongLibraryPage {
    pub header: Header,
    pub sidebar: Sidebar,
    pub scroll_offset: f32,
    pub songs: Vec<SongEntry>,
    pub song_cards: Vec<SongCard>,
    pub selected_song_index: Option<usize>,
    pub hovered_song_index: Option<usize>,
    pub search_query: String,
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
            search_query: String::new(),
        }
    }

    pub fn load_songs(&mut self, songs: Vec<SongEntry>) {
        self.songs = songs;
        self.rebuild_cards();
    }

    fn rebuild_cards(&mut self) {
        self.song_cards.clear();
        let card_w = 300.0;
        let card_h = 200.0;
        let gap = 24.0;
        let content_x = self.sidebar.width + spacing::XL;
        let start_y = 280.0;

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

    fn render_header(&self) {
        let panel = GlassPanel::new(
            self.sidebar.width + spacing::XL,
            spacing::XL,
            screen_width() - self.sidebar.width - spacing::XL * 2.0,
            160.0,
        );
        panel.render();

        let content_x = self.sidebar.width + spacing::XL * 2.0;
        let content_y = spacing::XL + spacing::XL;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Piano Repertoire",
            content_x,
            content_y + 40.0,
            48.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &format!(
                "Master your performance through deliberate practice. {} songs available in your library.",
                self.songs.len()
            ),
            content_x,
            content_y + 72.0,
            14.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );

        let btn_x = screen_width() - spacing::XL * 2.0 - 180.0;
        let btn_y = content_y + 20.0;
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

        draw_text(
            "▶",
            btn_x + 16.0,
            btn_y + 32.0,
            20.0,
            Color::new(primary_c_r, primary_c_g, primary_c_b, 1.0),
        );

        draw_text(
            "Practice Now",
            btn_x + 44.0,
            btn_y + 32.0,
            14.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );
    }

    fn render_bento_section(&self, songs: &[SongEntry]) {
        let content_x = self.sidebar.width + spacing::XL;
        let bento_y = 200.0;
        let panel_w = (screen_width() - content_x - spacing::XL * 2.0 - 24.0) * 0.7;
        let quick_w = (screen_width() - content_x - spacing::XL * 2.0 - 24.0) * 0.3;

        let current_progress_panel = GlassPanel::new(content_x, bento_y, panel_w, 180.0);
        current_progress_panel.render();

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "CURRENT PROGRESS",
            content_x + spacing::XL,
            bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Moonlight Sonata 1st Mvt.",
            content_x + spacing::XL,
            bento_y + spacing::XL + 36.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Last practiced 2 hours ago • Accuracy 84%",
            content_x + spacing::XL,
            bento_y + spacing::XL + 58.0,
            12.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );

        let ring_x = content_x + panel_w - 80.0;
        let ring_y = bento_y + 90.0;
        draw_circle(ring_x, ring_y, 40.0, Color::new(0.145, 0.145, 0.173, 1.0));

        draw_text(
            "80%",
            ring_x - 16.0,
            ring_y + 6.0,
            16.0,
            Color::new(primary_r, primary_g, primary_b, 1.0),
        );

        let quick_x = content_x + panel_w + 24.0;
        let quick_panel = GlassPanel::new(quick_x, bento_y, quick_w, 180.0);
        quick_panel.render();

        let (tert_r, tert_g, tert_b) = colors::to_normalized(colors::TERTIARY);
        draw_text(
            "QUICK START",
            quick_x + spacing::XL,
            bento_y + spacing::XL + 10.0,
            10.0,
            Color::new(tert_r, tert_g, tert_b, 1.0),
        );

        draw_text(
            "Daily Drill",
            quick_x + spacing::XL,
            bento_y + spacing::XL + 36.0,
            20.0,
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
            draw_text(
                badge,
                bx + 8.0,
                badge_y + 16.0,
                10.0,
                Color::new(tert_r, tert_g, tert_b, 1.0),
            );
        }

        draw_text(
            "15-minute technical warmup generated for your current skill level.",
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

        draw_text(
            "CONNECTED DEVICE",
            content_x + spacing::XL,
            footer_y + 20.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        draw_text(
            "Yamaha P-125 [Channel 1]",
            content_x + spacing::XL,
            footer_y + 38.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        draw_text(
            "SOUNDFONT",
            content_x + 250.0,
            footer_y + 20.0,
            10.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        draw_text(
            "Obsidian Grand Piano v1.2",
            content_x + 250.0,
            footer_y + 38.0,
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
                SidebarSection::MidiLibrary => {}
                SidebarSection::SongLists => {}
                SidebarSection::Recordings => {}
            }
        }

        self.render_header();
        self.render_bento_section(&self.songs);

        let mut clicked_song = None;
        self.hovered_song_index = None;

        for (idx, card) in self.song_cards.iter_mut().enumerate() {
            let adjusted_y = card.y - self.scroll_offset;

            if adjusted_y + card.height < self.sidebar.height as f32
                || adjusted_y > screen_height() - 60.0
            {
                continue;
            }

            let display_card = SongCard {
                x: card.x,
                y: adjusted_y,
                width: card.width,
                height: card.height,
                song: card.song.clone(),
                status: card.status,
                progress: card.progress,
                stars: card.stars,
                is_selected: card.is_selected,
                is_hovered: false,
            };

            let orig_card = card;
            orig_card.y = adjusted_y;
            let is_hovered = orig_card.render(mx, my);
            orig_card.y = card.y;

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

pub enum SongLibraryInteraction {
    None,
    SelectSong(SongEntry),
    NavigateToPractice,
    NavigateToSettings,
}
