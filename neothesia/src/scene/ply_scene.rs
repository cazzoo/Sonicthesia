//! PLY-specific scene implementations for Macroquad rendering
//!
//! This module provides PLY rendering implementations of all scenes,
//! adapted from the WGPU versions to work with MacroquadContext.

use std::time::Duration;
use crate::{
    NeothesiaEvent,
    context_macroquad::MacroquadContext,
    song::Song,
};

/// PLY-specific scene trait
pub trait PlyScene {
    /// Update the scene logic
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent>;
    
    /// Render the scene using PLY/Macroquad
    fn render(&mut self, ctx: &mut MacroquadContext);
}

/// PLY Menu Scene
pub struct PlyMenuScene {
    song: Option<Song>,
    selected_option: usize,
}

impl PlyMenuScene {
    pub fn new(song: Option<Song>) -> Self {
        Self {
            song,
            selected_option: 0,
        }
    }
}

impl PlyScene for PlyMenuScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;
        
        // Handle keyboard input
        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::Exit);
        }
        
        if is_key_pressed(KeyCode::Up) {
            self.selected_option = self.selected_option.saturating_sub(1);
        }
        
        if is_key_pressed(KeyCode::Down) {
            self.selected_option = (self.selected_option + 1).min(2);
        }
        
        if is_key_pressed(KeyCode::Enter) {
            match self.selected_option {
                0 => {
                    if let Some(song) = self.song.take() {
                        return Some(NeothesiaEvent::Play(song));
                    }
                }
                1 => {
                    // Free play mode
                    return Some(NeothesiaEvent::FreePlay(self.song.take()));
                }
                2 => {
                    return Some(NeothesiaEvent::Exit);
                }
                _ => {}
            }
        }
        
        None
    }
    
    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;
        
        clear_background(BLACK);
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        
        // Draw PLY rendering indicator
        draw_text(
            "🎨 PLY RENDERING ACTIVE",
            10.0,
            10.0,
            18.0,
            Color::from_rgba(0, 255, 0, 255),
        );
        
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            35.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw title
        draw_text(
            "NEOTHESIA",
            center_x - 100.0,
            center_y - 150.0,
            50.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw song info if available
        if let Some(song) = &self.song {
            draw_text(
                &format!("Song: {}", song.file.name),
                center_x - 150.0,
                center_y - 80.0,
                20.0,
                Color::from_rgba(200, 200, 255, 255),
            );
        } else {
            draw_text(
                "No song loaded",
                center_x - 80.0,
                center_y - 80.0,
                20.0,
                Color::from_rgba(255, 100, 100, 255),
            );
            draw_text(
                "Use --midi-file <path> to load a song",
                center_x - 200.0,
                center_y - 50.0,
                14.0,
                Color::from_rgba(150, 150, 150, 255),
            );
        }
        
        // Draw menu options
        let options = vec![
            "Play Song",
            "Free Play",
            "Exit",
        ];
        
        let start_y = center_y;
        for (i, option) in options.iter().enumerate() {
            let color = if i == self.selected_option {
                Color::from_rgba(100, 200, 100, 255)
            } else {
                Color::from_rgba(150, 150, 150, 255)
            };
            
            let prefix = if i == self.selected_option { "> " } else { "  " };
            
            draw_text(
                &format!("{}{}", prefix, option),
                center_x - 80.0,
                start_y + (i as f32 * 40.0),
                24.0,
                color,
            );
        }
        
        // Draw instructions
        draw_text(
            "Use UP/DOWN to select, ENTER to choose",
            center_x - 180.0,
            screen_h - 50.0,
            14.0,
            Color::from_rgba(100, 100, 100, 255),
        );
    }
}

/// PLY Playing Scene
pub struct PlyPlayingScene {
    song: Song,
    paused: bool,
}

impl PlyPlayingScene {
    pub fn new(song: Song) -> Self {
        Self {
            song,
            paused: false,
        }
    }
}

impl PlyScene for PlyPlayingScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;
        
        // Handle keyboard input
        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::MainMenu(None));
        }
        
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        
        None
    }
    
    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;
        
        clear_background(BLACK);
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        
        // Draw PLY rendering indicator
        draw_text(
            "🎨 PLY RENDERING ACTIVE - PLAYING",
            10.0,
            10.0,
            18.0,
            Color::from_rgba(0, 255, 0, 255),
        );
        
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            35.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw song info
        draw_text(
            &format!("Playing: {}", self.song.file.name),
            center_x - 150.0,
            center_y - 100.0,
            24.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw status
        if self.paused {
            draw_text(
                "PAUSED",
                center_x - 40.0,
                center_y,
                40.0,
                Color::from_rgba(255, 200, 0, 255),
            );
        } else {
            draw_text(
                "Playing...",
                center_x - 60.0,
                center_y,
                30.0,
                Color::from_rgba(100, 255, 100, 255),
            );
        }
        
        // Draw instructions
        draw_text(
            "SPACE: Pause/Resume | ESC: Return to menu",
            center_x - 200.0,
            screen_h - 50.0,
            14.0,
            Color::from_rgba(100, 100, 100, 255),
        );
    }
}

/// PLY Freeplay Scene
pub struct PlyFreeplayScene {
    song: Option<Song>,
}

impl PlyFreeplayScene {
    pub fn new(song: Option<Song>) -> Self {
        Self { song }
    }
}

impl PlyScene for PlyFreeplayScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;
        
        // Handle keyboard input
        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::MainMenu(None));
        }
        
        None
    }
    
    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;
        
        clear_background(BLACK);
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        
        // Draw PLY rendering indicator
        draw_text(
            "🎨 PLY RENDERING ACTIVE - FREEPLAY",
            10.0,
            10.0,
            18.0,
            Color::from_rgba(0, 255, 0, 255),
        );
        
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            35.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw title
        draw_text(
            "FREE PLAY MODE",
            center_x - 120.0,
            center_y - 50.0,
            30.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw song info if available
        if let Some(song) = &self.song {
            draw_text(
                &format!("Song: {}", song.file.name),
                center_x - 150.0,
                center_y + 20.0,
                20.0,
                Color::from_rgba(200, 200, 255, 255),
            );
        }
        
        // Draw instructions
        draw_text(
            "Use your MIDI keyboard or computer keyboard to play",
            center_x - 250.0,
            center_y + 80.0,
            16.0,
            Color::from_rgba(150, 150, 150, 255),
        );
        
        draw_text(
            "ESC: Return to menu",
            center_x - 100.0,
            screen_h - 50.0,
            14.0,
            Color::from_rgba(100, 100, 100, 255),
        );
    }
}

/// PLY Score Scene
pub struct PlyScoreScene {
    song: Song,
    score_data: crate::scene::playing_scene::midi_player::ScoreData,
}

impl PlyScoreScene {
    pub fn new(song: Song, score_data: crate::scene::playing_scene::midi_player::ScoreData) -> Self {
        Self {
            song,
            score_data,
        }
    }
}

impl PlyScene for PlyScoreScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;
        
        // Handle keyboard input
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(NeothesiaEvent::MainMenu(None));
        }
        
        None
    }
    
    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;
        
        clear_background(BLACK);
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        
        // Draw PLY rendering indicator
        draw_text(
            "🎨 PLY RENDERING ACTIVE - SCORE",
            10.0,
            10.0,
            18.0,
            Color::from_rgba(0, 255, 0, 255),
        );
        
        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            35.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw title
        draw_text(
            "SONG COMPLETE!",
            center_x - 100.0,
            center_y - 150.0,
            40.0,
            Color::from_rgba(100, 255, 100, 255),
        );
        
        // Draw song info
        draw_text(
            &format!("Song: {}", self.song.file.name),
            center_x - 150.0,
            center_y - 80.0,
            24.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw score
        let score = if self.score_data.total_notes > 0 {
            (self.score_data.correct_notes as f32 / self.score_data.total_notes as f32) * 100.0
        } else {
            0.0
        };
        
        draw_text(
            &format!("Score: {:.0}%", score),
            center_x - 80.0,
            center_y,
            30.0,
            Color::from_rgba(255, 200, 0, 255),
        );
        
        // Draw accuracy
        if self.score_data.total_notes > 0 {
            let accuracy = (self.score_data.correct_notes as f32 / self.score_data.total_notes as f32) * 100.0;
            draw_text(
                &format!("Accuracy: {:.1}%", accuracy),
                center_x - 100.0,
                center_y + 50.0,
                24.0,
                Color::from_rgba(100, 200, 255, 255),
            );
        }
        
        // Draw additional stats
        draw_text(
            &format!("Correct: {} | Missed: {}", self.score_data.correct_notes, self.score_data.missed_notes),
            center_x - 150.0,
            center_y + 90.0,
            16.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        
        // Draw instructions
        draw_text(
            "Press ENTER or ESC to return to menu",
            center_x - 180.0,
            screen_h - 50.0,
            14.0,
            Color::from_rgba(100, 100, 100, 255),
        );
    }
}
