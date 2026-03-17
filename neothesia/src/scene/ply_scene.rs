//! PLY-specific scene implementations for Macroquad rendering
//!
//! This module provides PLY rendering implementations of all scenes,
//! adapted from the WGPU versions to work with MacroquadContext.

use std::time::Duration;
use crate::{
    NeothesiaEvent,
    context_macroquad::MacroquadContext,
    context::Context,
    song::Song,
    song_library::SongRepository,
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
            self.selected_option = (self.selected_option + 1).min(3);
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
                    // Settings
                    return Some(NeothesiaEvent::ShowSettings);
                }
                3 => {
                    return Some(NeothesiaEvent::Exit);
                }
                _ => {}
            }
        }
        
        if is_key_pressed(KeyCode::S) {
            return Some(NeothesiaEvent::ShowSettings);
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
            "Settings",
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

/// PLY Settings Scene - Interactive settings menu with all controls
pub struct PlySettingsScene {
    /// Current scroll offset
    scroll_offset: f32,
    /// Currently hovered section
    hovered_section: Option<String>,
    /// Popup state
    popup: SettingsPopup,
    /// Discovered SoundFont files
    soundfont_files: Vec<std::path::PathBuf>,
    /// Current SoundFont index
    current_soundfont_index: Option<usize>,
    /// Song library directories
    song_directories: Vec<std::path::PathBuf>,
    /// Button areas for click detection
    button_areas: Vec<ButtonArea>,
    /// Toggle areas for click detection
    toggle_areas: Vec<ToggleArea>,
    /// Spin button areas for click detection
    spin_areas: Vec<SpinArea>,
}

/// Button area for click detection
#[derive(Clone)]
struct ButtonArea {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Toggle area for click detection
#[derive(Clone)]
struct ToggleArea {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value: bool,
}

/// Spin button area for click detection
#[derive(Clone)]
struct SpinArea {
    id: String,
    is_plus: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Settings popup state
#[derive(Debug, Clone, PartialEq)]
enum SettingsPopup {
    None,
    OutputSelector,
    InputSelector,
}

impl PlySettingsScene {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            hovered_section: None,
            popup: SettingsPopup::None,
            soundfont_files: Vec::new(),
            current_soundfont_index: None,
            song_directories: Vec::new(),
            button_areas: Vec::new(),
            toggle_areas: Vec::new(),
            spin_areas: Vec::new(),
        }
    }
    
    /// Clear all interactive areas at the start of each frame
    fn clear_areas(&mut self) {
        self.button_areas.clear();
        self.toggle_areas.clear();
        self.spin_areas.clear();
    }
    
    /// Check if a point is inside a rectangle
    fn is_inside(&self, x: f32, y: f32, rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32) -> bool {
        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }
    
    /// Draw a settings row with title, subtitle, and interactive control
    fn draw_settings_row(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        title: &str,
        subtitle: &str,
        is_hovered: bool,
    ) -> bool {
        use macroquad::prelude::*;
        
        // Draw background
        let bg_color = if is_hovered {
            Color::from_rgba(60, 55, 70, 255)
        } else {
            Color::from_rgba(45, 43, 50, 255)
        };
        
        draw_rectangle(x, y, width, height, bg_color);
        
        // Draw title
        draw_text(
            title,
            x + 15.0,
            y + 12.0,
            16.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw subtitle
        draw_text(
            subtitle,
            x + 15.0,
            y + 32.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );
        
        // Check if mouse is over this row
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height
    }
    
    /// Draw a button
    fn draw_button(&self, x: f32, y: f32, width: f32, height: f32, label: &str, is_hovered: bool) -> bool {
        use macroquad::prelude::*;
        
        let bg_color = if is_hovered {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        
        draw_rectangle(x, y, width, height, bg_color);
        draw_rectangle_lines(x, y, width, height, 1.0, Color::from_rgba(100, 100, 100, 255));
        
        // Center text
        let text_width = measure_text(label, None, 14, 1.0).width;
        draw_text(
            label,
            x + (width - text_width) / 2.0,
            y + (height - 14.0) / 2.0 + 10.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Check if mouse is over this button
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height
    }
    
    /// Draw a toggle switch
    fn draw_toggle(&self, x: f32, y: f32, width: f32, height: f32, value: bool, is_hovered: bool) -> bool {
        use macroquad::prelude::*;
        
        let bg_color = if value {
            Color::from_rgba(160, 81, 255, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        
        draw_rectangle(x, y, width, height, bg_color);
        
        // Draw thumb
        let thumb_size = height - 4.0;
        let thumb_x = if value {
            x + width - thumb_size - 2.0
        } else {
            x + 2.0
        };
        
        draw_rectangle(
            thumb_x,
            y + 2.0,
            thumb_size,
            thumb_size,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Check if mouse is over this toggle
        let (mouse_x, mouse_y) = mouse_position();
        mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height
    }
    
    /// Draw spin buttons (plus/minus)
    fn draw_spin_buttons(&self, x: f32, y: f32, size: f32, value: &str, is_hovered_plus: bool, is_hovered_minus: bool) -> (bool, bool) {
        use macroquad::prelude::*;
        
        let gap = 5.0;
        let minus_x = x;
        let plus_x = x + size + gap;
        
        // Draw minus button
        let minus_color = if is_hovered_minus {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        draw_rectangle(minus_x, y, size, size, minus_color);
        draw_text(
            "-",
            minus_x + size / 2.0 - 5.0,
            y + size / 2.0 + 5.0,
            20.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw plus button
        let plus_color = if is_hovered_plus {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        draw_rectangle(plus_x, y, size, size, plus_color);
        draw_text(
            "+",
            plus_x + size / 2.0 - 5.0,
            y + size / 2.0 + 5.0,
            20.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        
        // Draw value
        draw_text(
            value,
            plus_x + size + gap + 5.0,
            y + size / 2.0 + 5.0,
            14.0,
            Color::from_rgba(200, 200, 200, 255),
        );
        
        // Check hover states
        let (mouse_x, mouse_y) = mouse_position();
        let hover_minus = mouse_x >= minus_x && mouse_x <= minus_x + size && mouse_y >= y && mouse_y <= y + size;
        let hover_plus = mouse_x >= plus_x && mouse_x <= plus_x + size && mouse_y >= y && mouse_y <= y + size;
        
        (hover_minus, hover_plus)
    }
    
    /// Handle button click
    fn handle_button_click(&mut self, ctx: &mut MacroquadContext, id: &str) -> Option<NeothesiaEvent> {
        match id {
            "back" => return Some(NeothesiaEvent::MainMenu(None)),
            "output" => self.popup = SettingsPopup::OutputSelector,
            "input" => self.popup = SettingsPopup::InputSelector,
            "add_soundfont_folder" => {
                log::info!("Add SoundFont folder requested");
            }
            "add_song_directory" => {
                log::info!("Add song directory requested");
            }
            _ => {}
        }
        None
    }
    
    /// Handle toggle click
    fn handle_toggle_click(&mut self, ctx: &mut MacroquadContext, id: &str) {
        match id {
            "vertical_guidelines" => {
                ctx.config.set_vertical_guidelines(!ctx.config.vertical_guidelines());
                ctx.config.save();
            }
            "horizontal_guidelines" => {
                ctx.config.set_horizontal_guidelines(!ctx.config.horizontal_guidelines());
                ctx.config.save();
            }
            "glow" => {
                ctx.config.set_glow(!ctx.config.glow());
                ctx.config.save();
            }
            "note_labels" => {
                ctx.config.set_note_labels(!ctx.config.note_labels());
                ctx.config.save();
            }
            _ => {}
        }
    }
    
    /// Handle spin button click
    fn handle_spin_click(&mut self, ctx: &mut MacroquadContext, id: &str, is_plus: bool) {
        match id {
            "range_start" => {
                if is_plus {
                    let v = (ctx.config.piano_range().start() + 1).min(127);
                    if v + 24 < *ctx.config.piano_range().end() {
                        ctx.config.set_piano_range_start(v);
                    }
                } else {
                    ctx.config.set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
                }
                ctx.config.save();
            }
            "range_end" => {
                if is_plus {
                    ctx.config.set_piano_range_end(ctx.config.piano_range().end() + 1);
                } else {
                    let v = ctx.config.piano_range().end().saturating_sub(1);
                    if *ctx.config.piano_range().start() + 24 < v {
                        ctx.config.set_piano_range_end(v);
                    }
                }
                ctx.config.save();
            }
            "audio_gain" => {
                let new_gain = if is_plus {
                    ctx.config.audio_gain() + 0.1
                } else {
                    ctx.config.audio_gain() - 0.1
                };
                ctx.config.set_audio_gain(new_gain);
                ctx.config.save();
            }
            _ => {}
        }
    }
}

impl PlyScene for PlySettingsScene {
    fn update(&mut self, ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;
        
        // Clear interactive areas at the start of each frame
        self.clear_areas();
        
        // Handle keyboard shortcuts
        if is_key_pressed(KeyCode::Escape) {
            if self.popup != SettingsPopup::None {
                self.popup = SettingsPopup::None;
            } else {
                return Some(NeothesiaEvent::MainMenu(None));
            }
        }
        
        // Handle scroll
        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 20.0).max(0.0);
        }
        
        // Handle mouse clicks
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            
            // Collect button clicks first to avoid borrow checker issues
            let mut button_click = None;
            for button in &self.button_areas {
                if self.is_inside(mouse_x, mouse_y, button.x, button.y, button.width, button.height) {
                    button_click = Some(button.id.clone());
                    break;
                }
            }
            if let Some(id) = button_click {
                return self.handle_button_click(ctx, &id);
            }
            
            // Collect toggle clicks first to avoid borrow checker issues
            let mut toggle_click = None;
            for toggle in &self.toggle_areas {
                if self.is_inside(mouse_x, mouse_y, toggle.x, toggle.y, toggle.width, toggle.height) {
                    toggle_click = Some(toggle.id.clone());
                    break;
                }
            }
            if let Some(id) = toggle_click {
                self.handle_toggle_click(ctx, &id);
            }
            
            // Collect spin clicks first to avoid borrow checker issues
            let mut spin_click = None;
            for spin in &self.spin_areas {
                if self.is_inside(mouse_x, mouse_y, spin.x, spin.y, spin.width, spin.height) {
                    spin_click = Some((spin.id.clone(), spin.is_plus));
                    break;
                }
            }
            if let Some((id, is_plus)) = spin_click {
                self.handle_spin_click(ctx, &id, is_plus);
            }
        }
        
        None
    }
    
    fn render(&mut self, ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;
        
        clear_background(Color::from_rgba(30, 30, 35, 255));
        
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        // Draw PLY rendering indicator
        draw_text(
            "🎨 PLY RENDERING ACTIVE - SETTINGS",
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
        
        // Calculate layout
        let margin_x = (screen_w - 650.0).max(0.0) / 2.0;
        let start_y = 60.0;
        let row_height = 55.0;
        let section_gap = 30.0;
        let mut current_y = start_y - self.scroll_offset;
        
        let (mouse_x, mouse_y) = mouse_position();
        
        // OUTPUT SECTION
        draw_text(
            "OUTPUT",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;
        
        let output_binding = ctx.config.output();
        let output = output_binding.as_deref().unwrap_or("None");
        let output_str = output.to_string();
        let output_hovered = self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Output",
            &output_str,
            false,
        );
        current_y += row_height + 10.0;
        
        // Check if synth output is selected
        let output_str = output.to_string();
        let is_synth = output_str.eq_ignore_ascii_case("Synth") || output_str.contains("Synth");
        
        if is_synth {
            // SoundFont selection
            let soundfont_info = if let Some(index) = self.current_soundfont_index {
                if let Some(sf) = self.soundfont_files.get(index) {
                    format!("SoundFont {} of {}", index + 1, self.soundfont_files.len())
                } else {
                    "No SoundFont selected".to_string()
                }
            } else {
                "No SoundFont selected".to_string()
            };
            
            self.draw_settings_row(
                margin_x,
                current_y,
                650.0,
                row_height,
                "SoundFont",
                &soundfont_info,
                false,
            );
            current_y += row_height + 10.0;
            
            // Audio Gain
            let gain = ctx.config.audio_gain();
            self.draw_settings_row(
                margin_x,
                current_y,
                650.0,
                row_height,
                "Audio Gain",
                &format!("{:.1}", gain),
                false,
            );
            current_y += row_height + 10.0;
        }
        
        current_y += section_gap;
        
        // INPUT SECTION
        draw_text(
            "INPUT",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;
        
        let input_binding = ctx.config.input();
        let input = input_binding.as_deref().unwrap_or("None");
        let input_str = input.to_string();
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Input",
            &input_str,
            false,
        );
        current_y += row_height + section_gap;
        
        // NOTE RANGE SECTION
        draw_text(
            "NOTE RANGE",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;
        
        let range = ctx.config.piano_range();
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Start",
            &range.start().to_string(),
            false,
        );
        current_y += row_height + 10.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "End",
            &range.end().to_string(),
            false,
        );
        current_y += row_height + section_gap;
        
        // RENDER SECTION
        draw_text(
            "RENDER",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Vertical Guidelines",
            if ctx.config.vertical_guidelines() { "On" } else { "Off" },
            false,
        );
        current_y += row_height + 10.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Horizontal Guidelines",
            if ctx.config.horizontal_guidelines() { "On" } else { "Off" },
            false,
        );
        current_y += row_height + 10.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Glow",
            if ctx.config.glow() { "On" } else { "Off" },
            false,
        );
        current_y += row_height + 10.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Note Labels",
            if ctx.config.note_labels() { "On" } else { "Off" },
            false,
        );
        current_y += row_height + section_gap;
        
        // SONG LIBRARY SECTION
        draw_text(
            "SONG LIBRARY",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;
        
        let song_count = ctx.song_library_db.song_count().unwrap_or(0);
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Total Songs",
            &song_count.to_string(),
            false,
        );
        current_y += row_height + 10.0;
        
        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Song Directories",
            &format!("{} directories", ctx.config.song_directories().len()),
            false,
        );
        
        // Draw bottom bar
        let bar_y = screen_h - 60.0;
        draw_rectangle(0.0, bar_y, screen_w, 60.0, Color::from_rgba(37, 35, 42, 255));
        
        // Draw back button
        let back_btn_x = 10.0;
        let back_btn_y = bar_y + 10.0;
        let back_btn_w = 80.0;
        let back_btn_h = 40.0;
        
        let (back_mouse_x, back_mouse_y) = mouse_position();
        let back_hovered = back_mouse_x >= back_btn_x && back_mouse_x <= back_btn_x + back_btn_w
            && back_mouse_y >= back_btn_y && back_mouse_y <= back_btn_y + back_btn_h;
        
        self.draw_button(
            back_btn_x,
            back_btn_y,
            back_btn_w,
            back_btn_h,
            "← Back",
            back_hovered,
        );
        
        // Draw instructions
        draw_text(
            "Scroll to navigate • Click settings to modify • ESC to go back",
            screen_w / 2.0 - 200.0,
            bar_y + 25.0,
            14.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}
