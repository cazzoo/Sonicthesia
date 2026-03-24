use crate::scene::ply_fonts;
use crate::song_library::SongEntry;
use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use super::progress_bar::ProgressBar;
use super::star_rating::StarRating;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongStatus {
    Learning,
    Active,
    New,
    NotStarted,
    CriticalPractice,
}

impl SongStatus {
    pub fn label(&self) -> &'static str {
        match self {
            SongStatus::Learning => "Learning",
            SongStatus::Active => "Active",
            SongStatus::New => "New",
            SongStatus::NotStarted => "Not Started",
            SongStatus::CriticalPractice => "Critical Practice",
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            SongStatus::Learning => colors::to_normalized(colors::SECONDARY),
            SongStatus::Active => colors::to_normalized(colors::PRIMARY),
            SongStatus::New => colors::to_normalized(colors::TERTIARY),
            SongStatus::NotStarted => colors::to_normalized(colors::OUTLINE),
            SongStatus::CriticalPractice => colors::to_normalized(colors::ERROR),
        }
    }
}

pub struct SongCard {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub song: SongEntry,
    pub status: SongStatus,
    pub progress: f32,
    pub stars: u8,
    pub is_selected: bool,
    pub is_hovered: bool,
}

impl SongCard {
    pub fn new(x: f32, y: f32, song: SongEntry) -> Self {
        let status = Self::calculate_status(&song);
        let progress = Self::calculate_progress(&song);
        let stars = Self::calculate_stars(&song);

        Self {
            x,
            y,
            width: 300.0,
            height: 200.0,
            song,
            status,
            progress,
            stars,
            is_selected: false,
            is_hovered: false,
        }
    }

    fn calculate_status(song: &SongEntry) -> SongStatus {
        if song.play_count == 0 {
            SongStatus::NotStarted
        } else if let Some(score) = song.best_score {
            if score < 50.0 {
                SongStatus::CriticalPractice
            } else if score < 80.0 {
                SongStatus::Learning
            } else {
                SongStatus::Active
            }
        } else {
            SongStatus::New
        }
    }

    fn calculate_progress(song: &SongEntry) -> f32 {
        song.best_score.unwrap_or(0.0) / 100.0
    }

    fn calculate_stars(song: &SongEntry) -> u8 {
        ((song.difficulty as f32 / 10.0) * 5.0).ceil() as u8
    }

    pub fn render(&mut self, mx: f32, my: f32) -> bool {
        self.is_hovered =
            mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        let bg_alpha = if self.is_hovered { 0.8 } else { 0.6 };
        draw_rectangle(
            self.x,
            self.y,
            self.width,
            self.height,
            Color::new(bg_r, bg_g, bg_b, bg_alpha),
        );

        if self.is_hovered || self.is_selected {
            let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
            let border_width = if self.is_selected { 2.0 } else { 1.0 };
            let border_alpha = if self.is_selected { 1.0 } else { 0.2 };
            draw_rectangle_lines(
                self.x,
                self.y,
                self.width,
                self.height,
                border_width,
                Color::new(primary_r, primary_g, primary_b, border_alpha),
            );

            if self.is_selected {
                draw_rectangle(
                    self.x,
                    self.y,
                    self.width,
                    self.height,
                    Color::new(primary_r, primary_g, primary_b, 0.1),
                );
            }
        }

        let icon_x = self.x + spacing::LG;
        let icon_y = self.y + spacing::LG;
        let icon_size = 40.0;

        let (icon_bg_r, icon_bg_g, icon_bg_b) = if self.is_hovered || self.is_selected {
            colors::to_normalized(colors::PRIMARY)
        } else {
            colors::to_normalized(colors::SURFACE_CONTAINER)
        };
        draw_rectangle(
            icon_x,
            icon_y,
            icon_size,
            icon_size,
            Color::new(icon_bg_r, icon_bg_g, icon_bg_b, 0.2),
        );

        let (icon_r, icon_g, icon_b) = if self.is_hovered || self.is_selected {
            colors::to_normalized(colors::PRIMARY)
        } else {
            colors::to_normalized(colors::PRIMARY_DIM)
        };
        ply_fonts::draw_body(
            "♪",
            icon_x + 10.0,
            icon_y + 30.0,
            22.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        let stars_x = self.x + self.width - spacing::LG - 80.0;
        let stars_y = icon_y + 8.0;
        let star_rating = StarRating::new(stars_x, stars_y)
            .rating(self.stars)
            .max_stars(5)
            .star_size(14.0);
        star_rating.render();

        let title_x = self.x + spacing::LG;
        let title_y = icon_y + icon_size + spacing::MD + 14.0;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);

        let display_name = if self.song.name.len() > 28 {
            format!("{}...", &self.song.name[..25])
        } else {
            self.song.name.clone()
        };
        ply_fonts::draw_headline(
            &display_name,
            title_x,
            title_y,
            16.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let meta_y = title_y + 20.0;
        let (meta_r, meta_g, meta_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        let genre = self.song.genre.as_deref().unwrap_or("Classical");
        let duration = format_duration(self.song.duration_secs);
        let meta_text = format!("{} • {} • MIDI", genre, duration);
        ply_fonts::draw_body(
            &meta_text,
            title_x,
            meta_y,
            11.0,
            Color::new(meta_r, meta_g, meta_b, 1.0),
        );

        let status_y = meta_y + 24.0;
        let (status_r, status_g, status_b) = self.status.color();

        draw_circle(
            title_x + 4.0,
            status_y - 4.0,
            4.0,
            Color::new(status_r, status_g, status_b, 1.0),
        );

        ply_fonts::draw_body(
            self.status.label(),
            title_x + 14.0,
            status_y,
            10.0,
            Color::new(status_r, status_g, status_b, 1.0),
        );

        let pct_x = self.x + self.width - spacing::LG - 50.0;
        let pct_text = format!("{:.0}%", self.progress * 100.0);
        ply_fonts::draw_mono(
            &pct_text,
            pct_x,
            status_y,
            10.0,
            Color::new(meta_r, meta_g, meta_b, 1.0),
        );

        let progress_y = status_y + 10.0;
        let progress_bar = ProgressBar::new(title_x, progress_y, self.width - spacing::LG * 2.0)
            .height(4.0)
            .progress(self.progress)
            .color(Color::new(status_r, status_g, status_b, 1.0));
        progress_bar.render();

        self.is_hovered
    }

    pub fn was_clicked(&self, mx: f32, my: f32, mouse_pressed: bool) -> bool {
        self.is_hovered && mouse_pressed
    }
}

fn format_duration(secs: u32) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{}:{:02}", mins, secs)
}
