//! PLY-specific scene implementations for Macroquad rendering
//!
//! This module provides PLY rendering implementations of all scenes,
//! adapted from the WGPU versions to work with MacroquadContext.

use crate::{
    context_macroquad::MacroquadContext,
    effects::{EffectsManager, ScreenFlash, ScreenShake},
    scoring::{LiveScoreTracker, StreakMilestone, TimingQuality},
    settings::SettingsPage,
    song::Song,
    song_library::SongRepository,
    NeothesiaEvent,
};
use std::time::Duration;

use crate::input_stubs::{ElementType, FocusableElement, InputPriority, UnifiedInputManager};
use crate::render::ply::PianoKeyboardRenderer;
use crate::ui::components::{Header, NavItem};
use crate::virtual_resolution::{vh, vmouse, vw};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, spacing};
use piano_layout::KeyboardLayout;

const SIDEBAR_WIDTH: f32 = 256.0;
fn song_entry_from_song(song: &Song) -> crate::song_library::SongEntry {
    crate::song_library::SongEntry {
        id: song.song_id.unwrap_or(0),
        file_path: std::path::PathBuf::from(&song.file.name),
        name: song.file.name.clone(),
        difficulty: 5,
        duration_secs: 300,
        track_count: song.file.tracks.len(),
        play_count: 0,
        last_score: None,
        best_score: None,
        last_played_at: None,
        created_at: chrono::Utc::now(),
        genre: Some("Classical".to_string()),
        labels: Vec::new(),
    }
}

/// PLY-specific scene trait
pub trait PlyScene {
    /// Update the scene logic
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent>;

    /// Render the scene using PLY/Macroquad
    fn render(&mut self, ctx: &mut MacroquadContext);

    /// Handle MIDI input events (for visual feedback on virtual piano)
    fn handle_midi_event(&mut self, _channel: u8, _message: &midi_file::midly::MidiMessage) {
        // Default implementation does nothing
        // Scenes can override this to show visual feedback
    }
}

/// PLY Menu Scene
pub struct PlyMenuScene {
    song: Option<Song>,
    input_manager: UnifiedInputManager,
}

impl PlyMenuScene {
    pub fn new(song: Option<Song>) -> Self {
        let mut input_manager = UnifiedInputManager::new();

        // Initialize cursor visibility callback
        input_manager
            .focus()
            .priority()
            .set_cursor_visibility_callback(Box::new(|visible| {
                if visible {
                    macroquad::input::show_mouse(true);
                } else {
                    macroquad::input::show_mouse(false);
                }
            }));

        // Register menu options as focusable elements
        // Options: Play Song (only if song), Free Play, Song Library, Settings, Exit
        let menu_options = if song.is_some() {
            vec![
                ("menu_play".to_string(), "Play Song".to_string()),
                ("menu_freeplay".to_string(), "Free Play".to_string()),
                ("menu_library".to_string(), "Song Library".to_string()),
                ("menu_settings".to_string(), "Settings".to_string()),
                ("menu_exit".to_string(), "Exit".to_string()),
            ]
        } else {
            vec![
                ("menu_freeplay".to_string(), "Free Play".to_string()),
                ("menu_library".to_string(), "Song Library".to_string()),
                ("menu_settings".to_string(), "Settings".to_string()),
                ("menu_exit".to_string(), "Exit".to_string()),
            ]
        };

        for (i, (id, label)) in menu_options.into_iter().enumerate() {
            input_manager.focus().register_element(FocusableElement {
                id,
                label,
                element_type: ElementType::Button,
                position: (0.0, 0.0), // Will be updated in render()
                size: (200.0, 40.0),
                focusable: true,
            });
        }

        // Focus first element initially
        let first_id = input_manager
            .focus()
            .elements()
            .first()
            .map(|e| e.id.clone());
        if let Some(id) = first_id {
            input_manager.focus().set_focus(&id);
        }

        Self {
            song,
            input_manager,
        }
    }
}

impl PlyMenuScene {
    /// Get the index of the currently focused option
    fn focused_index(&self) -> Option<usize> {
        // Use the immutable accessor from UnifiedInputManager
        self.input_manager.focused_index()
    }

    /// Activate the currently focused option
    fn activate_focused(&mut self, ctx: &mut MacroquadContext) -> Option<NeothesiaEvent> {
        if let Some(focused) = self.input_manager.focus().focused_element() {
            let has_song = self.song.is_some();

            match focused.id.as_str() {
                "menu_play" => {
                    if has_song {
                        if let Some(song) = self.song.take() {
                            // Check if we should resume playback
                            if let Some(resume_time) = ctx.resume_playback_time.take() {
                                return Some(NeothesiaEvent::ResumePlay(song, resume_time));
                            }
                            return Some(NeothesiaEvent::StartSession {
                                song,
                                config: crate::common::SessionConfig::default(),
                            });
                        }
                    }
                }
                "menu_freeplay" => {
                    ctx.resume_playback_time = None;
                    return Some(NeothesiaEvent::FreePlay(self.song.take()));
                }
                "menu_library" => {
                    ctx.resume_playback_time = None;
                    return Some(NeothesiaEvent::ShowSongLibrary(self.song.take()));
                }
                "menu_settings" => {
                    return Some(NeothesiaEvent::ShowSettings);
                }
                "menu_exit" => {
                    return Some(NeothesiaEvent::Exit);
                }
                _ => {}
            }
        }
        None
    }
}

impl PlyScene for PlyMenuScene {
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        // Update unified input manager
        self.input_manager.update(delta.as_secs_f64());

        // Handle keyboard input - set keyboard priority when navigating
        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::Exit);
        }

        if is_key_pressed(KeyCode::Up) {
            // Set keyboard priority when using arrow keys
            self.input_manager
                .focus()
                .priority()
                .set_keyboard_priority();
            self.input_manager.focus().focus_previous();
        }

        if is_key_pressed(KeyCode::Down) {
            // Set keyboard priority when using arrow keys
            self.input_manager
                .focus()
                .priority()
                .set_keyboard_priority();
            self.input_manager.focus().focus_next();
        }

        if is_key_pressed(KeyCode::Enter) {
            // Set keyboard priority when activating
            self.input_manager
                .focus()
                .priority()
                .set_keyboard_priority();
            return self.activate_focused(ctx);
        }

        if is_key_pressed(KeyCode::S) {
            return Some(NeothesiaEvent::ShowSettings);
        }

        // Handle mouse hover with unified input system
        let (mouse_x, mouse_y) = vmouse();

        // Update mouse position in unified input manager
        self.input_manager
            .focus()
            .priority()
            .update_mouse_position(mouse_x, mouse_y);

        let screen_w = vw();
        let screen_h = vh();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;

        // Menu option positions (must match render())
        let start_y = center_y;
        let option_height = 40.0;
        let menu_start_x = center_x - 80.0;

        // Check if a song is loaded
        let has_song = self.song.is_some();

        // Check if mouse is over any menu option - update focus based on input priority
        if self.input_manager.focus().priority().has_mouse_priority() {
            if mouse_x >= menu_start_x && mouse_x <= menu_start_x + 200.0 {
                // Get menu option IDs based on whether song is loaded
                let option_ids: Vec<&str> = if has_song {
                    vec![
                        "menu_play",
                        "menu_freeplay",
                        "menu_library",
                        "menu_settings",
                        "menu_exit",
                    ]
                } else {
                    vec![
                        "menu_freeplay",
                        "menu_library",
                        "menu_settings",
                        "menu_exit",
                    ]
                };

                for (i, option_id) in option_ids.iter().enumerate() {
                    let option_y = start_y + (i as f32 * option_height);
                    if mouse_y >= option_y && mouse_y <= option_y + option_height {
                        // Set focus to the hovered option using unified input manager
                        self.input_manager.focus().set_focus(option_id);
                        break;
                    }
                }
            }
        }

        // Handle mouse click
        if is_mouse_button_pressed(MouseButton::Left) {
            return self.activate_focused(ctx);
        }

        None
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        clear_background(BLACK);

        let screen_w = vw();
        let screen_h = vh();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;

        // Draw title with headline font
        crate::scene::ply_fonts::draw_headline(
            "NEOTHESIA",
            center_x - 100.0,
            center_y - 150.0,
            50.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Draw song info if available
        if let Some(song) = &self.song {
            crate::scene::ply_fonts::draw_body(
                &format!("Song: {}", song.file.name),
                center_x - 150.0,
                center_y - 80.0,
                20.0,
                Color::from_rgba(200, 200, 255, 255),
            );
        } else {
            crate::scene::ply_fonts::draw_body(
                "No song loaded",
                center_x - 80.0,
                center_y - 80.0,
                20.0,
                Color::from_rgba(255, 100, 100, 255),
            );
            crate::scene::ply_fonts::draw_body(
                "Use --midi-file <path> to load a song",
                center_x - 200.0,
                center_y - 50.0,
                14.0,
                Color::from_rgba(150, 150, 150, 255),
            );
        }

        // Build menu options dynamically based on whether song is loaded
        let has_song = self.song.is_some();
        let options: Vec<&str> = if has_song {
            vec!["Play Song", "Free Play", "Song Library", "Settings", "Exit"]
        } else {
            vec!["Free Play", "Song Library", "Settings", "Exit"]
        };

        // Get input priority for focus indicator color
        let has_keyboard_priority = self
            .input_manager
            .focus()
            .priority()
            .has_keyboard_priority();

        let start_y = center_y;
        for (i, option) in options.iter().enumerate() {
            let is_focused = Some(i) == self.focused_index();

            // Use unified focus indicator - single color based on input priority
            let color = if is_focused {
                if has_keyboard_priority {
                    Color::from_rgba(160, 81, 255, 255) // Purple for keyboard focus
                } else {
                    Color::from_rgba(100, 200, 100, 255) // Green for mouse focus
                }
            } else {
                Color::from_rgba(150, 150, 150, 255) // Gray for normal
            };

            let prefix = if is_focused { "> " } else { "  " };

            crate::scene::ply_fonts::draw_body(
                &format!("{}{}", prefix, option),
                center_x - 80.0,
                start_y + (i as f32 * 40.0),
                24.0,
                color,
            );
        }

        // Draw instructions with body font
        crate::scene::ply_fonts::draw_body(
            "Use UP/DOWN to select, ENTER to choose, or click with mouse",
            center_x - 230.0,
            screen_h - 50.0,
            14.0,
            Color::from_rgba(100, 100, 100, 255),
        );
    }
}

/// PLY Song Library Scene
pub struct PlySongLibraryScene {
    song: Option<Song>,
    songs: Vec<crate::song_library::SongEntry>,
    scroll_offset: f32,
    selected_song_index: Option<usize>,
    hovered_song_index: Option<usize>,
}

impl PlySongLibraryScene {
    pub fn new(song: Option<Song>) -> Self {
        Self {
            song,
            songs: Vec::new(),
            scroll_offset: 0.0,
            selected_song_index: None,
            hovered_song_index: None,
        }
    }

    pub fn load_songs(&mut self, ctx: &mut MacroquadContext) {
        use crate::song_library::{FilterState, SortPreference};
        if let Ok(entries) = ctx
            .song_library_db
            .list_songs(&SortPreference::default(), &FilterState::default())
        {
            self.songs = entries;
            log::info!("🎯 PLY SONG LIBRARY: Loaded {} songs", self.songs.len());
        }
    }
}

impl PlyScene for PlySongLibraryScene {
    fn update(&mut self, ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        // Load songs on first update if empty
        if self.songs.is_empty() {
            self.load_songs(ctx);
        }

        // Handle keyboard input
        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::MainMenu(self.song.take()));
        }

        // Handle scroll
        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 20.0).max(0.0);
        }

        // Handle mouse clicks
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = vmouse();
            let screen_w = vw();
            let screen_h = vh();

            // Check back button
            let back_btn_x = 10.0;
            let back_btn_y = screen_h - 50.0;
            let back_btn_w = 80.0;
            let back_btn_h = 40.0;

            if mouse_x >= back_btn_x
                && mouse_x <= back_btn_x + back_btn_w
                && mouse_y >= back_btn_y
                && mouse_y <= back_btn_y + back_btn_h
            {
                return Some(NeothesiaEvent::MainMenu(self.song.take()));
            }

            // Check song cards
            if let Some(idx) = self.hovered_song_index {
                if let Some(entry) = self.songs.get(idx) {
                    log::info!(
                        "🎯 PLY SONG LIBRARY: Loading song '{}' (id={})",
                        entry.name,
                        entry.id
                    );
                    log::info!("🎯 PLY SONG LIBRARY: File path: {:?}", entry.file_path);

                    // FIX: Load the selected song from the file path
                    match midi_file::MidiFile::new(&entry.file_path) {
                        Ok(midi_file) => {
                            let mut song = Song::new(midi_file);
                            song.song_id = Some(entry.id);
                            log::info!(
                                "✅ PLY SONG LIBRARY: Successfully loaded song '{}' from file",
                                song.file.name
                            );
                            return Some(NeothesiaEvent::MainMenu(Some(song)));
                        }
                        Err(e) => {
                            log::error!(
                                "❌ PLY SONG LIBRARY: Failed to load song from {:?}: {}",
                                entry.file_path,
                                e
                            );
                            // Return to main menu with the old song if loading fails
                            return Some(NeothesiaEvent::MainMenu(self.song.take()));
                        }
                    }
                }
            }
        }

        // Update hover state
        let (mouse_x, mouse_y) = vmouse();
        let screen_w = vw();
        let screen_h = vh();
        let margin_top = 60.0;
        let card_w = 280.0;
        let card_h = 160.0;
        let gap = 12.0;
        let columns = ((screen_w - 40.0) / (card_w + gap)).floor().max(1.0) as usize;

        self.hovered_song_index = None;
        for (idx, _entry) in self.songs.iter().enumerate() {
            let row = idx / columns;
            let col = idx % columns;
            let card_x = 20.0 + col as f32 * (card_w + gap);
            let card_y = margin_top + row as f32 * (card_h + gap) - self.scroll_offset;

            if mouse_x >= card_x
                && mouse_x <= card_x + card_w
                && mouse_y >= card_y
                && mouse_y <= card_y + card_h
            {
                self.hovered_song_index = Some(idx);
                break;
            }
        }

        None
    }

    fn render(&mut self, ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        clear_background(Color::from_rgba(25, 25, 30, 255));

        let screen_w = vw();
        let screen_h = vh();

        // Draw title with headline font
        let title = format!("Song Library - {} songs", self.songs.len());
        crate::scene::ply_fonts::draw_headline(
            &title,
            screen_w / 2.0 - 100.0,
            20.0,
            24.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Draw song cards
        let margin_top = 60.0;
        let card_w = 280.0;
        let card_h = 160.0;
        let gap = 12.0;
        let columns = ((screen_w - 40.0) / (card_w + gap)).floor().max(1.0) as usize;

        for (idx, entry) in self.songs.iter().enumerate() {
            let row = idx / columns;
            let col = idx % columns;
            let card_x = 20.0 + col as f32 * (card_w + gap);
            let card_y = margin_top + row as f32 * (card_h + gap) - self.scroll_offset;

            // Skip if outside visible area
            if card_y + card_h < 0.0 || card_y > screen_h - 60.0 {
                continue;
            }

            let is_hovered = self.hovered_song_index == Some(idx);

            // Draw card background
            let bg_color = if is_hovered {
                Color::from_rgba(60, 55, 70, 255)
            } else {
                Color::from_rgba(37, 35, 42, 255)
            };
            draw_rectangle(card_x, card_y, card_w, card_h, bg_color);

            // Draw card border
            let border_color = if is_hovered {
                Color::from_rgba(160, 81, 255, 255)
            } else {
                Color::from_rgba(80, 80, 80, 255)
            };
            draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, border_color);

            // Draw song name with headline font
            crate::scene::ply_fonts::draw_headline(
                &entry.name,
                card_x + 12.0,
                card_y + 20.0,
                16.0,
                Color::from_rgba(255, 255, 255, 255),
            );

            // Draw difficulty with body font
            let difficulty = crate::song_library::difficulty_label(entry.difficulty);
            let diff_color = match entry.difficulty {
                1..=3 => Color::from_rgba(80, 180, 112, 255),
                4..=7 => Color::from_rgba(180, 168, 80, 255),
                8..=10 => Color::from_rgba(180, 80, 80, 255),
                _ => Color::from_rgba(150, 150, 150, 255),
            };
            crate::scene::ply_fonts::draw_body(
                &format!("Difficulty: {}", difficulty),
                card_x + 12.0,
                card_y + 45.0,
                14.0,
                diff_color,
            );

            // Draw play count with body font
            let play_text = if entry.play_count == 0 {
                "Never played".to_string()
            } else {
                format!("Played {} times", entry.play_count)
            };
            crate::scene::ply_fonts::draw_body(
                &play_text,
                card_x + 12.0,
                card_y + 65.0,
                12.0,
                Color::from_rgba(150, 150, 150, 255),
            );

            // Draw scores with body font
            let mut y_offset = 85.0;
            if let Some(score) = entry.last_score {
                crate::scene::ply_fonts::draw_body(
                    &format!("Last Score: {:.0}%", score),
                    card_x + 12.0,
                    card_y + y_offset,
                    12.0,
                    Color::from_rgba(150, 150, 150, 255),
                );
                y_offset += 18.0;
            }

            if let Some(best) = entry.best_score {
                crate::scene::ply_fonts::draw_body(
                    &format!("Best Score: {:.0}%", best),
                    card_x + 12.0,
                    card_y + y_offset,
                    12.0,
                    Color::from_rgba(150, 200, 150, 255),
                );
            }

            // Draw click instruction with body font
            if is_hovered {
                crate::scene::ply_fonts::draw_body(
                    "Click to load",
                    card_x + card_w - 90.0,
                    card_y + card_h - 15.0,
                    12.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            }
        }

        // Draw bottom bar
        let bar_y = screen_h - 60.0;
        draw_rectangle(
            0.0,
            bar_y,
            screen_w,
            60.0,
            Color::from_rgba(37, 35, 42, 255),
        );

        // Draw back button
        let back_btn_x = 10.0;
        let back_btn_y = bar_y + 10.0;
        let back_btn_w = 80.0;
        let back_btn_h = 40.0;

        let (mouse_x, mouse_y) = vmouse();
        let back_hovered = mouse_x >= back_btn_x
            && mouse_x <= back_btn_x + back_btn_w
            && mouse_y >= back_btn_y
            && mouse_y <= back_btn_y + back_btn_h;

        let back_bg_color = if back_hovered {
            Color::from_rgba(160, 81, 255, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        draw_rectangle(
            back_btn_x,
            back_btn_y,
            back_btn_w,
            back_btn_h,
            back_bg_color,
        );
        draw_rectangle_lines(
            back_btn_x,
            back_btn_y,
            back_btn_w,
            back_btn_h,
            1.0,
            Color::from_rgba(100, 100, 100, 255),
        );

        // Draw back button with headline font
        crate::scene::ply_fonts::draw_headline(
            "← Back",
            back_btn_x + 20.0,
            back_btn_y + 25.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Draw instructions with body font
        crate::scene::ply_fonts::draw_body(
            "Click a song to load it • ESC: Back to menu",
            screen_w / 2.0 - 180.0,
            bar_y + 25.0,
            14.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}

/// Top bar button for Macroquad rendering
struct TopBarButton {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl TopBarButton {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn is_hovered(&self, mx: f32, my: f32) -> bool {
        mx >= self.x && mx <= self.x + self.w && my >= self.y && my <= self.y + self.h
    }

    fn render(&self, mx: f32, my: f32, mouse_down: bool) {
        let hovered = self.is_hovered(mx, my);
        let color = if hovered && mouse_down {
            Color::from_rgba(97, 97, 97, 255)
        } else if hovered {
            Color::from_rgba(87, 87, 87, 255)
        } else {
            Color::from_rgba(67, 67, 67, 255)
        };
        draw_rectangle(self.x, self.y, self.w, self.h, color);
    }
}

/// PLY Playing Scene — full Macroquad top bar with playback controls
pub struct PlyPlayingScene {
    song: Song,
    paused: bool,
    piano_keyboard: PianoKeyboardRenderer,
    mouse_was_pressed: bool,
    waterfall: Option<crate::render::ply::waterfall::PlyWaterfallRenderer>,
    playback_time: f32,

    // Top bar state
    speed_multiplier: f32,
    runtime_gain: f32,
    looper_active: bool,
    looper_start: f32,
    looper_end: f32,
    is_seeking: bool,
    is_dragging_looper_start: bool,
    is_dragging_looper_end: bool,
    wait_mode: bool,
    waiting_for_notes: std::collections::HashSet<u8>,

    // Cached song duration
    song_length: f32,
    lead_in: f32,

    // Waterfall audio tracking
    active_waterfall_notes: std::collections::HashSet<u8>,

    live_score: LiveScoreTracker,
    effects: EffectsManager,
    last_timing_quality: Option<TimingQuality>,
}

impl PlyPlayingScene {
    pub fn new(song: Song) -> Self {
        let config = Config::new();
        let range = config.piano_range();
        let keyboard_range = piano_layout::KeyboardRange::new(range.clone());
        let sizing = piano_layout::Sizing::new(40.0, 120.0);
        let layout = KeyboardLayout::from_range(sizing, keyboard_range);

        let piano_keyboard = PianoKeyboardRenderer::new(layout, &config);

        // Compute song duration from tracks and count total notes
        let lead_in = 3.0f32;
        let mut last_note_end = 0.0f32;
        let mut total_notes_in_song = 0u32;
        for track in song.file.tracks.iter() {
            total_notes_in_song += track.notes.len() as u32;
            if let Some(note) = track.notes.last() {
                let end = note.start.as_secs_f32() + note.duration.as_secs_f32();
                last_note_end = last_note_end.max(end);
            }
        }
        let song_length = last_note_end + lead_in;

        // Initialize gain from config
        let runtime_gain = config.playback_gain();

        Self {
            song,
            paused: false,
            piano_keyboard,
            mouse_was_pressed: false,
            waterfall: None,
            playback_time: -lead_in,
            speed_multiplier: config.speed_multiplier(),
            runtime_gain,
            looper_active: false,
            looper_start: 0.0,
            looper_end: 0.0,
            is_seeking: false,
            is_dragging_looper_start: false,
            is_dragging_looper_end: false,
            wait_mode: false,
            waiting_for_notes: std::collections::HashSet::new(),
            song_length,
            lead_in,
            active_waterfall_notes: std::collections::HashSet::new(),
            live_score: LiveScoreTracker::new().with_total_notes(total_notes_in_song),
            effects: EffectsManager::new(),
            last_timing_quality: None,
        }
    }

    pub fn new_resumed(song: Song, resume_time: f32) -> Self {
        let mut scene = Self::new(song);
        scene.playback_time = resume_time;
        scene.paused = true; // Start paused, user can unpause when ready
        scene
    }

    fn initialize_waterfall(&mut self, ctx: &mut MacroquadContext) {
        use crate::render::ply::waterfall::PlyWaterfallRenderer;
        use neothesia_core::waterfall::TrackChannelConfig;

        let mut waterfall = PlyWaterfallRenderer::new();
        let tracks: &[midi_file::MidiTrack] = &self.song.file.tracks;
        let hidden_tracks: Vec<usize> = Vec::new();
        let track_channel_configs: Vec<TrackChannelConfig> = Vec::new();

        let range = ctx.config.piano_range();
        let keyboard_range = piano_layout::KeyboardRange::new(range);
        let sizing = piano_layout::Sizing::new(40.0, 120.0);
        let layout = KeyboardLayout::from_range(sizing, keyboard_range);

        waterfall.initialize(
            tracks,
            &hidden_tracks,
            &track_channel_configs,
            &ctx.config,
            &layout,
        );
        self.waterfall = Some(waterfall);
    }

    fn percentage(&self) -> f32 {
        if self.song_length <= 0.0 {
            return 0.0;
        }
        (self.playback_time / self.song_length).clamp(0.0, 1.0)
    }

    fn set_percentage(&mut self, p: f32) {
        self.playback_time = p.clamp(0.0, 1.0) * self.song_length;
    }

    fn format_time(secs: f32) -> String {
        let total_secs = secs.max(0.0) as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }

    // ─── Top Bar Layout Constants ───
    const TOP_BAR_H: f32 = 30.0;
    const PROGRESS_BAR_H: f32 = 45.0;

    // ─── Design System Colors (Sonic Obsidian) ───
    const COLOR_BACKGROUND: Color = Color::new(0.055, 0.055, 0.075, 1.0); // #0e0e13
    const COLOR_SURFACE_CONTAINER: Color = Color::new(0.098, 0.098, 0.122, 1.0); // #19191f
    const COLOR_SURFACE_CONTAINER_HIGHEST: Color = Color::new(0.145, 0.145, 0.173, 1.0); // #25252c
    const COLOR_PRIMARY: Color = Color::new(0.859, 0.565, 1.0, 1.0); // #db90ff
    const COLOR_PRIMARY_CONTAINER: Color = Color::new(0.827, 0.482, 1.0, 1.0); // #d37bff
    const COLOR_SECONDARY: Color = Color::new(0.373, 0.620, 1.0, 1.0); // #5f9eff
    const COLOR_TERTIARY: Color = Color::new(1.0, 0.431, 0.502, 1.0); // #ff6e80
    const COLOR_ON_SURFACE: Color = Color::new(0.973, 0.961, 0.992, 1.0); // #f8f5fd
    const COLOR_ON_SURFACE_VARIANT: Color = Color::new(0.667, 0.655, 0.694, 1.0); // #acaab1
    const COLOR_OUTLINE_VARIANT: Color = Color::new(0.282, 0.278, 0.302, 1.0); // #48474d
    const COLOR_SURFACE_VARIANT: Color = Color::new(0.145, 0.145, 0.173, 1.0); // #25252c

    fn render_top_bar(&self, mx: f32, my: f32, mouse_down: bool) {
        let sw = vw();
        let dark_gray = Color::from_rgba(37, 35, 42, 255);

        // Panel background
        draw_rectangle(0.0, 0.0, sw, Self::TOP_BAR_H, dark_gray);

        // ── Left: Back button ──
        let back_btn = TopBarButton::new(0.0, 0.0, Self::TOP_BAR_H, Self::TOP_BAR_H);
        back_btn.render(mx, my, mouse_down);
        crate::scene::ply_fonts::draw_body("<", 10.0, 20.0, 18.0, WHITE);

        // ── Center: Speed + Gain ──
        let group_w = 170.0;
        let gap = 20.0;
        let total_w = group_w * 2.0 + gap;
        let start_x = (sw - total_w) / 2.0;

        // Speed group
        let speed_x = start_x;
        crate::scene::ply_fonts::draw_body(
            "Speed",
            speed_x,
            20.0,
            12.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        let speed_minus = TopBarButton::new(speed_x + 50.0, 3.0, 35.0, 24.0);
        speed_minus.render(mx, my, mouse_down);
        crate::scene::ply_fonts::draw_body("-", speed_x + 63.0, 20.0, 16.0, WHITE);

        let speed_pct = format!("{}%", (self.speed_multiplier() * 100.0).round());
        crate::scene::ply_fonts::draw_body(&speed_pct, speed_x + 88.0, 20.0, 14.0, WHITE);

        let speed_plus = TopBarButton::new(speed_x + 135.0, 3.0, 35.0, 24.0);
        speed_plus.render(mx, my, mouse_down);
        crate::scene::ply_fonts::draw_body("+", speed_x + 148.0, 20.0, 16.0, WHITE);

        // Gain group
        let gain_x = start_x + group_w + gap;
        crate::scene::ply_fonts::draw_body(
            "Gain",
            gain_x,
            20.0,
            12.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        let gain_minus = TopBarButton::new(gain_x + 50.0, 3.0, 35.0, 24.0);
        gain_minus.render(mx, my, mouse_down);
        crate::scene::ply_fonts::draw_body("-", gain_x + 63.0, 20.0, 16.0, WHITE);

        let gain_pct = format!("{}%", (self.runtime_gain * 100.0).round());
        crate::scene::ply_fonts::draw_body(&gain_pct, gain_x + 88.0, 20.0, 14.0, WHITE);

        let gain_plus = TopBarButton::new(gain_x + 135.0, 3.0, 35.0, 24.0);
        gain_plus.render(mx, my, mouse_down);
        crate::scene::ply_fonts::draw_body("+", gain_x + 148.0, 20.0, 16.0, WHITE);

        // ── Right: Playback controls ──
        let btn_size = Self::TOP_BAR_H;
        let mut rx = sw;

        // Play / Pause button
        rx -= btn_size;
        let play_btn = TopBarButton::new(rx, 0.0, btn_size, btn_size);
        play_btn.render(mx, my, mouse_down);
        if self.paused {
            crate::scene::ply_fonts::draw_body(">", rx + 10.0, 20.0, 18.0, WHITE);
        } else {
            crate::scene::ply_fonts::draw_body("||", rx + 8.0, 20.0, 16.0, WHITE);
        }

        // Looper button
        rx -= btn_size;
        let looper_color = if self.looper_active {
            Color::from_rgba(56, 145, 255, 255)
        } else {
            Color::from_rgba(67, 67, 67, 255)
        };
        let looper_hover = mx >= rx && mx <= rx + btn_size && my >= 0.0 && my <= btn_size;
        let looper_bg = if looper_hover && mouse_down {
            Color::from_rgba(97, 97, 97, 255)
        } else if looper_hover {
            Color::from_rgba(87, 87, 87, 255)
        } else {
            looper_color
        };
        draw_rectangle(rx, 0.0, btn_size, btn_size, looper_bg);
        crate::scene::ply_fonts::draw_body("L", rx + 10.0, 20.0, 16.0, WHITE);

        // Wait mode button
        rx -= btn_size;
        let wait_color = if self.wait_mode {
            Color::from_rgba(56, 145, 255, 255)
        } else {
            Color::from_rgba(67, 67, 67, 255)
        };
        let wait_hover = mx >= rx && mx <= rx + btn_size && my >= 0.0 && my <= btn_size;
        let wait_bg = if wait_hover && mouse_down {
            Color::from_rgba(97, 97, 97, 255)
        } else if wait_hover {
            Color::from_rgba(87, 87, 87, 255)
        } else {
            wait_color
        };
        draw_rectangle(rx, 0.0, btn_size, btn_size, wait_bg);
        crate::scene::ply_fonts::draw_body("W", rx + 10.0, 20.0, 16.0, WHITE);
    }

    fn render_progress_bar(&self, mx: f32, my: f32, mouse_down: bool) {
        let sw = vw();
        let bar_y = Self::TOP_BAR_H;
        let bar_h = Self::PROGRESS_BAR_H;
        let progress_w = sw * self.percentage();

        // Background
        draw_rectangle(0.0, bar_y, sw, bar_h, Color::from_rgba(30, 30, 35, 255));

        // Played portion
        draw_rectangle(
            0.0,
            bar_y,
            progress_w,
            bar_h,
            Color::from_rgba(56, 145, 255, 255),
        );

        // Hover highlight
        if my >= bar_y && my <= bar_y + bar_h && mx >= 0.0 && mx <= sw {
            draw_rectangle(
                mx - 1.0,
                bar_y,
                2.0,
                bar_h,
                Color::from_rgba(255, 255, 255, 100),
            );
        }

        // Measure markers
        let light = Color::from_rgba(255, 255, 255, 128);
        let dark = Color::from_rgba(102, 102, 102, 255);
        for m in self.song.file.measures.iter() {
            let measure_pct = m.as_secs_f32() / self.song_length;
            let mx_pos = measure_pct * sw;
            let color = if mx_pos < progress_w { light } else { dark };
            draw_rectangle(mx_pos, bar_y, 1.0, bar_h, color);
        }

        // Looper region
        if self.looper_active && self.song_length > 0.0 {
            let loop_start_x = (self.looper_start / self.song_length) * sw;
            let loop_end_x = (self.looper_end / self.song_length) * sw;

            // Looper region overlay
            draw_rectangle(
                loop_start_x,
                bar_y - 5.0,
                loop_end_x - loop_start_x,
                bar_h + 10.0,
                Color::from_rgba(255, 56, 187, 60),
            );

            // Looper handles
            let handle_w = 5.0;
            let handle_h = bar_h + 10.0;

            let ls_hover = mx >= loop_start_x - 3.0
                && mx <= loop_start_x + handle_w + 3.0
                && my >= bar_y - 5.0
                && my <= bar_y + bar_h + 5.0;
            let ls_color = if ls_hover || self.is_dragging_looper_start {
                WHITE
            } else {
                Color::from_rgba(255, 56, 187, 255)
            };
            draw_rectangle(loop_start_x, bar_y - 5.0, handle_w, handle_h, ls_color);

            let le_hover = mx >= loop_end_x - 3.0
                && mx <= loop_end_x + handle_w + 3.0
                && my >= bar_y - 5.0
                && my <= bar_y + bar_h + 5.0;
            let le_color = if le_hover || self.is_dragging_looper_end {
                WHITE
            } else {
                Color::from_rgba(255, 56, 187, 255)
            };
            draw_rectangle(loop_end_x, bar_y - 5.0, handle_w, handle_h, le_color);
        }

        // Playback cursor — thin white line
        draw_rectangle(progress_w - 1.0, bar_y, 2.0, bar_h, WHITE);

        // Time labels
        let current = Self::format_time(self.playback_time.max(0.0));
        let total = Self::format_time(self.song_length);
        let time_text = format!("{} / {}", current, total);
        draw_text(
            &time_text,
            10.0,
            bar_y + bar_h - 8.0,
            12.0,
            Color::from_rgba(200, 200, 200, 255),
        );
    }

    fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }

    fn find_matching_note(&self, pressed_note: u8) -> Option<std::time::Duration> {
        use std::time::Duration;

        let Some(waterfall) = &self.waterfall else {
            return None;
        };

        let current_time_secs = self.playback_time;
        let timing_window = Duration::from_millis(200);

        for midi_note in waterfall.notes().inner().iter() {
            if midi_note.note != pressed_note {
                continue;
            }

            let note_start = midi_note.start;
            let note_start_secs = note_start.as_secs_f32();

            let delta = if current_time_secs >= note_start_secs {
                Duration::from_secs_f32(current_time_secs - note_start_secs)
            } else {
                Duration::from_secs_f32(note_start_secs - current_time_secs)
            };

            if delta <= timing_window {
                return Some(delta);
            }
        }

        None
    }

    fn process_note_hit(&mut self, pressed_note: u8) {
        use std::time::Duration;

        self.waiting_for_notes.remove(&pressed_note);

        if let Some(delta) = self.find_matching_note(pressed_note) {
            let quality = TimingQuality::from_delta(delta);
            let (_, milestone) = self.live_score.on_note_hit(quality);
            self.last_timing_quality = Some(quality);

            match quality {
                TimingQuality::Miss => {
                    self.effects.trigger_shake(ScreenShake::small());
                }
                _ => {}
            }

            if let Some(m) = milestone {
                match m {
                    StreakMilestone::Multiplier8x => {
                        self.effects.trigger_flash(ScreenFlash::gold(0.3));
                    }
                    StreakMilestone::OnFire => {
                        self.effects.trigger_shake(ScreenShake::medium());
                        self.effects.trigger_flash(ScreenFlash::gold(0.5));
                    }
                    StreakMilestone::Legendary => {
                        self.effects.trigger_shake(ScreenShake::large());
                        self.effects.trigger_flash(ScreenFlash::gold(0.8));
                    }
                    _ => {}
                }
            }
        }
    }

    fn create_score_data(&self) -> crate::scoring_data::ScoreData {
        let result = self.live_score.to_score_data();
        crate::scoring_data::ScoreData {
            total_notes: result.total_notes as usize,
            correct_notes: (result.total_notes - result.miss_count) as usize,
            missed_notes: result.miss_count as usize,
            too_early: 0,
            too_late: 0,
            on_time: (result.perfect_count + result.good_count + result.okay_count) as usize,
            accuracy: result.accuracy,
            grade: result.grade().to_string(),
            stars: result.stars.count(),
            max_streak: result.max_streak,
            score: result.score,
            perfect_count: result.perfect_count,
            good_count: result.good_count,
            okay_count: result.okay_count,
        }
    }

    /// Render the score panel in top-left corner with glassmorphism background
    fn render_score_panel(&self) {
        let score = self.live_score.score();
        let streak = self.live_score.streak().current();
        let accuracy = self.live_score.accuracy();

        // Panel dimensions and position (top-left)
        let panel_x = 32.0;
        let panel_y = 32.0;
        let panel_w = 220.0;
        let panel_h = 140.0;

        // Glassmorphism background (surface_container with transparency)
        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::new(0.098, 0.098, 0.122, 0.85), // surface_container at 85% opacity
        );

        // Left accent border (2px primary color)
        draw_rectangle(panel_x, panel_y, 3.0, panel_h, Self::COLOR_PRIMARY);

        // "CURRENT SCORE" label (small, uppercase, on_surface_variant)
        crate::scene::ply_fonts::draw_label(
            "CURRENT SCORE",
            panel_x + 16.0,
            panel_y + 20.0,
            10.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        // Score number (large, primary color)
        let score_text = format!("{}", score);
        crate::scene::ply_fonts::draw_headline(
            &score_text,
            panel_x + 16.0,
            panel_y + 55.0,
            32.0,
            Self::COLOR_PRIMARY,
        );

        // Streak and Accuracy row
        let row_y = panel_y + 90.0;

        // Streak label and value
        crate::scene::ply_fonts::draw_label(
            "STREAK",
            panel_x + 16.0,
            row_y,
            8.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );
        let streak_text = format!("x{}", streak);
        crate::scene::ply_fonts::draw_body(
            &streak_text,
            panel_x + 16.0,
            row_y + 18.0,
            16.0,
            Self::COLOR_SECONDARY,
        );

        // Separator line
        draw_rectangle(
            panel_x + 85.0,
            row_y - 5.0,
            1.0,
            35.0,
            Self::COLOR_OUTLINE_VARIANT,
        );

        // Accuracy label and value
        crate::scene::ply_fonts::draw_label(
            "ACCURACY",
            panel_x + 95.0,
            row_y,
            8.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );
        let accuracy_text = format!("{:.1}%", accuracy);
        crate::scene::ply_fonts::draw_body(
            &accuracy_text,
            panel_x + 95.0,
            row_y + 18.0,
            16.0,
            Self::COLOR_TERTIARY,
        );
    }

    /// Render song title and session config bar in top-center
    fn render_song_info(&self, mx: f32, my: f32, mouse_down: bool) {
        let sw = vw();
        let center_x = sw / 2.0;

        let title_y = 16.0;
        crate::scene::ply_fonts::draw_headline(
            &self.song.file.name,
            center_x - 150.0,
            title_y,
            18.0,
            Self::COLOR_ON_SURFACE,
        );

        crate::scene::ply_fonts::draw_body(
            "MIDI Performance",
            center_x - 100.0,
            title_y + 22.0,
            10.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        let bar_y = title_y + 40.0;
        let bar_w = 420.0;
        let bar_h = 44.0;
        let bar_x = center_x - bar_w / 2.0;

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Self::COLOR_SURFACE_CONTAINER);
        draw_rectangle(
            bar_x,
            bar_y,
            bar_w,
            1.0,
            Color::new(0.282, 0.278, 0.302, 0.2),
        );
        draw_rectangle(
            bar_x,
            bar_y + bar_h - 1.0,
            bar_w,
            1.0,
            Color::new(0.282, 0.278, 0.302, 0.2),
        );

        let btn_h = 20.0;
        let btn_y = bar_y + (bar_h - btn_h) / 2.0;
        let mut cx = bar_x + 12.0;

        crate::scene::ply_fonts::draw_label(
            "SPEED",
            cx,
            bar_y + 8.0,
            7.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        draw_rectangle(cx, btn_y, btn_h, btn_h, Self::COLOR_SURFACE_VARIANT);
        crate::scene::ply_fonts::draw_body(
            "-",
            cx + 7.0,
            btn_y + 14.0,
            14.0,
            Self::COLOR_ON_SURFACE,
        );

        let speed = self.speed_multiplier();
        let speed_pct = (speed * 100.0).round() as i32;
        crate::scene::ply_fonts::draw_body(
            &format!("{}%", speed_pct),
            cx + 24.0,
            btn_y + 14.0,
            12.0,
            Self::COLOR_PRIMARY,
        );

        draw_rectangle(cx + 62.0, btn_y, btn_h, btn_h, Self::COLOR_SURFACE_VARIANT);
        crate::scene::ply_fonts::draw_body(
            "+",
            cx + 67.0,
            btn_y + 14.0,
            14.0,
            Self::COLOR_ON_SURFACE,
        );

        cx += 92.0;

        draw_rectangle(
            cx,
            bar_y + 6.0,
            1.0,
            bar_h - 12.0,
            Self::COLOR_OUTLINE_VARIANT,
        );
        cx += 8.0;

        let pause_r = 14.0;
        let pause_cx = cx + pause_r;
        let pause_cy = bar_y + bar_h / 2.0;
        draw_circle(pause_cx, pause_cy, pause_r, Self::COLOR_PRIMARY);
        if self.paused {
            let ts = 7.0;
            draw_line(
                pause_cx - ts / 2.0,
                pause_cy - ts / 2.0,
                pause_cx + ts / 2.0,
                pause_cy,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_line(
                pause_cx + ts / 2.0,
                pause_cy,
                pause_cx - ts / 2.0,
                pause_cy + ts / 2.0,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_line(
                pause_cx - ts / 2.0,
                pause_cy + ts / 2.0,
                pause_cx - ts / 2.0,
                pause_cy - ts / 2.0,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
        } else {
            let bw = 2.5;
            let bh = 12.0;
            let bg = 4.0;
            draw_rectangle(
                pause_cx - bg - bw,
                pause_cy - bh / 2.0,
                bw,
                bh,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_rectangle(
                pause_cx + bg,
                pause_cy - bh / 2.0,
                bw,
                bh,
                Self::COLOR_SURFACE_CONTAINER,
            );
        }

        cx += pause_r * 2.0 + 8.0;

        draw_rectangle(
            cx,
            bar_y + 6.0,
            1.0,
            bar_h - 12.0,
            Self::COLOR_OUTLINE_VARIANT,
        );
        cx += 8.0;

        let wait_w = 50.0;
        let wait_color = if self.wait_mode {
            Self::COLOR_PRIMARY
        } else {
            Self::COLOR_SURFACE_VARIANT
        };
        let wait_hover = mx >= cx && mx <= cx + wait_w && my >= bar_y && my <= bar_y + bar_h;
        let wait_bg = if wait_hover && mouse_down {
            Color::new(0.2, 0.2, 0.25, 1.0)
        } else if self.wait_mode {
            Color::new(0.145, 0.098, 0.2, 1.0)
        } else {
            wait_color
        };
        draw_rectangle(cx, bar_y + 4.0, wait_w, bar_h - 8.0, wait_bg);
        draw_rectangle(
            cx,
            bar_y + 4.0,
            wait_w,
            bar_h - 8.0,
            Color::new(wait_color.r, wait_color.g, wait_color.b, 0.5),
        );
        let wait_text_color = if self.wait_mode {
            Self::COLOR_ON_SURFACE
        } else {
            Self::COLOR_ON_SURFACE_VARIANT
        };
        crate::scene::ply_fonts::draw_label(
            "WAIT",
            cx + 13.0,
            bar_y + bar_h / 2.0 + 4.0,
            10.0,
            wait_text_color,
        );

        cx += wait_w + 6.0;

        let loop_w = 50.0;
        let loop_color = if self.looper_active {
            Self::COLOR_SECONDARY
        } else {
            Self::COLOR_SURFACE_VARIANT
        };
        let loop_hover = mx >= cx && mx <= cx + loop_w && my >= bar_y && my <= bar_y + bar_h;
        let loop_bg = if loop_hover && mouse_down {
            Color::new(0.2, 0.2, 0.25, 1.0)
        } else if self.looper_active {
            Color::new(0.098, 0.145, 0.2, 1.0)
        } else {
            loop_color
        };
        draw_rectangle(cx, bar_y + 4.0, loop_w, bar_h - 8.0, loop_bg);
        draw_rectangle(
            cx,
            bar_y + 4.0,
            loop_w,
            bar_h - 8.0,
            Color::new(loop_color.r, loop_color.g, loop_color.b, 0.5),
        );
        let loop_text_color = if self.looper_active {
            Self::COLOR_ON_SURFACE
        } else {
            Self::COLOR_ON_SURFACE_VARIANT
        };
        crate::scene::ply_fonts::draw_label(
            "LOOP",
            cx + 11.0,
            bar_y + bar_h / 2.0 + 4.0,
            10.0,
            loop_text_color,
        );

        cx += loop_w + 8.0;

        draw_rectangle(
            cx,
            bar_y + 6.0,
            1.0,
            bar_h - 12.0,
            Self::COLOR_OUTLINE_VARIANT,
        );
        cx += 8.0;

        let settings_r = 12.0;
        draw_circle(
            cx + settings_r,
            bar_y + bar_h / 2.0,
            settings_r,
            Self::COLOR_SURFACE_VARIANT,
        );
        crate::scene::ply_fonts::draw_body(
            "⚙",
            cx + settings_r - 6.0,
            bar_y + bar_h / 2.0 + 5.0,
            12.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );
    }

    /// Handle clicks on pause and settings buttons in the session config bar
    fn handle_song_info_click(
        &mut self,
        mx: f32,
        my: f32,
        mouse_just_pressed: bool,
        ctx: &mut MacroquadContext,
    ) -> Option<NeothesiaEvent> {
        if !mouse_just_pressed {
            return None;
        }

        let sw = vw();
        let center_x = sw / 2.0;
        let title_y = 16.0;
        let bar_y = title_y + 40.0;
        let bar_w = 420.0;
        let bar_h = 44.0;
        let bar_x = center_x - bar_w / 2.0;

        if my < bar_y || my > bar_y + bar_h || mx < bar_x || mx > bar_x + bar_w {
            return None;
        }

        let btn_h = 20.0;
        let btn_y = bar_y + (bar_h - btn_h) / 2.0;
        let mut cx = bar_x + 12.0;

        if mx >= cx && mx <= cx + btn_h && my >= btn_y && my <= btn_y + btn_h {
            let current_speed = ctx.config.speed_multiplier();
            ctx.config.set_speed_multiplier(current_speed - 0.1);
            return None;
        }

        if mx >= cx + 62.0 && mx <= cx + 62.0 + btn_h && my >= btn_y && my <= btn_y + btn_h {
            let current_speed = ctx.config.speed_multiplier();
            ctx.config.set_speed_multiplier(current_speed + 0.1);
            return None;
        }

        cx += 92.0;
        cx += 8.0;

        let pause_r = 14.0;
        let pause_cx = cx + pause_r;
        let pause_cy = bar_y + bar_h / 2.0;
        let pause_dist = ((mx - pause_cx).powi(2) + (my - pause_cy).powi(2)).sqrt();
        if pause_dist <= pause_r {
            self.paused = !self.paused;
            return None;
        }

        cx += pause_r * 2.0 + 8.0;
        cx += 8.0;

        let wait_w = 50.0;
        if mx >= cx && mx <= cx + wait_w {
            self.wait_mode = !self.wait_mode;
            self.song.config.wait_mode = self.wait_mode;
            if !self.wait_mode {
                self.waiting_for_notes.clear();
            }
            return None;
        }

        cx += wait_w + 6.0;

        let loop_w = 50.0;
        if mx >= cx && mx <= cx + loop_w {
            self.looper_active = !self.looper_active;
            if self.looper_active {
                let start = if self.looper_start > 0.0 {
                    self.looper_start
                } else {
                    self.playback_time.max(0.0)
                };
                let end = if self.looper_end > self.looper_start {
                    self.looper_end
                } else {
                    (self.playback_time + 5.0).min(self.song_length)
                };
                self.looper_start = start;
                self.looper_end = end;
            }
            return None;
        }

        cx += loop_w + 8.0;
        cx += 8.0;

        let settings_r = 12.0;
        let settings_cx = cx + settings_r;
        let settings_cy = bar_y + bar_h / 2.0;
        let settings_dist = ((mx - settings_cx).powi(2) + (my - settings_cy).powi(2)).sqrt();
        if settings_dist <= settings_r {
            self.paused = true;
            ctx.resume_playback_time = Some(self.playback_time);
            ctx.resume_song = Some(self.song.clone());
            return Some(NeothesiaEvent::ShowSettings);
        }

        None
    }

    /// Render vertical timeline progress bar on left side
    fn render_vertical_timeline(&self, mx: f32, my: f32, mouse_down: bool) {
        let screen_h = vh();
        let center_y = screen_h / 2.0;

        let timeline_x = 40.0;
        let timeline_top = center_y - 200.0;
        let timeline_bottom = center_y + 200.0;
        let timeline_h = timeline_bottom - timeline_top;

        let current_time = self.playback_time.max(0.0);
        let remaining_time = (self.song_length - self.playback_time).max(0.0);

        crate::scene::ply_fonts::draw_mono(
            &Self::format_time(remaining_time),
            timeline_x - 5.0,
            timeline_top - 14.0,
            10.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        crate::scene::ply_fonts::draw_mono(
            &Self::format_time(current_time),
            timeline_x - 5.0,
            timeline_bottom + 8.0,
            10.0,
            Self::COLOR_PRIMARY,
        );

        draw_rectangle(
            timeline_x - 2.0,
            timeline_top,
            4.0,
            timeline_h,
            Self::COLOR_SURFACE_CONTAINER_HIGHEST,
        );

        if self.looper_active {
            let loop_start_y =
                timeline_bottom - (self.looper_start / self.song_length) * timeline_h;
            let loop_end_y = timeline_bottom - (self.looper_end / self.song_length) * timeline_h;

            draw_rectangle(
                timeline_x - 4.0,
                loop_end_y,
                8.0,
                loop_start_y - loop_end_y,
                Color::new(0.373, 0.620, 1.0, 0.15),
            );

            let start_hover = (mx - timeline_x).abs() < 12.0 && (my - loop_start_y).abs() < 8.0;
            let start_r = if start_hover { 7.0 } else { 5.0 };
            draw_rectangle(
                timeline_x - 12.0,
                loop_start_y - 2.0,
                24.0,
                4.0,
                Self::COLOR_SECONDARY,
            );
            draw_circle(timeline_x, loop_start_y, start_r, Self::COLOR_SECONDARY);

            let end_hover = (mx - timeline_x).abs() < 12.0 && (my - loop_end_y).abs() < 8.0;
            let end_r = if end_hover { 7.0 } else { 5.0 };
            draw_rectangle(
                timeline_x - 12.0,
                loop_end_y - 2.0,
                24.0,
                4.0,
                Self::COLOR_SECONDARY,
            );
            draw_circle(timeline_x, loop_end_y, end_r, Self::COLOR_SECONDARY);
        }

        let progress = self.percentage();
        let fill_h = timeline_h * progress;
        draw_rectangle(
            timeline_x - 2.0,
            timeline_bottom - fill_h,
            4.0,
            fill_h,
            Self::COLOR_PRIMARY,
        );

        let thumb_y = timeline_bottom - fill_h;
        draw_circle(timeline_x, thumb_y, 8.0, Self::COLOR_PRIMARY);
        draw_circle(
            timeline_x,
            thumb_y,
            12.0,
            Color::new(0.859, 0.565, 1.0, 0.3),
        );
    }

    fn handle_vertical_timeline_click(
        &mut self,
        mx: f32,
        my: f32,
        mouse_down: bool,
        mouse_just_pressed: bool,
    ) {
        let screen_h = vh();
        let center_y = screen_h / 2.0;
        let timeline_x = 40.0;
        let timeline_top = center_y - 200.0;
        let timeline_bottom = center_y + 200.0;
        let timeline_h = timeline_bottom - timeline_top;

        if mx < timeline_x - 20.0 || mx > timeline_x + 20.0 {
            if !mouse_down {
                self.is_seeking = false;
                self.is_dragging_looper_start = false;
                self.is_dragging_looper_end = false;
            }
            return;
        }

        if my < timeline_top - 20.0 || my > timeline_bottom + 20.0 {
            if !mouse_down {
                self.is_seeking = false;
                self.is_dragging_looper_start = false;
                self.is_dragging_looper_end = false;
            }
            return;
        }

        let y_to_time = |y: f32| -> f32 {
            let p = ((timeline_bottom - y) / timeline_h).clamp(0.0, 1.0);
            p * self.song_length
        };

        if self.looper_active && mouse_just_pressed {
            let loop_start_y =
                timeline_bottom - (self.looper_start / self.song_length) * timeline_h;
            let loop_end_y = timeline_bottom - (self.looper_end / self.song_length) * timeline_h;

            if (my - loop_start_y).abs() < 10.0 {
                self.is_dragging_looper_start = true;
                self.is_seeking = false;
                return;
            }
            if (my - loop_end_y).abs() < 10.0 {
                self.is_dragging_looper_end = true;
                self.is_seeking = false;
                return;
            }
        }

        if self.is_dragging_looper_start && mouse_down {
            let new_start = y_to_time(my);
            if new_start > self.looper_end {
                self.looper_start = self.looper_end;
            } else {
                self.looper_start = new_start;
            }
            return;
        }

        if self.is_dragging_looper_end && mouse_down {
            let new_end = y_to_time(my);
            if new_end < self.looper_start {
                self.looper_end = self.looper_start;
            } else {
                self.looper_end = new_end;
            }
            return;
        }

        if mouse_just_pressed {
            self.is_seeking = true;
        }

        if self.is_seeking && mouse_down {
            let p = ((timeline_bottom - my) / timeline_h).clamp(0.0, 1.0);
            self.set_percentage(p);
            if let Some(waterfall) = &mut self.waterfall {
                waterfall.update(self.playback_time);
            }
        }

        if !mouse_down {
            self.is_seeking = false;
            self.is_dragging_looper_start = false;
            self.is_dragging_looper_end = false;
        }
    }

    /// Render latest timing quality feedback on top-right
    fn render_timing_feedback(&self) {
        if let Some(quality) = &self.last_timing_quality {
            let (text, color) = match quality {
                TimingQuality::Perfect => ("PERFECT", Color::new(1.0, 0.843, 0.0, 1.0)), // Gold
                TimingQuality::Good => ("GOOD", Color::new(0.0, 1.0, 0.0, 1.0)),         // Green
                TimingQuality::Okay => ("OKAY", Color::new(0.0, 0.533, 1.0, 1.0)),       // Blue
                TimingQuality::Miss => ("MISS", Color::new(1.0, 0.0, 0.0, 1.0)),         // Red
            };

            let screen_w = vw();
            let panel_y = 32.0;
            let panel_x = screen_w - 200.0;
            let panel_w = 160.0;
            let panel_h = 80.0;

            // Background panel
            draw_rectangle(
                panel_x,
                panel_y,
                panel_w,
                panel_h,
                Color::new(0.098, 0.098, 0.122, 0.85), // surface_container at 85% opacity
            );

            // Right accent border (colored by quality)
            draw_rectangle(panel_x + panel_w - 3.0, panel_y, 3.0, panel_h, color);

            // "LAST NOTE" label
            crate::scene::ply_fonts::draw_label(
                "LAST NOTE",
                panel_x + 12.0,
                panel_y + 20.0,
                10.0,
                Self::COLOR_ON_SURFACE_VARIANT,
            );

            // Quality text (large, colored)
            crate::scene::ply_fonts::draw_headline(
                text,
                panel_x + 12.0,
                panel_y + 55.0,
                28.0,
                color,
            );
        }
    }

    /// Render MIDI log overlay on right side
    fn render_midi_log(&self) {
        // Only show if there are recent MIDI events (placeholder for now)
        let screen_w = vw();
        let screen_h = vh();

        let log_x = screen_w - 180.0;
        let log_y = screen_h / 2.0 - 80.0;
        let log_w = 160.0;
        let log_h = 160.0;

        // Sample MIDI events (in real implementation, these would come from actual MIDI input)
        let events = [
            ("NOTE_ON 64 VEL:98", Self::COLOR_PRIMARY),
            ("NOTE_ON 67 VEL:102", Self::COLOR_SECONDARY),
            ("NOTE_OFF 64 VEL:0", Self::COLOR_PRIMARY),
            ("CC_SUSTAIN 127", Self::COLOR_TERTIARY),
        ];

        for (i, (text, color)) in events.iter().enumerate() {
            let event_y = log_y + (i as f32) * 35.0;
            let opacity = 0.9 - (i as f32) * 0.15;

            // Event background
            draw_rectangle(
                log_x,
                event_y,
                log_w,
                28.0,
                Color::new(0.0, 0.0, 0.0, 0.6 * opacity),
            );

            // Right accent border
            draw_rectangle(
                log_x + log_w - 2.0,
                event_y,
                2.0,
                28.0,
                Color::new(color.r, color.g, color.b, opacity),
            );

            // Event text
            crate::scene::ply_fonts::draw_mono(
                text,
                log_x + 8.0,
                event_y + 18.0,
                9.0,
                Color::new(color.r, color.g, color.b, opacity),
            );
        }
    }

    /// Render close button in top-right corner
    fn render_close_button(&self, mx: f32, my: f32, mouse_down: bool) {
        let screen_w = vw();
        let btn_x = screen_w - 60.0;
        let btn_y = 32.0;
        let btn_size = 40.0;

        // Button background (surface_container_highest)
        draw_rectangle(
            btn_x,
            btn_y,
            btn_size,
            btn_size,
            Self::COLOR_SURFACE_CONTAINER_HIGHEST,
        );

        // Hover effect
        if mx >= btn_x && mx <= btn_x + btn_size && my >= btn_y && my <= btn_y + btn_size {
            draw_rectangle(
                btn_x,
                btn_y,
                btn_size,
                btn_size,
                Color::new(1.0, 0.431, 0.502, 0.2),
            );
        }

        // X icon
        let icon_size = 14.0;
        let icon_x = btn_x + (btn_size - icon_size) / 2.0;
        let icon_y = btn_y + (btn_size - icon_size) / 2.0;

        draw_line(
            icon_x,
            icon_y,
            icon_x + icon_size,
            icon_y + icon_size,
            2.0,
            Self::COLOR_ON_SURFACE,
        );
        draw_line(
            icon_x + icon_size,
            icon_y,
            icon_x,
            icon_y + icon_size,
            2.0,
            Self::COLOR_ON_SURFACE,
        );
    }

    /// Handle close button click
    fn handle_close_button_click(&self, mx: f32, my: f32) -> Option<NeothesiaEvent> {
        let screen_w = vw();
        let btn_x = screen_w - 60.0;
        let btn_y = 32.0;
        let btn_size = 40.0;

        if mx >= btn_x && mx <= btn_x + btn_size && my >= btn_y && my <= btn_y + btn_size {
            let entry = song_entry_from_song(&self.song);
            Some(NeothesiaEvent::ShowSongSelected {
                song: self.song.clone(),
                entry,
            })
        } else {
            None
        }
    }

    fn handle_top_bar_click(
        &mut self,
        ctx: &mut MacroquadContext,
        mx: f32,
        my: f32,
    ) -> Option<NeothesiaEvent> {
        let sw = vw();
        let btn_size = Self::TOP_BAR_H;

        // ── Back button ──
        if mx >= 0.0 && mx <= btn_size && my >= 0.0 && my <= btn_size {
            let entry = song_entry_from_song(&self.song);
            return Some(NeothesiaEvent::ShowSongSelected {
                song: self.song.clone(),
                entry,
            });
        }

        // ── Speed controls ──
        let group_w = 170.0;
        let gap = 20.0;
        let total_w = group_w * 2.0 + gap;
        let start_x = (sw - total_w) / 2.0;
        let speed_x = start_x;

        if mx >= speed_x + 50.0 && mx <= speed_x + 85.0 && my >= 3.0 && my <= 27.0 {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
        }
        if mx >= speed_x + 135.0 && mx <= speed_x + 170.0 && my >= 3.0 && my <= 27.0 {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
        }

        // ── Gain controls ──
        let gain_x = start_x + group_w + gap;
        if mx >= gain_x + 50.0 && mx <= gain_x + 85.0 && my >= 3.0 && my <= 27.0 {
            self.runtime_gain = (self.runtime_gain - 0.1).max(0.0);
            ctx.output_manager.connection().set_gain(self.runtime_gain);
        }
        if mx >= gain_x + 135.0 && mx <= gain_x + 170.0 && my >= 3.0 && my <= 27.0 {
            self.runtime_gain += 0.1;
            ctx.output_manager.connection().set_gain(self.runtime_gain);
        }

        // ── Right buttons ──
        let mut rx = sw;

        // Play / Pause
        rx -= btn_size;
        if mx >= rx && mx <= rx + btn_size && my >= 0.0 && my <= btn_size {
            self.paused = !self.paused;
        }

        // Looper toggle
        rx -= btn_size;
        if mx >= rx && mx <= rx + btn_size && my >= 0.0 && my <= btn_size {
            self.looper_active = !self.looper_active;
            if self.looper_active && self.looper_start == 0.0 && self.looper_end == 0.0 {
                self.looper_start = self.playback_time.max(0.0);
                self.looper_end = (self.playback_time + 5.0).min(self.song_length);
            }
        }

        // Wait mode toggle
        rx -= btn_size;
        if mx >= rx && mx <= rx + btn_size && my >= 0.0 && my <= btn_size {
            self.wait_mode = !self.wait_mode;
            self.song.config.wait_mode = self.wait_mode;
        }

        None
    }

    fn handle_progress_bar_click(
        &mut self,
        mx: f32,
        my: f32,
        mouse_down: bool,
        mouse_just_pressed: bool,
    ) {
        let sw = vw();
        let bar_y = Self::TOP_BAR_H;
        let bar_h = Self::PROGRESS_BAR_H;

        if !mouse_down {
            self.is_seeking = false;
            self.is_dragging_looper_start = false;
            self.is_dragging_looper_end = false;
            return;
        }

        // Check looper handle dragging
        if self.looper_active && self.song_length > 0.0 {
            let loop_start_x = (self.looper_start / self.song_length) * sw;
            let loop_end_x = (self.looper_end / self.song_length) * sw;

            if self.is_dragging_looper_start {
                let p = (mx / sw).clamp(0.0, 1.0);
                let new_time = p * self.song_length;
                if new_time < self.looper_end - 0.5 {
                    self.looper_start = new_time;
                }
                return;
            }

            if self.is_dragging_looper_end {
                let p = (mx / sw).clamp(0.0, 1.0);
                let new_time = p * self.song_length;
                if new_time > self.looper_start + 0.5 {
                    self.looper_end = new_time;
                }
                return;
            }

            // Start dragging on mouse press near handles
            if mouse_just_pressed {
                if (loop_start_x - mx).abs() < 10.0 {
                    self.is_dragging_looper_start = true;
                    return;
                }
                if (loop_end_x - mx).abs() < 10.0 {
                    self.is_dragging_looper_end = true;
                    return;
                }
            }
        }

        // Regular seek
        if my >= bar_y && my <= bar_y + bar_h {
            if mouse_just_pressed {
                self.is_seeking = true;
            }
            if self.is_seeking {
                let p = (mx / sw).clamp(0.0, 1.0);
                self.set_percentage(p);
                if let Some(waterfall) = &mut self.waterfall {
                    waterfall.update(self.playback_time);
                }
            }
        }
    }
}

impl PlyScene for PlyPlayingScene {
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent> {
        use midi_file::midly::{num::u7, MidiMessage};

        let dt = delta.as_secs_f32();
        let (mouse_x, mouse_y) = vmouse();
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_just_pressed = is_mouse_button_pressed(MouseButton::Left);

        if self.waterfall.is_none() {
            self.initialize_waterfall(ctx);
        }

        // ── Keyboard shortcuts ──
        if is_key_pressed(KeyCode::Escape) {
            self.paused = true;
            ctx.resume_playback_time = Some(self.playback_time);
            let entry = song_entry_from_song(&self.song);
            return Some(NeothesiaEvent::ShowSongSelected {
                song: self.song.clone(),
                entry,
            });
        }
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }

        // Arrow keys: speed adjustment
        if is_key_pressed(KeyCode::Up) {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
        }
        if is_key_pressed(KeyCode::Down) {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
        }
        self.speed_multiplier = ctx.config.speed_multiplier();

        // ── Close button click handling ──
        if mouse_just_pressed {
            if let Some(event) = self.handle_close_button_click(mouse_x, mouse_y) {
                self.paused = true;
                ctx.resume_playback_time = Some(self.playback_time);
                return Some(event);
            }
        }

        // ── Song info button click handling (pause, settings) ──
        if mouse_just_pressed {
            if let Some(event) =
                self.handle_song_info_click(mouse_x, mouse_y, mouse_just_pressed, ctx)
            {
                return Some(event);
            }
        }

        // ── Vertical timeline seek interaction ──
        if mouse_just_pressed || mouse_down {
            self.handle_vertical_timeline_click(mouse_x, mouse_y, mouse_down, mouse_just_pressed);
        }

        // ── Playback advancement ──
        if !self.paused && !self.is_seeking {
            let speed = ctx.config.speed_multiplier();

            // Wait mode: don't advance if we're waiting for notes
            let blocked_by_wait = if self.wait_mode {
                if let Some(waterfall) = &self.waterfall {
                    let active = waterfall.get_active_notes();
                    for (note, _) in &active {
                        if !self.waiting_for_notes.contains(note) {
                            self.waiting_for_notes.insert(*note);
                        }
                    }
                    !self.waiting_for_notes.is_empty()
                } else {
                    false
                }
            } else {
                false
            };

            if !blocked_by_wait {
                let effective_dt = dt * speed;
                self.playback_time += effective_dt;
            }

            // Looper: wrap around
            if self.looper_active
                && self.playback_time > self.looper_end
                && self.looper_end > self.looper_start
            {
                self.playback_time = self.looper_start;
            }
        }

        if let Some(waterfall) = &mut self.waterfall {
            waterfall.update(self.playback_time);

            // Waterfall audio and learn mode
            if !self.paused && !self.is_seeking {
                let active_notes = waterfall.get_active_notes();
                let mut current_notes = std::collections::HashSet::new();

                // Trigger NoteOn for new active notes
                for (note, velocity) in &active_notes {
                    current_notes.insert(*note);
                    if !self.active_waterfall_notes.contains(note) {
                        self.piano_keyboard.handle_note_event(*note, *velocity);
                        let message = MidiMessage::NoteOn {
                            key: u7::new(*note),
                            vel: u7::new(*velocity),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }

                for note in self.active_waterfall_notes.drain() {
                    if !current_notes.contains(&note) {
                        self.piano_keyboard.handle_note_event(note, 0);
                        let message = MidiMessage::NoteOff {
                            key: u7::new(note),
                            vel: u7::new(0),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }

                self.active_waterfall_notes = current_notes;

                // Learn mode: highlight upcoming notes
                let upcoming_notes = waterfall.get_upcoming_notes(0.5);
                for note in upcoming_notes {
                    self.piano_keyboard.highlight_key(note, true);
                }
            }
        }

        self.piano_keyboard.update(dt);

        // ── Piano keyboard mouse input ──
        let keyboard_y_start = vh() - 150.0;
        let is_over_keyboard = mouse_y >= keyboard_y_start;

        if is_over_keyboard {
            if mouse_down && !self.mouse_was_pressed {
                if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                    Vec2::new(mouse_x, mouse_y),
                    MouseButton::Left,
                    true,
                ) {
                    for note in notes {
                        self.process_note_hit(note);
                        let message = MidiMessage::NoteOn {
                            key: u7::new(note),
                            vel: u7::new(100),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            } else if !mouse_down && self.mouse_was_pressed {
                if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                    Vec2::new(mouse_x, mouse_y),
                    MouseButton::Left,
                    false,
                ) {
                    for note in notes {
                        let message = MidiMessage::NoteOff {
                            key: u7::new(note),
                            vel: u7::new(0),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            } else if mouse_down && self.mouse_was_pressed {
                if let Some(notes) = self
                    .piano_keyboard
                    .handle_mouse_drag(Vec2::new(mouse_x, mouse_y))
                {
                    for note in notes {
                        let message = MidiMessage::NoteOn {
                            key: u7::new(note),
                            vel: u7::new(100),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            }
        }

        self.mouse_was_pressed = mouse_down;

        self.effects.update(delta);

        if !self.paused && !self.looper_active && self.playback_time >= self.song_length {
            return Some(NeothesiaEvent::ShowScore {
                song: self.song.clone(),
                score_data: self.create_score_data(),
            });
        }

        None
    }

    fn handle_midi_event(&mut self, _channel: u8, message: &midi_file::midly::MidiMessage) {
        use midi_file::midly::MidiMessage;

        match message {
            MidiMessage::NoteOn { key, vel } => {
                let note = key.as_int();
                self.process_note_hit(note);
                self.piano_keyboard.handle_note_event(note, vel.as_int());
            }
            MidiMessage::NoteOff { key, .. } => {
                self.piano_keyboard.handle_note_event(key.as_int(), 0);
            }
            _ => {}
        }
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        clear_background(Self::COLOR_BACKGROUND);

        let (mx, my) = vmouse();
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        if let Some(waterfall) = &mut self.waterfall {
            let sw = vw();
            let sh = vh();
            waterfall.render_ply(0.0, 0.0, sw, sh);
        }
        self.piano_keyboard.render();

        self.render_score_panel();
        self.render_timing_feedback();
        self.render_song_info(mx, my, mouse_down);
        self.render_vertical_timeline(mx, my, mouse_down);
        self.render_close_button(mx, my, mouse_down);
    }
}

/// PLY Freeplay Scene
pub struct PlyFreeplayScene {
    song: Option<Song>,
    piano_keyboard: PianoKeyboardRenderer,
    mouse_was_pressed: bool,
    soundfonts: Vec<crate::output_manager::SoundFontEntry>,
    current_soundfont_index: usize,
    audio_gain: f32,
    header: Header,
    pending_nav_event: Option<NeothesiaEvent>,
    sidebar_prev_hover: bool,
    sidebar_next_hover: bool,
    sidebar_dec_hover: bool,
    sidebar_inc_hover: bool,
    sidebar_btn_y: f32,
}

impl PlyFreeplayScene {
    pub fn new(song: Option<Song>, ctx: &mut MacroquadContext) -> Self {
        let config = Config::new();
        let range = config.piano_range();
        let keyboard_range = piano_layout::KeyboardRange::new(range.clone());
        let sizing = piano_layout::Sizing::new(40.0, 120.0);
        let layout = KeyboardLayout::from_range(sizing, keyboard_range);

        let piano_keyboard = PianoKeyboardRenderer::new(layout, &config);

        let soundfont_folders = ctx.config.synth_config.soundfont_folders().clone();
        let soundfonts = crate::output_manager::discover_soundfonts(&soundfont_folders);
        let current_soundfont_index = ctx.config.synth_config.soundfont_index().unwrap_or(0);
        let audio_gain = ctx.config.synth_config.audio_gain();

        Self {
            song,
            piano_keyboard,
            mouse_was_pressed: false,
            soundfonts,
            current_soundfont_index,
            audio_gain,
            header: {
                let mut h = Header::new();
                h.set_active_nav(NavItem::FreePlay);
                h.show_back_button = true;
                h
            },
            pending_nav_event: None,
            sidebar_prev_hover: false,
            sidebar_next_hover: false,
            sidebar_dec_hover: false,
            sidebar_inc_hover: false,
            sidebar_btn_y: 0.0,
        }
    }

    fn current_soundfont_name(&self) -> String {
        if let Some(entry) = self.soundfonts.get(self.current_soundfont_index) {
            let file_name = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            let count = self.soundfonts.len();
            if count > 0 {
                format!(
                    "{} ({} of {})",
                    file_name,
                    self.current_soundfont_index + 1,
                    count
                )
            } else {
                file_name.to_string()
            }
        } else {
            "No SoundFont".to_string()
        }
    }

    fn previous_soundfont(&mut self, ctx: &mut MacroquadContext) {
        if self.soundfonts.is_empty() {
            return;
        }
        let count = self.soundfonts.len();
        let new_index = if self.current_soundfont_index == 0 {
            count - 1
        } else {
            self.current_soundfont_index - 1
        };
        self.switch_to_soundfont_index(new_index, ctx);
    }

    fn next_soundfont(&mut self, ctx: &mut MacroquadContext) {
        if self.soundfonts.is_empty() {
            return;
        }
        let count = self.soundfonts.len();
        let new_index = (self.current_soundfont_index + 1) % count;
        self.switch_to_soundfont_index(new_index, ctx);
    }

    fn switch_to_soundfont_index(&mut self, index: usize, ctx: &mut MacroquadContext) {
        if let Some(entry) = self.soundfonts.get(index) {
            self.current_soundfont_index = index;

            if !ctx.output_manager.is_synth_output() {
                log::warn!("SoundFont switching only available with synth output");
                return;
            }

            if let Err(e) = ctx.output_manager.switch_soundfont(&entry.path) {
                log::error!("Failed to switch SoundFont: {}", e);
            }
            ctx.config
                .synth_config
                .set_soundfont_path(Some(entry.path.clone()));
            ctx.config.synth_config.set_soundfont_index(Some(index));
            let _ = ctx.config.save();
        }
    }

    fn decrease_audio_gain(&mut self, ctx: &mut MacroquadContext) {
        self.audio_gain = (self.audio_gain - 0.1).max(0.0);
        ctx.config.synth_config.set_audio_gain(self.audio_gain);
        ctx.output_manager.connection().set_gain(self.audio_gain);
        let _ = ctx.config.save();
    }

    fn increase_audio_gain(&mut self, ctx: &mut MacroquadContext) {
        self.audio_gain = self.audio_gain + 0.1;
        ctx.config.synth_config.set_audio_gain(self.audio_gain);
        ctx.output_manager.connection().set_gain(self.audio_gain);
        let _ = ctx.config.save();
    }

    fn render_freeplay_sidebar(&mut self, mx: f32, my: f32, mouse_pressed: bool) {
        let screen_h = vh();
        let sidebar_w = SIDEBAR_WIDTH;
        let top_offset = self.header.height;
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            0.0,
            top_offset,
            sidebar_w,
            screen_h - top_offset,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );
        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            sidebar_w - 1.0,
            top_offset,
            1.0,
            screen_h - top_offset,
            Color::new(border_r, border_g, border_b, 0.1),
        );
        use crate::scene::ply_fonts;
        let cx = spacing::LG;
        let mut cy = top_offset + spacing::XL + 20.0;
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Free Play",
            cx,
            cy,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );
        cy += 24.0;
        if let Some(ref song) = self.song {
            let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            let name = &song.file.name;
            let display = if name.len() > 24 {
                format!("{}...", &name[..21])
            } else {
                name.clone()
            };
            ply_fonts::draw_body(&display, cx, cy, 12.0, Color::new(sub_r, sub_g, sub_b, 1.0));
        }
        cy += 28.0;
        let (div_r, div_g, div_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            spacing::SM,
            cy,
            sidebar_w - spacing::LG,
            1.0,
            Color::new(div_r, div_g, div_b, 0.2),
        );
        cy += spacing::LG;
        let (sec_r, sec_g, sec_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_label(
            "SOUNDFONT",
            cx,
            cy,
            10.0,
            Color::new(sec_r, sec_g, sec_b, 0.7),
        );
        cy += 20.0;
        let sf_name = self.current_soundfont_name();
        let (val_r, val_g, val_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_body(&sf_name, cx, cy, 12.0, Color::new(val_r, val_g, val_b, 1.0));
        cy += 24.0;
        let btn_w = 100.0;
        let btn_h = 28.0;
        let btn_gap = 8.0;
        let (btn_r, btn_g, btn_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        let prev_x = cx;
        let prev_hover = mx >= prev_x && mx <= prev_x + btn_w && my >= cy && my <= cy + btn_h;
        draw_rectangle(
            prev_x,
            cy,
            btn_w,
            btn_h,
            Color::new(btn_r, btn_g, btn_b, if prev_hover { 1.0 } else { 0.7 }),
        );
        let (pbtn_r, pbtn_g, pbtn_b) = colors::to_normalized(colors::ON_SURFACE);
        let prev_label = "\u{25C0} Prev";
        let prev_tw = measure_text(prev_label, ply_fonts::body_font(), 11, 1.0).width;
        ply_fonts::draw_body(
            prev_label,
            prev_x + (btn_w - prev_tw) / 2.0,
            cy + 19.0,
            11.0,
            Color::new(pbtn_r, pbtn_g, pbtn_b, 1.0),
        );
        let next_x = cx + btn_w + btn_gap;
        let next_hover = mx >= next_x && mx <= next_x + btn_w && my >= cy && my <= cy + btn_h;
        draw_rectangle(
            next_x,
            cy,
            btn_w,
            btn_h,
            Color::new(btn_r, btn_g, btn_b, if next_hover { 1.0 } else { 0.7 }),
        );
        let next_label = "Next \u{25B6}";
        let next_tw = measure_text(next_label, ply_fonts::body_font(), 11, 1.0).width;
        ply_fonts::draw_body(
            next_label,
            next_x + (btn_w - next_tw) / 2.0,
            cy + 19.0,
            11.0,
            Color::new(pbtn_r, pbtn_g, pbtn_b, 1.0),
        );
        cy += btn_h + spacing::LG;
        draw_rectangle(
            spacing::SM,
            cy,
            sidebar_w - spacing::LG,
            1.0,
            Color::new(div_r, div_g, div_b, 0.2),
        );
        cy += spacing::LG;
        ply_fonts::draw_label(
            "AUDIO GAIN",
            cx,
            cy,
            10.0,
            Color::new(sec_r, sec_g, sec_b, 0.7),
        );
        cy += 20.0;
        let gain_text = format!("{:.0}%", self.audio_gain * 100.0);
        ply_fonts::draw_headline(
            &gain_text,
            cx,
            cy,
            24.0,
            Color::new(val_r, val_g, val_b, 1.0),
        );
        cy += 32.0;
        let gain_btn_w = 100.0;
        let dec_x = cx;
        let dec_hover = mx >= dec_x && mx <= dec_x + gain_btn_w && my >= cy && my <= cy + btn_h;
        draw_rectangle(
            dec_x,
            cy,
            gain_btn_w,
            btn_h,
            Color::new(btn_r, btn_g, btn_b, if dec_hover { 1.0 } else { 0.7 }),
        );
        let dec_label = "\u{2212} Volume";
        let dec_tw = measure_text(dec_label, ply_fonts::body_font(), 11, 1.0).width;
        ply_fonts::draw_body(
            dec_label,
            dec_x + (gain_btn_w - dec_tw) / 2.0,
            cy + 19.0,
            11.0,
            Color::new(pbtn_r, pbtn_g, pbtn_b, 1.0),
        );
        let inc_x = cx + gain_btn_w + btn_gap;
        let inc_hover = mx >= inc_x && mx <= inc_x + gain_btn_w && my >= cy && my <= cy + btn_h;
        draw_rectangle(
            inc_x,
            cy,
            gain_btn_w,
            btn_h,
            Color::new(btn_r, btn_g, btn_b, if inc_hover { 1.0 } else { 0.7 }),
        );
        let inc_label = "+ Volume";
        let inc_tw = measure_text(inc_label, ply_fonts::body_font(), 11, 1.0).width;
        ply_fonts::draw_body(
            inc_label,
            inc_x + (gain_btn_w - inc_tw) / 2.0,
            cy + 19.0,
            11.0,
            Color::new(pbtn_r, pbtn_g, pbtn_b, 1.0),
        );
        self.sidebar_prev_hover = prev_hover;
        self.sidebar_next_hover = next_hover;
        self.sidebar_dec_hover = dec_hover;
        self.sidebar_inc_hover = inc_hover;
        self.sidebar_btn_y = cy;
    }
}
impl PlyScene for PlyFreeplayScene {
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent> {
        use midi_file::midly::MidiMessage;

        if let Some(event) = self.pending_nav_event.take() {
            return Some(event);
        }

        let dt = delta.as_secs_f32();
        let (mouse_x, mouse_y) = vmouse();
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_just_pressed = is_mouse_button_pressed(MouseButton::Left);

        if is_key_pressed(KeyCode::Escape) {
            if let Some(ref song) = self.song {
                let entry = song_entry_from_song(song);
                return Some(NeothesiaEvent::ShowSongSelected {
                    song: song.clone(),
                    entry,
                });
            } else {
                return Some(NeothesiaEvent::ShowSongLibrary(None));
            }
        }

        if mouse_just_pressed {
            if self.sidebar_prev_hover {
                self.previous_soundfont(ctx);
            }
            if self.sidebar_next_hover {
                self.next_soundfont(ctx);
            }
            if self.sidebar_dec_hover {
                self.decrease_audio_gain(ctx);
            }
            if self.sidebar_inc_hover {
                self.increase_audio_gain(ctx);
            }
        }

        let content_x = SIDEBAR_WIDTH;
        let is_over_keyboard = mouse_x >= content_x && mouse_y >= self.header.height;

        if is_over_keyboard {
            if mouse_down && !self.mouse_was_pressed {
                if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                    Vec2::new(mouse_x, mouse_y),
                    MouseButton::Left,
                    true,
                ) {
                    for note in notes {
                        let message = MidiMessage::NoteOn {
                            key: midi_file::midly::num::u7::new(note),
                            vel: midi_file::midly::num::u7::new(100),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            } else if !mouse_down && self.mouse_was_pressed {
                if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                    Vec2::new(mouse_x, mouse_y),
                    MouseButton::Left,
                    false,
                ) {
                    for note in notes {
                        let message = MidiMessage::NoteOff {
                            key: midi_file::midly::num::u7::new(note),
                            vel: midi_file::midly::num::u7::new(0),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            } else if mouse_down && self.mouse_was_pressed {
                if let Some(notes) = self
                    .piano_keyboard
                    .handle_mouse_drag(Vec2::new(mouse_x, mouse_y))
                {
                    for note in notes {
                        let message = MidiMessage::NoteOn {
                            key: midi_file::midly::num::u7::new(note),
                            vel: midi_file::midly::num::u7::new(100),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }
            }
        }

        self.mouse_was_pressed = mouse_down;
        self.piano_keyboard.update(dt);
        None
    }

    fn handle_midi_event(&mut self, _channel: u8, message: &midi_file::midly::MidiMessage) {
        use midi_file::midly::MidiMessage;
        match message {
            MidiMessage::NoteOn { key, vel } => {
                self.piano_keyboard
                    .handle_note_event(key.as_int(), vel.as_int());
            }
            MidiMessage::NoteOff { key, .. } => {
                self.piano_keyboard.handle_note_event(key.as_int(), 0);
            }
            _ => {}
        }
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        clear_background(Color::new(0.055, 0.055, 0.075, 1.0));
        let screen_w = vw();
        let screen_h = vh();
        let (mx, my) = vmouse();
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let header_h = 64.0;
        let content_x = SIDEBAR_WIDTH;
        let content_w = screen_w - SIDEBAR_WIDTH;
        let content_h = screen_h - header_h;
        let kb_h = content_h * 0.2;
        let kb_y = header_h + content_h - kb_h - 20.0;
        let kb_w = content_w * 0.95;
        let kb_x = content_x + (content_w - kb_w) / 2.0;
        self.piano_keyboard
            .set_position_and_size(kb_x, kb_y, kb_w, kb_h);
        self.piano_keyboard.render();
        self.render_freeplay_sidebar(mx, my, mouse_pressed);
        let header_nav = self.header.render(mx, my, mouse_pressed);
        if let Some(nav) = header_nav {
            match nav {
                NavItem::Back => {
                    if let Some(ref song) = self.song {
                        let entry = song_entry_from_song(song);
                        self.pending_nav_event = Some(NeothesiaEvent::ShowSongSelected {
                            song: song.clone(),
                            entry,
                        });
                    } else {
                        self.pending_nav_event = Some(NeothesiaEvent::ShowSongLibrary(None));
                    }
                }
                NavItem::Library => {
                    self.pending_nav_event =
                        Some(NeothesiaEvent::ShowSongLibrary(self.song.clone()));
                }
                NavItem::FreePlay => {}
                NavItem::Settings => {
                    self.pending_nav_event = Some(NeothesiaEvent::ShowSettings);
                }
                NavItem::Practice => {}
            }
        }
    }
}

fn format_score_display(score: u64) -> String {
    let s = score.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    format!("Score: {}", result.chars().rev().collect::<String>())
}

/// PLY Score Scene
pub struct PlyScoreScene {
    song: Song,
    score_data: crate::scoring_data::ScoreData,
}

impl PlyScoreScene {
    pub fn new(song: Song, score_data: crate::scoring_data::ScoreData) -> Self {
        Self { song, score_data }
    }

    pub fn from_live_score(song: Song, live_score: &crate::scoring::LiveScoreTracker) -> Self {
        let result = live_score.to_score_data();

        Self {
            song,
            score_data: crate::scoring_data::ScoreData {
                total_notes: result.total_notes as usize,
                correct_notes: (result.total_notes - result.miss_count) as usize,
                missed_notes: result.miss_count as usize,
                too_early: 0,
                too_late: 0,
                on_time: (result.perfect_count + result.good_count + result.okay_count) as usize,
                accuracy: result.accuracy,
                grade: result.grade().to_string(),
                stars: result.stars.count(),
                max_streak: result.max_streak,
                score: result.score,
                perfect_count: result.perfect_count,
                good_count: result.good_count,
                okay_count: result.okay_count,
            },
        }
    }

    /// Draw text with headline font (Space Grotesk) - delegates to shared module
    fn draw_headline(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        crate::scene::ply_fonts::draw_headline(text, x, y, size, color);
    }

    /// Draw text with body font (Inter) - delegates to shared module
    fn draw_body(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        crate::scene::ply_fonts::draw_body(text, x, y, size, color);
    }

    /// Calculate grade from accuracy
    fn grade(&self) -> &str {
        &self.score_data.grade
    }

    /// Get grade color (purple for high grades)
    fn grade_color(&self) -> Color {
        match self.score_data.grade.as_str() {
            "S" => Color::from_rgba(219, 144, 255, 255), // primary #db90ff
            "A" | "A+" => Color::from_rgba(219, 144, 255, 255),
            "B" => Color::from_rgba(95, 158, 255, 255), // secondary
            "C" => Color::from_rgba(100, 200, 120, 255),
            "D" => Color::from_rgba(220, 160, 80, 255),
            _ => Color::from_rgba(255, 110, 128, 255), // tertiary
        }
    }

    /// Draw a glassmorphism-style card with rounded corners
    fn draw_glass_card(x: f32, y: f32, w: f32, h: f32, color: &Color) {
        use macroquad::prelude::*;

        let border_radius = 12.0;

        // Draw rounded rectangle background
        draw_rectangle(x + border_radius, y, w - 2.0 * border_radius, h, *color);
        draw_rectangle(x, y + border_radius, w, h - 2.0 * border_radius, *color);

        // Draw corner circles to complete rounded shape
        draw_circle(x + border_radius, y + border_radius, border_radius, *color);
        draw_circle(
            x + w - border_radius,
            y + border_radius,
            border_radius,
            *color,
        );
        draw_circle(
            x + border_radius,
            y + h - border_radius,
            border_radius,
            *color,
        );
        draw_circle(
            x + w - border_radius,
            y + h - border_radius,
            border_radius,
            *color,
        );
    }

    /// Draw an accuracy progress bar
    fn draw_accuracy_bar(x: f32, y: f32, w: f32, h: f32, pct: f32, color: &Color) {
        // Background
        draw_rectangle(x, y, w, h, Color::from_rgba(37, 37, 44, 255));
        // Fill
        draw_rectangle(x, y, w * pct.min(1.0), h, *color);
    }

    /// Check if mouse is in a rectangle
    fn is_mouse_in_rect(x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mx, my) = vmouse();
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }
}

impl PlyScene for PlyScoreScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        // Keyboard shortcuts
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            return Some(NeothesiaEvent::MainMenu(None));
        }

        if is_key_pressed(KeyCode::R) {
            return Some(NeothesiaEvent::StartSession {
                song: self.song.clone(),
                config: crate::common::SessionConfig::default(),
            });
        }

        // Mouse click handling
        if is_mouse_button_pressed(MouseButton::Left) {
            let screen_w = vw();
            let margin = 40.0;
            let content_w = (screen_w - margin * 2.0).min(1200.0);
            let content_x = (screen_w - content_w) / 2.0;

            // Right column buttons
            let right_col_w = content_w * 0.25;
            let right_x = content_x + content_w * 0.35 + 16.0 + content_w * 0.40 + 16.0;

            let main_y = 60.0 + 140.0; // header_y + 140.0
            let timeline_h = 200.0;
            let btn_y = main_y + timeline_h + 16.0;
            let btn_h = 48.0;
            let btn_gap = 12.0;

            // PLAY AGAIN button
            if Self::is_mouse_in_rect(right_x, btn_y, right_col_w, btn_h) {
                return Some(NeothesiaEvent::StartSession {
                    song: self.song.clone(),
                    config: crate::common::SessionConfig::default(),
                });
            }

            // NEXT SONG button
            let next_y = btn_y + btn_h + btn_gap;
            if Self::is_mouse_in_rect(right_x, next_y, right_col_w, btn_h) {
                return Some(NeothesiaEvent::MainMenu(None));
            }

            // VIEW REPLAY button (same as PLAY AGAIN)
            let replay_y = next_y + btn_h + btn_gap;
            if Self::is_mouse_in_rect(right_x, replay_y, right_col_w, 40.0) {
                return Some(NeothesiaEvent::StartSession {
                    song: self.song.clone(),
                    config: crate::common::SessionConfig::default(),
                });
            }
        }

        None
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        // Sonic Obsidian Design System Colors
        let bg_color = Color::from_rgba(14, 14, 19, 255); // #0e0e13
        let surface_low = Color::from_rgba(19, 19, 24, 255); // #131318
        let surface = Color::from_rgba(25, 25, 31, 255); // #19191f
        let surface_high = Color::from_rgba(37, 37, 44, 255); // #25252c
        let surface_bright = Color::from_rgba(44, 43, 51, 255); // #2c2b33

        let primary = Color::from_rgba(219, 144, 255, 255); // #db90ff
        let primary_dim = Color::from_rgba(210, 119, 255, 255); // #d277ff
        let secondary = Color::from_rgba(95, 158, 255, 255); // #5f9eff
        let tertiary = Color::from_rgba(255, 110, 128, 255); // #ff6e80

        let on_surface = Color::from_rgba(248, 245, 253, 255); // #f8f5fd
        let on_surface_var = Color::from_rgba(172, 170, 177, 255); // #acaab1
        let error = Color::from_rgba(255, 110, 132, 255); // #ff6e84

        clear_background(bg_color);

        let screen_w = vw();
        let screen_h = vh();

        // Layout constants
        let margin = 40.0;
        let card_gap = 16.0;
        let content_w = (screen_w - margin * 2.0).min(1200.0);
        let content_x = (screen_w - content_w) / 2.0;

        // Header section
        let header_y = 60.0;

        // "PERFORMANCE COMPLETE" label (Space Grotesk - headline)
        self.draw_body(
            "PERFORMANCE COMPLETE",
            content_x + content_w / 2.0 - 130.0,
            header_y,
            14.0,
            primary,
        );

        // Song title (Space Grotesk - headline)
        let title = &self.song.file.name;
        let title_size = 48.0;
        self.draw_headline(
            title,
            content_x + content_w / 2.0 - (title.len() as f32 * 12.0),
            header_y + 50.0,
            title_size,
            on_surface,
        );

        // Artist placeholder (Space Grotesk - headline)
        self.draw_body(
            "Frédéric Chopin",
            content_x + content_w / 2.0 - 70.0,
            header_y + 90.0,
            18.0,
            on_surface_var,
        );

        // Main content area starts here
        let main_y = header_y + 140.0;
        let card_h = 200.0;

        // === LEFT COLUMN: Grade Card + VS Personal Best ===
        let left_col_w = content_w * 0.35;
        let left_x = content_x;

        // Grade Card
        Self::draw_glass_card(left_x, main_y, left_col_w, card_h, &surface_high);

        // Grade display (A+) - Space Grotesk headline
        let grade_color = self.grade_color();
        self.draw_headline(
            "A+",
            left_x + left_col_w / 2.0 - 35.0,
            main_y + 60.0,
            64.0,
            grade_color,
        );

        // Score - Space Grotesk headline
        let score_str = format_score_display(self.score_data.score);
        self.draw_headline(
            &score_str,
            left_x + left_col_w / 2.0 - (score_str.len() as f32 * 8.0),
            main_y + 110.0,
            28.0,
            on_surface,
        );
        // TOTAL SCORE - Inter body (uppercase label)
        self.draw_body(
            "TOTAL SCORE",
            left_x + left_col_w / 2.0 - 50.0,
            main_y + 135.0,
            12.0,
            on_surface_var,
        );

        // NEW PERSONAL BEST badge
        let badge_w = 160.0;
        let badge_x = left_x + left_col_w / 2.0 - badge_w / 2.0;
        let badge_y = main_y + 155.0;
        draw_rectangle(
            badge_x,
            badge_y,
            badge_w,
            24.0,
            Color::from_rgba(219, 144, 255, 40),
        );
        draw_rectangle_lines(
            badge_x,
            badge_y,
            badge_w,
            24.0,
            1.0,
            Color::from_rgba(219, 144, 255, 80),
        );
        self.draw_body(
            "NEW PERSONAL BEST!",
            badge_x + 12.0,
            badge_y + 16.0,
            10.0,
            primary,
        );

        // VS Personal Best Card
        let vs_card_y = main_y + card_h + card_gap;
        let vs_card_h = 80.0;
        Self::draw_glass_card(left_x, vs_card_y, left_col_w, vs_card_h, &surface_high);

        self.draw_body(
            "VS PERSONAL BEST",
            left_x + 20.0,
            vs_card_y + 25.0,
            10.0,
            on_surface_var,
        );
        self.draw_headline(
            "+15,000 pts",
            left_x + 20.0,
            vs_card_y + 55.0,
            20.0,
            secondary,
        );

        // === CENTER COLUMN: Accuracy Breakdown + Stats ===
        let center_col_w = content_w * 0.40;
        let center_x = left_x + left_col_w + card_gap;

        // Accuracy Breakdown Card
        let acc_card_h = 200.0;
        Self::draw_glass_card(center_x, main_y, center_col_w, acc_card_h, &surface_high);

        self.draw_body(
            "ACCURACY BREAKDOWN",
            center_x + 20.0,
            main_y + 25.0,
            10.0,
            on_surface_var,
        );

        // Progress bars for accuracy categories
        let bar_start_y = main_y + 50.0;
        let bar_h = 8.0;
        let bar_gap = 35.0;
        let total = self.score_data.total_notes as f32;

        let perfect_pct = if total > 0.0 {
            self.score_data.perfect_count as f32 / total
        } else {
            0.0
        };
        let good_pct = if total > 0.0 {
            self.score_data.good_count as f32 / total
        } else {
            0.0
        };
        let okay_pct = if total > 0.0 {
            self.score_data.okay_count as f32 / total
        } else {
            0.0
        };
        let miss_pct = if total > 0.0 {
            self.score_data.missed_notes as f32 / total
        } else {
            0.0
        };

        // Perfect bar
        Self::draw_accuracy_bar(
            center_x + 20.0,
            bar_start_y,
            center_col_w - 120.0,
            bar_h,
            perfect_pct,
            &primary,
        );
        self.draw_body(
            "Perfect",
            center_x + center_col_w - 90.0,
            bar_start_y + 8.0,
            12.0,
            on_surface,
        );
        self.draw_headline(
            &self.score_data.perfect_count.to_string(),
            center_x + center_col_w - 30.0,
            bar_start_y + 8.0,
            12.0,
            primary,
        );

        // Great/Good bar
        Self::draw_accuracy_bar(
            center_x + 20.0,
            bar_start_y + bar_gap,
            center_col_w - 120.0,
            bar_h,
            good_pct,
            &secondary,
        );
        self.draw_body(
            "Great",
            center_x + center_col_w - 90.0,
            bar_start_y + bar_gap + 8.0,
            12.0,
            on_surface,
        );
        self.draw_headline(
            &self.score_data.good_count.to_string(),
            center_x + center_col_w - 30.0,
            bar_start_y + bar_gap + 8.0,
            12.0,
            secondary,
        );

        // Good/Okay bar
        Self::draw_accuracy_bar(
            center_x + 20.0,
            bar_start_y + bar_gap * 2.0,
            center_col_w - 120.0,
            bar_h,
            okay_pct,
            &on_surface_var,
        );
        self.draw_body(
            "Good",
            center_x + center_col_w - 90.0,
            bar_start_y + bar_gap * 2.0 + 8.0,
            12.0,
            on_surface,
        );
        self.draw_headline(
            &self.score_data.okay_count.to_string(),
            center_x + center_col_w - 30.0,
            bar_start_y + bar_gap * 2.0 + 8.0,
            12.0,
            on_surface_var,
        );

        // Miss bar
        Self::draw_accuracy_bar(
            center_x + 20.0,
            bar_start_y + bar_gap * 3.0,
            center_col_w - 120.0,
            bar_h,
            miss_pct,
            &error,
        );
        self.draw_body(
            "Miss",
            center_x + center_col_w - 90.0,
            bar_start_y + bar_gap * 3.0 + 8.0,
            12.0,
            on_surface,
        );
        self.draw_headline(
            &self.score_data.missed_notes.to_string(),
            center_x + center_col_w - 30.0,
            bar_start_y + bar_gap * 3.0 + 8.0,
            12.0,
            error,
        );

        // Stats row (MAX STREAK + MAX COMBO)
        let stats_y = main_y + acc_card_h + card_gap;
        let stats_h = 80.0;
        let stats_half_w = (center_col_w - card_gap) / 2.0;

        // MAX STREAK card
        Self::draw_glass_card(center_x, stats_y, stats_half_w, stats_h, &surface_high);
        self.draw_body(
            "MAX STREAK",
            center_x + stats_half_w / 2.0 - 45.0,
            stats_y + 25.0,
            10.0,
            on_surface_var,
        );
        self.draw_headline(
            &format!("x{}", self.score_data.max_streak),
            center_x + stats_half_w / 2.0 - 25.0,
            stats_y + 60.0,
            24.0,
            primary,
        );

        // MAX COMBO card
        Self::draw_glass_card(
            center_x + stats_half_w + card_gap,
            stats_y,
            stats_half_w,
            stats_h,
            &surface_high,
        );
        self.draw_body(
            "MAX COMBO",
            center_x + stats_half_w + card_gap + stats_half_w / 2.0 - 40.0,
            stats_y + 25.0,
            10.0,
            on_surface_var,
        );
        let combo_text = if self.score_data.max_streak >= 100 {
            "UNSTOPPABLE"
        } else if self.score_data.max_streak >= 50 {
            "ON FIRE"
        } else {
            "COMBO"
        };
        self.draw_headline(
            combo_text,
            center_x + stats_half_w + card_gap + stats_half_w / 2.0
                - (combo_text.len() as f32 * 4.0),
            stats_y + 60.0,
            16.0,
            tertiary,
        );

        // === RIGHT COLUMN: Timeline + Actions ===
        let right_col_w = content_w * 0.25;
        let right_x = center_x + center_col_w + card_gap;

        // Accuracy Timeline Card
        let timeline_h = 200.0;
        Self::draw_glass_card(right_x, main_y, right_col_w, timeline_h, &surface_high);

        self.draw_body(
            "ACCURACY TIMELINE",
            right_x + 20.0,
            main_y + 25.0,
            10.0,
            on_surface_var,
        );

        // Draw bar chart for timeline
        let chart_start_y = main_y + 50.0;
        let chart_h = 120.0;
        let bar_count = 12;
        let chart_bar_w = (right_col_w - 40.0) / bar_count as f32 - 2.0;

        for i in 0..bar_count {
            let bar_x = right_x + 20.0 + i as f32 * (chart_bar_w + 2.0);
            let bar_ratio = 0.5 + (i as f32 * 0.05).min(0.5);
            let h = chart_h * bar_ratio;
            let bar_color = if i == 5 {
                error
            } else {
                Color::from_rgba(219, 144, 255, 100)
            };
            draw_rectangle(
                bar_x,
                chart_start_y + chart_h - h,
                chart_bar_w,
                h,
                bar_color,
            );
        }

        self.draw_body(
            "START",
            right_x + 20.0,
            chart_start_y + chart_h + 15.0,
            8.0,
            on_surface_var,
        );
        self.draw_body(
            "04:32",
            right_x + right_col_w - 50.0,
            chart_start_y + chart_h + 15.0,
            8.0,
            on_surface_var,
        );

        // Action buttons
        let btn_y = main_y + timeline_h + card_gap;
        let btn_h = 48.0;
        let btn_gap = 12.0;

        // PLAY AGAIN button (gradient style)
        let play_again_hover = Self::is_mouse_in_rect(right_x, btn_y, right_col_w, btn_h);
        let play_again_color = if play_again_hover {
            Color::from_rgba(211, 123, 255, 255) // brighter on hover
        } else {
            primary
        };
        draw_rectangle(right_x, btn_y, right_col_w, btn_h, play_again_color);
        self.draw_headline(
            "PLAY AGAIN",
            right_x + right_col_w / 2.0 - 45.0,
            btn_y + 30.0,
            14.0,
            bg_color,
        );

        // NEXT SONG button (outlined style)
        let next_y = btn_y + btn_h + btn_gap;
        let next_hover = Self::is_mouse_in_rect(right_x, next_y, right_col_w, btn_h);
        let next_bg = if next_hover {
            surface_bright
        } else {
            surface_high
        };
        draw_rectangle(right_x, next_y, right_col_w, btn_h, next_bg);
        draw_rectangle_lines(
            right_x,
            next_y,
            right_col_w,
            btn_h,
            1.0,
            Color::from_rgba(72, 71, 77, 50),
        );
        self.draw_headline(
            "NEXT SONG",
            right_x + right_col_w / 2.0 - 42.0,
            next_y + 30.0,
            14.0,
            on_surface,
        );

        // VIEW REPLAY button (text only)
        let replay_y = next_y + btn_h + btn_gap;
        let replay_hover = Self::is_mouse_in_rect(right_x, replay_y, right_col_w, 40.0);
        let replay_color = if replay_hover { primary } else { primary_dim };
        self.draw_body(
            "VIEW REPLAY",
            right_x + right_col_w / 2.0 - 48.0,
            replay_y + 20.0,
            12.0,
            replay_color,
        );

        // === FOOTER STATS ===
        let footer_y = screen_h - 60.0;
        let footer_w = content_w;
        let stat_w = footer_w / 4.0;

        // Time Played
        self.draw_body(
            "TIME PLAYED",
            content_x + 10.0,
            footer_y,
            8.0,
            on_surface_var,
        );
        self.draw_headline("04:32", content_x + 10.0, footer_y + 20.0, 16.0, on_surface);

        // Average Offset
        self.draw_body(
            "AVERAGE OFFSET",
            content_x + stat_w + 10.0,
            footer_y,
            8.0,
            on_surface_var,
        );
        self.draw_headline(
            "+12ms",
            content_x + stat_w + 10.0,
            footer_y + 20.0,
            16.0,
            on_surface,
        );

        // Difficulty
        self.draw_body(
            "DIFFICULTY",
            content_x + stat_w * 2.0 + 10.0,
            footer_y,
            8.0,
            on_surface_var,
        );
        self.draw_headline(
            "MAESTRO",
            content_x + stat_w * 2.0 + 10.0,
            footer_y + 20.0,
            16.0,
            tertiary,
        );

        // Notes Processed
        self.draw_body(
            "NOTES PROCESSED",
            content_x + stat_w * 3.0 + 10.0,
            footer_y,
            8.0,
            on_surface_var,
        );
        self.draw_headline(
            &format!(
                "{} / {}",
                self.score_data.correct_notes, self.score_data.total_notes
            ),
            content_x + stat_w * 3.0 + 10.0,
            footer_y + 20.0,
            16.0,
            on_surface,
        );

        // Keyboard hints
        self.draw_body(
            "ENTER: Main Menu | R: Replay | ESC: Main Menu",
            screen_w / 2.0 - 150.0,
            screen_h - 15.0,
            10.0,
            on_surface_var,
        );
    }
}

pub struct PlyNewSongLibraryScene {
    song: Option<Song>,
    page: crate::settings::pages::song_library::SongLibraryPage,
    pending_event: Option<NeothesiaEvent>,
}

impl PlyNewSongLibraryScene {
    pub fn new(song: Option<Song>) -> Self {
        Self {
            song,
            page: crate::settings::pages::song_library::SongLibraryPage::new(),
            pending_event: None,
        }
    }

    pub fn load_songs(&mut self, ctx: &mut MacroquadContext) {
        use crate::song_library::{FilterState, SortPreference};
        if let Ok(entries) = ctx
            .song_library_db
            .list_songs(&SortPreference::default(), &FilterState::default())
        {
            self.page.load_songs(entries);
            log::info!("🎯 PLY NEW SONG LIBRARY: Loaded songs");
        }
    }
}

impl PlyScene for PlyNewSongLibraryScene {
    fn update(&mut self, ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        if self.page.songs.is_empty() {
            self.load_songs(ctx);
        }

        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::MainMenu(self.song.take()));
        }

        self.pending_event.take()
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        let (mx, my) = vmouse();
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        use crate::settings::pages::song_library::SongLibraryInteraction;

        match self.page.render(mx, my, mouse_pressed, mouse_down) {
            SongLibraryInteraction::SelectSong(entry) => {
                match midi_file::MidiFile::new(&entry.file_path) {
                    Ok(midi_file) => {
                        let mut song = Song::new(midi_file);
                        song.song_id = Some(entry.id);
                        self.pending_event = Some(NeothesiaEvent::ShowSongSelected { song, entry });
                    }
                    Err(e) => {
                        log::error!("Failed to load song from {:?}: {}", entry.file_path, e);
                    }
                }
            }
            SongLibraryInteraction::NavigateToFreePlay => {
                self.pending_event = Some(NeothesiaEvent::FreePlay(self.song.take()));
            }
            SongLibraryInteraction::NavigateToPractice => {
                self.pending_event = Some(NeothesiaEvent::FreePlay(self.song.take()));
            }
            SongLibraryInteraction::NavigateToSettings => {
                self.pending_event = Some(NeothesiaEvent::ShowSettings);
            }
            SongLibraryInteraction::None => {}
        }
    }
}

/// PLY Song Selected Scene
pub struct PlySongSelectedScene {
    song: Option<Song>,
    page: Option<crate::settings::pages::song_selected::SongSelectedPage>,
    pending_event: Option<NeothesiaEvent>,
}

impl PlySongSelectedScene {
    pub fn new(song: Song) -> Self {
        let entry = crate::song_library::SongEntry {
            id: song.song_id.unwrap_or(0),
            file_path: std::path::PathBuf::from(&song.file.name),
            name: song.file.name.clone(),
            difficulty: 5,
            duration_secs: 300,
            track_count: song.file.tracks.len(),
            play_count: 0,
            last_score: None,
            best_score: None,
            last_played_at: None,
            created_at: chrono::Utc::now(),
            genre: Some("Classical".to_string()),
            labels: Vec::new(),
        };

        Self {
            song: Some(song),
            page: Some(crate::settings::pages::song_selected::SongSelectedPage::new(entry)),
            pending_event: None,
        }
    }
}

impl PlyScene for PlySongSelectedScene {
    fn update(&mut self, _ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::ShowSongLibrary(self.song.take()));
        }

        self.pending_event.take()
    }

    fn render(&mut self, _ctx: &mut MacroquadContext) {
        if let Some(page) = &mut self.page {
            let (mx, my) = vmouse();
            let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
            let mouse_down = is_mouse_button_down(MouseButton::Left);

            use crate::settings::pages::song_selected::SongSelectedInteraction;

            match page.render(mx, my, mouse_pressed, mouse_down) {
                SongSelectedInteraction::NavigateBack => {
                    self.pending_event = Some(NeothesiaEvent::ShowSongLibrary(self.song.take()));
                }
                SongSelectedInteraction::NavigateToFreePlay => {
                    self.pending_event = Some(NeothesiaEvent::FreePlay(self.song.take()));
                }
                SongSelectedInteraction::NavigateToSettings => {
                    self.pending_event = Some(NeothesiaEvent::ShowSettings);
                }
                SongSelectedInteraction::ModeSelected(mode) => {
                    log::info!("Mode selected: {:?}", mode);
                }
                SongSelectedInteraction::StartSession { config } => {
                    if let Some(song) = self.song.take() {
                        self.pending_event = Some(NeothesiaEvent::StartSession { song, config });
                    }
                }
                SongSelectedInteraction::None => {}
            }
        }
    }
}
