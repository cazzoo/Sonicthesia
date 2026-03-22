//! PLY-specific scene implementations for Macroquad rendering
//!
//! This module provides PLY rendering implementations of all scenes,
//! adapted from the WGPU versions to work with MacroquadContext.

use crate::{
    context_macroquad::MacroquadContext,
    effects::{EffectsManager, ScreenFlash, ScreenShake},
    scoring::{LiveScoreTracker, StreakMilestone, TimingQuality},
    song::Song,
    song_library::SongRepository,
    NeothesiaEvent,
};
use std::time::Duration;

use crate::input_stubs::{ElementType, FocusableElement, InputPriority, UnifiedInputManager};
use crate::render::ply::PianoKeyboardRenderer;
use macroquad::prelude::*;
use neothesia_core::config::Config;
use piano_layout::KeyboardLayout;

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
                            return Some(NeothesiaEvent::Play(song));
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
        let (mouse_x, mouse_y) = mouse_position();

        // Update mouse position in unified input manager
        self.input_manager
            .focus()
            .priority()
            .update_mouse_position(mouse_x, mouse_y);

        let screen_w = screen_width();
        let screen_h = screen_height();
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

        let screen_w = screen_width();
        let screen_h = screen_height();
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
            let (mouse_x, mouse_y) = mouse_position();
            let screen_w = screen_width();
            let screen_h = screen_height();

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
        let (mouse_x, mouse_y) = mouse_position();
        let screen_w = screen_width();
        let screen_h = screen_height();
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

        let screen_w = screen_width();
        let screen_h = screen_height();

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

        let (mouse_x, mouse_y) = mouse_position();
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
    runtime_gain: f32,
    looper_active: bool,
    looper_start: f32,
    looper_end: f32,
    is_seeking: bool,
    is_dragging_looper_start: bool,
    is_dragging_looper_end: bool,
    wait_mode: bool,

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
            runtime_gain,
            looper_active: false,
            looper_start: 0.0,
            looper_end: 0.0,
            is_seeking: false,
            is_dragging_looper_start: false,
            is_dragging_looper_end: false,
            wait_mode: false,
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
        let sw = screen_width();
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
        let sw = screen_width();
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
        1.0
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
        let sw = screen_width();
        let center_x = sw / 2.0;

        // Song title and artist (top-center)
        let title_y = 40.0;
        crate::scene::ply_fonts::draw_headline(
            &self.song.file.name,
            center_x - 150.0,
            title_y,
            20.0,
            Self::COLOR_ON_SURFACE,
        );

        // Artist/subtitle (simplified - could get from MIDI metadata)
        crate::scene::ply_fonts::draw_body(
            "MIDI Performance",
            center_x - 100.0,
            title_y + 25.0,
            12.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        // Session config bar (pill-shaped container)
        let bar_y = title_y + 55.0;
        let bar_w = 280.0;
        let bar_h = 50.0;
        let bar_x = center_x - bar_w / 2.0;

        // Bar background (surface_container with rounded corners)
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Self::COLOR_SURFACE_CONTAINER);

        // Border (outline_variant at low opacity)
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

        // Speed section with +/- controls
        let speed_x = bar_x + 30.0;
        crate::scene::ply_fonts::draw_label(
            "SPEED",
            speed_x,
            bar_y + 12.0,
            8.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        // - button
        let minus_btn_x = speed_x;
        let minus_btn_y = bar_y + 22.0;
        let btn_size = 20.0;
        draw_rectangle(
            minus_btn_x,
            minus_btn_y,
            btn_size,
            btn_size,
            Self::COLOR_SURFACE_VARIANT,
        );
        crate::scene::ply_fonts::draw_body(
            "-",
            minus_btn_x + 7.0,
            minus_btn_y + 14.0,
            14.0,
            Self::COLOR_ON_SURFACE,
        );

        // Speed percentage
        let speed = self.speed_multiplier();
        let speed_pct = (speed * 100.0).round() as i32;
        crate::scene::ply_fonts::draw_body(
            &format!("{}%", speed_pct),
            speed_x + 25.0,
            bar_y + 38.0,
            14.0,
            Self::COLOR_PRIMARY,
        );

        // + button
        let plus_btn_x = speed_x + 60.0;
        draw_rectangle(
            plus_btn_x,
            minus_btn_y,
            btn_size,
            btn_size,
            Self::COLOR_SURFACE_VARIANT,
        );
        crate::scene::ply_fonts::draw_body(
            "+",
            plus_btn_x + 5.0,
            minus_btn_y + 14.0,
            14.0,
            Self::COLOR_ON_SURFACE,
        );

        // Separator
        draw_rectangle(
            bar_x + 130.0,
            bar_y + 10.0,
            1.0,
            30.0,
            Self::COLOR_OUTLINE_VARIANT,
        );

        // Pause button
        let btn_x = bar_x + 160.0;
        let btn_size = 36.0;
        let btn_center_y = bar_y + bar_h / 2.0;

        // Pause button circle (primary color)
        draw_circle(
            btn_x + btn_size / 2.0,
            btn_center_y,
            btn_size / 2.0,
            Self::COLOR_PRIMARY,
        );

        // Show play or pause icon based on paused state
        if self.paused {
            // Play icon (triangle)
            let tri_size = 10.0;
            let tri_center_x = btn_x + btn_size / 2.0 + 2.0; // Slightly right of center
            let tri_center_y = btn_center_y;
            // Draw triangle using three lines
            draw_line(
                tri_center_x - tri_size / 2.0,
                tri_center_y - tri_size / 2.0,
                tri_center_x + tri_size / 2.0,
                tri_center_y,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_line(
                tri_center_x + tri_size / 2.0,
                tri_center_y,
                tri_center_x - tri_size / 2.0,
                tri_center_y + tri_size / 2.0,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_line(
                tri_center_x - tri_size / 2.0,
                tri_center_y + tri_size / 2.0,
                tri_center_x - tri_size / 2.0,
                tri_center_y - tri_size / 2.0,
                2.0,
                Self::COLOR_SURFACE_CONTAINER,
            );
        } else {
            // Pause icon (two vertical bars)
            let bar_width = 3.0;
            let bar_height = 14.0;
            let bar_gap = 5.0;
            draw_rectangle(
                btn_x + btn_size / 2.0 - bar_gap - bar_width,
                btn_center_y - bar_height / 2.0,
                bar_width,
                bar_height,
                Self::COLOR_SURFACE_CONTAINER,
            );
            draw_rectangle(
                btn_x + btn_size / 2.0 + bar_gap,
                btn_center_y - bar_height / 2.0,
                bar_width,
                bar_height,
                Self::COLOR_SURFACE_CONTAINER,
            );
        }

        // Settings button (gear icon placeholder)
        let settings_x = btn_x + btn_size + 10.0;
        draw_circle(
            settings_x + 12.0,
            btn_center_y,
            15.0,
            Self::COLOR_SURFACE_VARIANT,
        );
        crate::scene::ply_fonts::draw_body(
            "⚙",
            settings_x + 6.0,
            btn_center_y + 5.0,
            14.0,
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

        let sw = screen_width();
        let center_x = sw / 2.0;
        let title_y = 40.0;
        let bar_y = title_y + 55.0;
        let bar_w = 280.0;
        let bar_h = 50.0;
        let bar_x = center_x - bar_w / 2.0;

        // Check speed - button click
        let speed_x = bar_x + 30.0;
        let btn_size = 20.0;
        let minus_btn_x = speed_x;
        let minus_btn_y = bar_y + 22.0;
        if mx >= minus_btn_x
            && mx <= minus_btn_x + btn_size
            && my >= minus_btn_y
            && my <= minus_btn_y + btn_size
        {
            let current_speed = ctx.config.speed_multiplier();
            ctx.config.set_speed_multiplier(current_speed - 0.1);
            return None;
        }

        // Check speed + button click
        let plus_btn_x = speed_x + 60.0;
        if mx >= plus_btn_x
            && mx <= plus_btn_x + btn_size
            && my >= minus_btn_y
            && my <= minus_btn_y + btn_size
        {
            let current_speed = ctx.config.speed_multiplier();
            ctx.config.set_speed_multiplier(current_speed + 0.1);
            return None;
        }

        // Check pause button click
        let pause_btn_x = bar_x + 160.0;
        let pause_btn_size = 36.0;
        let pause_btn_center_y = bar_y + bar_h / 2.0;
        let pause_btn_cx = pause_btn_x + pause_btn_size / 2.0;
        let pause_dist = ((mx - pause_btn_cx).powi(2) + (my - pause_btn_center_y).powi(2)).sqrt();
        if pause_dist <= pause_btn_size / 2.0 {
            self.paused = !self.paused;
            return None;
        }

        // Check settings button click
        let settings_x = pause_btn_x + pause_btn_size + 10.0;
        let settings_cx = settings_x + 12.0;
        let settings_dist = ((mx - settings_cx).powi(2) + (my - pause_btn_center_y).powi(2)).sqrt();
        if settings_dist <= 15.0 {
            self.paused = true;
            ctx.resume_playback_time = Some(self.playback_time);
            ctx.resume_song = Some(self.song.clone());
            return Some(NeothesiaEvent::ShowSettings);
        }

        None
    }

    /// Render vertical timeline progress bar on left side
    fn render_vertical_timeline(&self, mx: f32, my: f32, mouse_down: bool) {
        let screen_h = screen_height();
        let center_y = screen_h / 2.0;

        // Timeline position (left side)
        let timeline_x = 40.0;
        let timeline_top = center_y - 150.0;
        let timeline_bottom = center_y + 150.0;
        let timeline_h = timeline_bottom - timeline_top;

        // Time labels
        let current_time = self.playback_time.max(0.0);
        let remaining_time = (self.song_length - self.playback_time).max(0.0);

        crate::scene::ply_fonts::draw_mono(
            &Self::format_time(remaining_time),
            timeline_x - 5.0,
            timeline_top - 10.0,
            10.0,
            Self::COLOR_ON_SURFACE_VARIANT,
        );

        crate::scene::ply_fonts::draw_mono(
            &Self::format_time(current_time),
            timeline_x - 5.0,
            timeline_bottom + 15.0,
            10.0,
            Self::COLOR_PRIMARY,
        );

        // Timeline track (surface_container_highest)
        draw_rectangle(
            timeline_x - 2.0,
            timeline_top,
            4.0,
            timeline_h,
            Self::COLOR_SURFACE_CONTAINER_HIGHEST,
        );

        // Progress fill (gradient from primary to secondary)
        let progress = self.percentage();
        let fill_h = timeline_h * progress;
        draw_rectangle(
            timeline_x - 2.0,
            timeline_bottom - fill_h,
            4.0,
            fill_h,
            Self::COLOR_PRIMARY,
        );

        // Thumb/handle (circle with glow effect)
        let thumb_y = timeline_bottom - fill_h;
        draw_circle(timeline_x, thumb_y, 8.0, Self::COLOR_PRIMARY);

        // Glow effect
        draw_circle(
            timeline_x,
            thumb_y,
            12.0,
            Color::new(0.859, 0.565, 1.0, 0.3),
        );
    }

    /// Handle vertical timeline click/drag for seeking
    fn handle_vertical_timeline_click(
        &mut self,
        mx: f32,
        my: f32,
        mouse_down: bool,
        mouse_just_pressed: bool,
    ) {
        let screen_h = screen_height();
        let center_y = screen_h / 2.0;
        let timeline_x = 40.0;
        let timeline_top = center_y - 150.0;
        let timeline_bottom = center_y + 150.0;
        let timeline_h = timeline_bottom - timeline_top;

        // Check if mouse is near the timeline area (10px horizontal tolerance)
        if mx < timeline_x - 15.0 || mx > timeline_x + 15.0 {
            if !mouse_down {
                self.is_seeking = false;
            }
            return;
        }

        // Check if mouse is within vertical bounds of timeline
        if my < timeline_top - 20.0 || my > timeline_bottom + 20.0 {
            if !mouse_down {
                self.is_seeking = false;
            }
            return;
        }

        if mouse_just_pressed {
            self.is_seeking = true;
        }

        if self.is_seeking && mouse_down {
            // Calculate percentage (inverted because bottom = 0%, top = 100%)
            let p = ((timeline_bottom - my) / timeline_h).clamp(0.0, 1.0);
            self.set_percentage(p);
            if let Some(waterfall) = &mut self.waterfall {
                waterfall.update(self.playback_time);
            }
        }

        if !mouse_down {
            self.is_seeking = false;
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

            let screen_w = screen_width();
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
        let screen_w = screen_width();
        let screen_h = screen_height();

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
        let screen_w = screen_width();
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
        let screen_w = screen_width();
        let btn_x = screen_w - 60.0;
        let btn_y = 32.0;
        let btn_size = 40.0;

        if mx >= btn_x && mx <= btn_x + btn_size && my >= btn_y && my <= btn_y + btn_size {
            Some(NeothesiaEvent::MainMenu(Some(self.song.clone())))
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
        let sw = screen_width();
        let btn_size = Self::TOP_BAR_H;

        // ── Back button ──
        if mx >= 0.0 && mx <= btn_size && my >= 0.0 && my <= btn_size {
            return Some(NeothesiaEvent::MainMenu(Some(self.song.clone())));
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
        let sw = screen_width();
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
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_just_pressed = is_mouse_button_pressed(MouseButton::Left);

        if self.waterfall.is_none() {
            self.initialize_waterfall(ctx);
        }

        // ── Keyboard shortcuts ──
        if is_key_pressed(KeyCode::Escape) {
            self.paused = true;
            ctx.resume_playback_time = Some(self.playback_time);
            return Some(NeothesiaEvent::MainMenu(Some(self.song.clone())));
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

        // ── Top bar click handling (kept for backward compatibility) ──
        if mouse_just_pressed && mouse_y <= Self::TOP_BAR_H {
            if let Some(event) = self.handle_top_bar_click(ctx, mouse_x, mouse_y) {
                self.paused = true;
                ctx.resume_playback_time = Some(self.playback_time);
                return Some(event);
            }
        }

        // ── Progress bar interaction (kept for backward compatibility) ──
        let progress_bar_bottom = Self::TOP_BAR_H + Self::PROGRESS_BAR_H;
        if mouse_y >= Self::TOP_BAR_H && mouse_y <= progress_bar_bottom {
            self.handle_progress_bar_click(mouse_x, mouse_y, mouse_down, mouse_just_pressed);
        } else if !mouse_down {
            self.is_seeking = false;
            self.is_dragging_looper_start = false;
            self.is_dragging_looper_end = false;
        }

        // ── Vertical timeline seek interaction ──
        if mouse_just_pressed || mouse_down {
            self.handle_vertical_timeline_click(mouse_x, mouse_y, mouse_down, mouse_just_pressed);
        }

        // ── Playback advancement ──
        if !self.paused && !self.is_seeking {
            let speed = ctx.config.speed_multiplier();
            let effective_dt = dt * speed;
            self.playback_time += effective_dt;

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
                        let message = MidiMessage::NoteOn {
                            key: u7::new(*note),
                            vel: u7::new(*velocity),
                        };
                        ctx.output_manager
                            .connection()
                            .midi_event(0u8.into(), message);
                    }
                }

                // Trigger NoteOff for notes that are no longer active
                for note in self.active_waterfall_notes.drain() {
                    if !current_notes.contains(&note) {
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
        let keyboard_y_start = screen_height() - 150.0;
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

        let (mx, my) = mouse_position();
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        if let Some(waterfall) = &mut self.waterfall {
            waterfall.render_ply();
        }
        self.piano_keyboard.render();

        // Render new HUD elements per design spec
        self.render_score_panel();
        self.render_timing_feedback(); // Top-right timing quality display
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
}

impl PlyScene for PlyFreeplayScene {
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent> {
        use midi_file::midly::MidiMessage;

        let dt = delta.as_secs_f32();

        // Comprehensive mouse input logging
        let screen_w = screen_width();
        let screen_h = screen_height();
        let mouse_pos = mouse_position();
        let mouse_is_down = is_mouse_button_down(MouseButton::Left);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_released = is_mouse_button_released(MouseButton::Left);

        // Log all mouse buttons
        let left_pressed = is_mouse_button_pressed(MouseButton::Left);
        let right_pressed = is_mouse_button_pressed(MouseButton::Right);
        let middle_pressed = is_mouse_button_pressed(MouseButton::Middle);

        log::debug!("[DEBUG] [PlyFreeplayScene::update] === MOUSE INPUT DUMP ===");
        log::debug!(
            "[DEBUG] [PlyFreeplayScene::update] Screen size: {:.0}x{:.0}",
            screen_w,
            screen_h
        );
        log::debug!(
            "[DEBUG] [PlyFreeplayScene::update] Mouse position: ({:.1}, {:.1})",
            mouse_pos.0,
            mouse_pos.1
        );
        log::debug!(
            "[DEBUG] [PlyFreeplayScene::update] Mouse buttons - Left: pressed={} down={} released={}, Right: pressed={}, Middle: pressed={}",
            left_pressed, mouse_is_down, mouse_is_released, right_pressed, middle_pressed
        );
        log::debug!(
            "[DEBUG] [PlyFreeplayScene::update] Previous state: mouse_was_pressed={}",
            self.mouse_was_pressed
        );

        // Check if mouse is over the piano keyboard area
        // Piano keyboard is typically at the bottom of the screen
        let keyboard_y_start = screen_h - 150.0; // Approximate keyboard area
        let is_over_keyboard = mouse_pos.1 >= keyboard_y_start;
        log::debug!(
            "[DEBUG] [PlyFreeplayScene::update] Mouse over keyboard area (y>={:.1}): {}",
            keyboard_y_start,
            is_over_keyboard
        );

        log::debug!("[DEBUG] [PlyFreeplayScene::update] Entry - dt={:.4}", dt);
        self.piano_keyboard.update(dt);

        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::MainMenu(None));
        }

        let top_bar_h = 30.0;
        let mouse_x = mouse_pos.0;
        let mouse_y = mouse_pos.1;

        if is_mouse_button_pressed(MouseButton::Left) && mouse_y <= top_bar_h {
            let btn_size = 30.0;
            let center_x = screen_w / 2.0;

            if mouse_x >= 0.0 && mouse_x <= btn_size {
                return Some(NeothesiaEvent::MainMenu(None));
            }

            let soundfont_name = self.current_soundfont_name();
            let text_w = measure_text(&soundfont_name, None, 14, 1.0).width;
            let prev_x = center_x - text_w / 2.0 - 40.0;
            if mouse_x >= prev_x && mouse_x <= prev_x + btn_size {
                self.previous_soundfont(ctx);
            }

            let next_x = center_x + text_w / 2.0 + 10.0;
            if mouse_x >= next_x && mouse_x <= next_x + btn_size {
                self.next_soundfont(ctx);
            }

            let dec_x = screen_w - 100.0;
            if mouse_x >= dec_x && mouse_x <= dec_x + btn_size {
                self.decrease_audio_gain(ctx);
            }

            let inc_x = screen_w - 65.0;
            if mouse_x >= inc_x && mouse_x <= inc_x + btn_size {
                self.increase_audio_gain(ctx);
            }

            return None;
        }

        // Only act on state CHANGES
        if mouse_is_down && !self.mouse_was_pressed {
            log::debug!("[DEBUG] [PlyFreeplayScene::update] Mouse just pressed - calling handle_mouse_input()");
            if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                Vec2::new(mouse_pos.0, mouse_pos.1),
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
        } else if !mouse_is_down && self.mouse_was_pressed {
            log::debug!("[DEBUG] [PlyFreeplayScene::update] Mouse just released - calling handle_mouse_input()");
            if let Some(notes) = self.piano_keyboard.handle_mouse_input(
                Vec2::new(mouse_pos.0, mouse_pos.1),
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
        } else if mouse_is_down && self.mouse_was_pressed {
            log::debug!(
                "[DEBUG] [PlyFreeplayScene::update] Mouse dragging - calling handle_mouse_drag()"
            );
            if let Some(notes) = self
                .piano_keyboard
                .handle_mouse_drag(Vec2::new(mouse_pos.0, mouse_pos.1))
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

        self.mouse_was_pressed = mouse_is_down;

        log::debug!("[DEBUG] [PlyFreeplayScene::update] Exit - update complete");
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
        clear_background(BLACK);

        let screen_w = screen_width();
        let screen_h = screen_height();
        let top_bar_h = 30.0;

        let dark_gray = Color::from_rgba(37, 35, 42, 255);
        let btn_color = Color::from_rgba(67, 67, 67, 255);
        let btn_hover = Color::from_rgba(87, 87, 87, 255);
        let btn_pressed = Color::from_rgba(97, 97, 97, 255);

        let (mouse_x, mouse_y) = mouse_position();
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        draw_rectangle(0.0, 0.0, screen_w, top_bar_h, dark_gray);

        let btn_size = 30.0;
        let btn_y = 0.0;

        let back_x = 0.0;
        let back_hover = mouse_x >= back_x
            && mouse_x <= back_x + btn_size
            && mouse_y >= btn_y
            && mouse_y <= btn_y + btn_size;
        let back_color = if back_hover && mouse_down {
            btn_pressed
        } else if back_hover {
            btn_hover
        } else {
            btn_color
        };
        draw_rectangle(back_x, btn_y, btn_size, btn_size, back_color);
        crate::scene::ply_fonts::draw_body("<-", back_x + 8.0, btn_y + 20.0, 16.0, WHITE);

        let soundfont_name = self.current_soundfont_name();
        let text_w = measure_text(
            &soundfont_name,
            crate::scene::ply_fonts::body_font(),
            14,
            1.0,
        )
        .width;
        let center_x = screen_w / 2.0;
        crate::scene::ply_fonts::draw_body(
            &soundfont_name,
            center_x - text_w / 2.0,
            20.0,
            14.0,
            WHITE,
        );

        let prev_x = center_x - text_w / 2.0 - 40.0;
        let prev_hover = mouse_x >= prev_x
            && mouse_x <= prev_x + btn_size
            && mouse_y >= btn_y
            && mouse_y <= btn_y + btn_size;
        let prev_color = if prev_hover && mouse_down {
            btn_pressed
        } else if prev_hover {
            btn_hover
        } else {
            btn_color
        };
        draw_rectangle(prev_x, btn_y, btn_size, btn_size, prev_color);
        crate::scene::ply_fonts::draw_body("<", prev_x + 10.0, btn_y + 20.0, 16.0, WHITE);

        let next_x = center_x + text_w / 2.0 + 10.0;
        let next_hover = mouse_x >= next_x
            && mouse_x <= next_x + btn_size
            && mouse_y >= btn_y
            && mouse_y <= btn_y + btn_size;
        let next_color = if next_hover && mouse_down {
            btn_pressed
        } else if next_hover {
            btn_hover
        } else {
            btn_color
        };
        draw_rectangle(next_x, btn_y, btn_size, btn_size, next_color);
        crate::scene::ply_fonts::draw_body(">", next_x + 10.0, btn_y + 20.0, 16.0, WHITE);

        let gain_text = format!("Gain: {:.1}", self.audio_gain);
        let gain_text_w =
            measure_text(&gain_text, crate::scene::ply_fonts::body_font(), 14, 1.0).width;
        let gain_text_x = screen_w - 180.0;
        crate::scene::ply_fonts::draw_body(&gain_text, gain_text_x, 20.0, 14.0, WHITE);

        let dec_x = screen_w - 100.0;
        let dec_hover = mouse_x >= dec_x
            && mouse_x <= dec_x + btn_size
            && mouse_y >= btn_y
            && mouse_y <= btn_y + btn_size;
        let dec_color = if dec_hover && mouse_down {
            btn_pressed
        } else if dec_hover {
            btn_hover
        } else {
            btn_color
        };
        draw_rectangle(dec_x, btn_y, btn_size, btn_size, dec_color);
        crate::scene::ply_fonts::draw_body("-", dec_x + 12.0, btn_y + 20.0, 16.0, WHITE);

        let inc_x = screen_w - 65.0;
        let inc_hover = mouse_x >= inc_x
            && mouse_x <= inc_x + btn_size
            && mouse_y >= btn_y
            && mouse_y <= btn_y + btn_size;
        let inc_color = if inc_hover && mouse_down {
            btn_pressed
        } else if inc_hover {
            btn_hover
        } else {
            btn_color
        };
        draw_rectangle(inc_x, btn_y, btn_size, btn_size, inc_color);
        crate::scene::ply_fonts::draw_body("+", inc_x + 10.0, btn_y + 20.0, 16.0, WHITE);

        self.piano_keyboard.render();
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
        let (mx, my) = mouse_position();
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
            return Some(NeothesiaEvent::Play(self.song.clone()));
        }

        // Mouse click handling
        if is_mouse_button_pressed(MouseButton::Left) {
            let screen_w = screen_width();
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
                return Some(NeothesiaEvent::Play(self.song.clone()));
            }

            // NEXT SONG button
            let next_y = btn_y + btn_h + btn_gap;
            if Self::is_mouse_in_rect(right_x, next_y, right_col_w, btn_h) {
                return Some(NeothesiaEvent::MainMenu(None));
            }

            // VIEW REPLAY button (same as PLAY AGAIN)
            let replay_y = next_y + btn_h + btn_gap;
            if Self::is_mouse_in_rect(right_x, replay_y, right_col_w, 40.0) {
                return Some(NeothesiaEvent::Play(self.song.clone()));
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

        let screen_w = screen_width();
        let screen_h = screen_height();

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

/// Setting type for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingType {
    Button,
    Toggle,
    Spinner,
    Slider,
    Picker,
}

/// Interactive setting for keyboard navigation
#[derive(Clone)]
struct InteractiveSetting {
    id: String,
    label: String,
    setting_type: SettingType,
    y_position: f32,
}

/// PLY Settings Scene - Interactive settings menu with all controls
pub struct PlySettingsScene {
    scroll_offset: f32,
    hovered_section: Option<String>,
    popup: SettingsPopup,
    popup_opened_this_frame: bool,
    soundfont_files: Vec<crate::output_manager::SoundFontEntry>,
    /// Current SoundFont index
    current_soundfont_index: Option<usize>,
    /// Song library directories
    song_directories: Vec<std::path::PathBuf>,
    /// SoundFont folders
    soundfont_folders: Vec<std::path::PathBuf>,
    /// Button areas for click detection
    button_areas: Vec<ButtonArea>,
    /// Toggle areas for click detection
    toggle_areas: Vec<ToggleArea>,
    /// Spin button areas for click detection
    spin_areas: Vec<SpinArea>,
    /// Slider areas for click detection and dragging
    slider_areas: Vec<SliderArea>,
    /// Stepper areas for click detection
    stepper_areas: Vec<StepperArea>,
    /// All interactive settings for keyboard navigation
    interactive_settings: Vec<InteractiveSetting>,
    /// Currently focused setting index
    focused_setting_index: Option<usize>,
    /// Keys that were pressed last frame (to prevent repeat)
    keys_pressed_last_frame: std::collections::HashSet<String>,
    /// Folder picker request state
    folder_picker_request: Option<FolderPickerRequest>,
    /// Currently dragged slider (if any)
    dragged_slider: Option<String>,
    /// Selected item index in the popup (for keyboard navigation)
    popup_selected_index: usize,
    /// Unified input manager for focus and priority management
    input_manager: UnifiedInputManager,
}

/// Folder picker request type
#[derive(Debug, Clone, PartialEq, Eq)]
enum FolderPickerRequest {
    SoundFontFolder,
    SongDirectory,
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

/// Slider area for click detection and dragging
#[derive(Clone)]
struct SliderArea {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
}

/// Stepper area for click detection
#[derive(Clone)]
struct StepperArea {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    options: Vec<String>,
    current_index: usize,
    is_cyclic: bool,
}

/// Settings popup state
#[derive(Debug, Clone, PartialEq)]
enum SettingsPopup {
    None,
    OutputSelector,
    InputSelector,
    ThemeSelector,
}

impl PlySettingsScene {
    pub fn new() -> Self {
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

        Self {
            scroll_offset: 0.0,
            hovered_section: None,
            popup: SettingsPopup::None,
            popup_opened_this_frame: false,
            soundfont_files: Vec::new(),
            current_soundfont_index: None,
            song_directories: Vec::new(),
            soundfont_folders: Vec::new(),
            button_areas: Vec::new(),
            toggle_areas: Vec::new(),
            spin_areas: Vec::new(),
            slider_areas: Vec::new(),
            stepper_areas: Vec::new(),
            interactive_settings: Vec::new(),
            focused_setting_index: None,
            keys_pressed_last_frame: std::collections::HashSet::new(),
            folder_picker_request: None,
            dragged_slider: None,
            popup_selected_index: 0,
            input_manager,
        }
    }

    /// Initialize settings with context data
    pub fn initialize(&mut self, ctx: &mut MacroquadContext) {
        // Load SoundFont folders from config
        self.soundfont_folders = ctx.config.synth_config.soundfont_folders().to_vec();

        // Discover SoundFonts
        self.soundfont_files = crate::output_manager::discover_soundfonts(&self.soundfont_folders);

        // Load current SoundFont index
        self.current_soundfont_index = ctx.config.synth_config.soundfont_index();

        // Load song directories
        self.song_directories = ctx.config.song_directories().to_vec();

        log::info!(
            "PLY Settings initialized with {} SoundFonts, {} song directories",
            self.soundfont_files.len(),
            self.song_directories.len()
        );
    }

    /// Register an interactive setting for keyboard navigation
    fn register_setting(
        &mut self,
        id: String,
        label: String,
        setting_type: SettingType,
        y_position: f32,
    ) {
        // Check if setting already exists
        if let Some(setting) = self.interactive_settings.iter_mut().find(|s| s.id == id) {
            // Update existing setting's y_position to reflect current rendered position
            // This is critical for mouse hover detection to work correctly when scrolling
            setting.y_position = y_position;

            // Also update the element position in the unified input manager
            self.input_manager
                .focus()
                .update_element_position(&id, (0.0, y_position));
        } else {
            // Add new setting
            let is_folder_picker = id == "add_soundfont_folder" || id == "add_song_directory";
            self.interactive_settings.push(InteractiveSetting {
                id: id.clone(),
                label: label.clone(),
                setting_type,
                y_position,
            });

            // Log folder picker button registration
            if is_folder_picker {
                log::info!(
                    "🔍 DEBUG: Registered folder picker button: '{}' at y={}",
                    label,
                    y_position
                );
            }

            // Auto-focus first setting ONLY if this is the very first setting being registered
            // (not just when focused_setting_index is None)
            if self.interactive_settings.len() == 1 {
                self.focused_setting_index = Some(0);
            }

            // Register the focusable element with the unified input manager
            // Convert SettingType to ElementType
            let element_type = match setting_type {
                SettingType::Button => ElementType::Button,
                SettingType::Toggle => ElementType::Toggle,
                SettingType::Spinner => ElementType::Spinner,
                SettingType::Slider => ElementType::Slider,
                SettingType::Picker => ElementType::Picker,
            };

            // Calculate screen position (centered horizontally)
            let screen_w = unsafe { macroquad::prelude::screen_width() };
            let margin_x = (screen_w - 650.0).max(0.0) / 2.0;

            self.input_manager
                .focus()
                .register_element(FocusableElement {
                    id: id.clone(),
                    label: label.clone(),
                    element_type,
                    position: (margin_x, y_position),
                    size: (650.0, 55.0),
                    focusable: true,
                });
        }
    }

    /// Get the currently focused setting
    fn focused_setting(&self) -> Option<&InteractiveSetting> {
        self.focused_setting_index
            .and_then(|idx| self.interactive_settings.get(idx))
    }

    /// Navigate to next setting
    fn focus_next(&mut self) {
        if self.interactive_settings.is_empty() {
            return;
        }

        let current = self.focused_setting_index.unwrap_or(0);
        self.focused_setting_index = Some((current + 1) % self.interactive_settings.len());
    }

    /// Navigate to previous setting
    fn focus_previous(&mut self) {
        if self.interactive_settings.is_empty() {
            return;
        }

        let current = self.focused_setting_index.unwrap_or(0);
        let count = self.interactive_settings.len();
        self.focused_setting_index = Some(if current == 0 { count - 1 } else { current - 1 });
    }

    /// Activate the currently focused setting
    fn activate_focused(&mut self, ctx: &mut MacroquadContext) -> Option<NeothesiaEvent> {
        if let Some(setting) = self.focused_setting() {
            let setting_id = setting.id.clone();
            let setting_type = setting.setting_type;
            match setting_type {
                SettingType::Button => {
                    return self.handle_button_click(ctx, &setting_id);
                }
                SettingType::Toggle => {
                    self.handle_toggle_click(ctx, &setting_id);
                }
                SettingType::Spinner => {
                    // For spinners, activation doesn't do anything special
                    // Use arrow keys to adjust values instead
                }
                SettingType::Slider => {
                    // For sliders, activation doesn't do anything special
                    // Use arrow keys to adjust values instead
                }
                SettingType::Picker => {
                    return self.handle_button_click(ctx, &setting_id);
                }
            }
        }
        None
    }

    /// Adjust value of focused setting (for spinners, sliders, and pickers)
    fn adjust_focused_value(&mut self, ctx: &mut MacroquadContext, delta: i32) {
        if let Some(setting) = self.focused_setting().cloned() {
            match setting.setting_type {
                SettingType::Spinner => {
                    // Use delta to determine direction: positive = increment, negative = decrement
                    let is_plus = delta > 0;
                    // Handle the adjustment with the correct direction
                    self.handle_spin_click(ctx, &setting.id, is_plus);
                }
                SettingType::Slider => {
                    // Handle slider keyboard navigation with step values
                    self.handle_slider_keyboard(ctx, &setting.id, delta);
                }
                SettingType::Picker => {
                    // Handle stepper navigation (like SoundFont selector)
                    let is_right = delta > 0;
                    self.handle_stepper_click(ctx, &setting.id, is_right);
                }
                _ => {}
            }
        }
    }

    /// Handle slider keyboard navigation
    fn handle_slider_keyboard(&mut self, ctx: &mut MacroquadContext, id: &str, delta: i32) {
        // Find the slider area to get its min, max, and step values
        let slider = match self.slider_areas.iter().find(|s| s.id == id) {
            Some(s) => s.clone(),
            None => return,
        };

        // Calculate new value
        let direction = if delta > 0 { 1.0 } else { -1.0 };
        let current_value = match id {
            "audio_gain_slider" => ctx.config.audio_gain(),
            "playback_gain_slider" => ctx.config.synth_config.playback_gain(),
            _ => slider.value,
        };

        let new_value = (current_value + direction * slider.step).clamp(slider.min, slider.max);

        // Update the appropriate config value
        match id {
            "audio_gain_slider" => {
                ctx.config.set_audio_gain(new_value);
                ctx.config.save();
            }
            "playback_gain_slider" => {
                ctx.config.synth_config.set_playback_gain(new_value);
                ctx.config.save();
            }
            _ => {}
        }
    }

    /// Check if a key was just pressed (not held down)
    fn is_key_just_pressed(&mut self, key: &str) -> bool {
        use macroquad::prelude::*;

        let is_pressed = match key {
            "Up" => is_key_pressed(KeyCode::Up),
            "Down" => is_key_pressed(KeyCode::Down),
            "Left" => is_key_pressed(KeyCode::Left),
            "Right" => is_key_pressed(KeyCode::Right),
            "Tab" => is_key_pressed(KeyCode::Tab),
            "Enter" => is_key_pressed(KeyCode::Enter),
            "Space" => is_key_pressed(KeyCode::Space),
            "Escape" => is_key_pressed(KeyCode::Escape),
            _ => false,
        };

        let was_not_pressed_last_frame = !self.keys_pressed_last_frame.contains(key);

        if is_pressed && was_not_pressed_last_frame {
            self.keys_pressed_last_frame.insert(key.to_string());
            true
        } else if !is_pressed {
            self.keys_pressed_last_frame.remove(key);
            false
        } else {
            false
        }
    }

    /// Clear all interactive areas at the start of each frame
    fn clear_areas(&mut self) {
        log::debug!(
            "🔍 DEBUG: Clearing button_areas (had {} entries)",
            self.button_areas.len()
        );
        self.button_areas.clear();
        self.toggle_areas.clear();
        self.spin_areas.clear();
        self.slider_areas.clear();
        self.stepper_areas.clear();
        // Don't clear interactive_settings - they persist across frames
        // But we do need to rebuild them each render frame to get correct positions
    }

    /// Check if a point is inside a rectangle
    fn is_inside(
        &self,
        x: f32,
        y: f32,
        rect_x: f32,
        rect_y: f32,
        rect_w: f32,
        rect_h: f32,
    ) -> bool {
        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }

    /// Draw a settings row with title, subtitle, and interactive control
    fn draw_settings_row(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        title: &str,
        subtitle: &str,
        is_hovered: bool,
        setting_id: Option<&str>,
        setting_type: SettingType,
    ) -> bool {
        use macroquad::prelude::*;

        // Register this setting for keyboard navigation
        let is_focused = if let Some(id) = setting_id {
            self.register_setting(id.to_string(), title.to_string(), setting_type, y);
            // Check if this setting is focused
            self.focused_setting()
                .map(|focused| focused.id == id)
                .unwrap_or(false)
        } else {
            false
        };

        // Draw background
        let bg_color = if is_focused {
            Color::from_rgba(100, 80, 140, 255) // Purple highlight for focused
        } else if is_hovered {
            Color::from_rgba(60, 55, 70, 255)
        } else {
            Color::from_rgba(45, 43, 50, 255)
        };

        draw_rectangle(x, y, width, height, bg_color);

        // Draw focus indicator on the left
        if is_focused {
            draw_rectangle(x, y, 4.0, height, Color::from_rgba(160, 81, 255, 255));
        }

        // Draw title with headline font
        let title_color = if is_focused {
            Color::from_rgba(200, 180, 255, 255)
        } else {
            Color::from_rgba(255, 255, 255, 255)
        };
        crate::scene::ply_fonts::draw_headline(title, x + 15.0, y + 12.0, 16.0, title_color);

        // Draw subtitle with body font
        crate::scene::ply_fonts::draw_body(
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

    /// Draw a button with custom font
    fn draw_button(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        label: &str,
        is_hovered: bool,
        is_focused: bool,
    ) -> bool {
        use macroquad::prelude::*;

        // Check focus state FIRST, then hover state (so focused buttons get focus color)
        let bg_color = if is_focused {
            Color::from_rgba(160, 81, 255, 255) // Purple for focused
        } else if is_hovered {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };

        draw_rectangle(x, y, width, height, bg_color);

        // Draw focus indicator if focused
        if is_focused {
            draw_rectangle_lines(
                x,
                y,
                width,
                height,
                2.0,
                Color::from_rgba(160, 81, 255, 255),
            );
        } else {
            draw_rectangle_lines(
                x,
                y,
                width,
                height,
                1.0,
                Color::from_rgba(100, 100, 100, 255),
            );
        }

        // Center text with headline font
        let text_width =
            measure_text(label, crate::scene::ply_fonts::headline_font(), 14, 1.0).width;
        crate::scene::ply_fonts::draw_headline(
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
    fn draw_toggle(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        value: bool,
        is_hovered: bool,
    ) -> bool {
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
    fn draw_spin_buttons(
        &self,
        x: f32,
        y: f32,
        size: f32,
        value: &str,
        is_hovered_plus: bool,
        is_hovered_minus: bool,
    ) -> (bool, bool) {
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
        let hover_minus =
            mouse_x >= minus_x && mouse_x <= minus_x + size && mouse_y >= y && mouse_y <= y + size;
        let hover_plus =
            mouse_x >= plus_x && mouse_x <= plus_x + size && mouse_y >= y && mouse_y <= y + size;

        (hover_minus, hover_plus)
    }

    /// Draw a horizontal slider control
    fn draw_slider(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        id: &str,
        is_focused: bool,
    ) -> (f32, bool) {
        use macroquad::prelude::*;

        let track_height = 6.0;
        let track_y = y + (height - track_height) / 2.0;
        let handle_size = 18.0;

        // Calculate fill ratio
        let range = max - min;
        let fill_ratio = if range > 0.0 {
            (value - min) / range
        } else {
            0.0
        };
        let fill_ratio = fill_ratio.clamp(0.0, 1.0);
        let fill_width = width * fill_ratio;
        // FIX: Clamp handle position to ensure it stays within slider bounds
        // The handle center should be within [x, x + width], so handle_x ranges from
        // x - handle_size/2 to x + width - handle_size/2
        let raw_handle_x = x + fill_width - handle_size / 2.0;
        let handle_x = raw_handle_x.clamp(x - handle_size / 2.0, x + width - handle_size / 2.0);
        let handle_y = track_y + (track_height - handle_size) / 2.0;

        // Check if mouse is over the slider
        let (mouse_x, mouse_y) = mouse_position();
        let is_hovered =
            mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height;

        // Check if this slider is being dragged
        let is_dragging = self
            .dragged_slider
            .as_ref()
            .map(|s| s == id)
            .unwrap_or(false);

        // Draw track background
        draw_rectangle(
            x,
            track_y,
            width,
            track_height,
            Color::from_rgba(74, 68, 88, 255),
        );

        // Draw filled portion
        if fill_width > 0.0 {
            let fill_color = if is_focused || is_dragging {
                Color::from_rgba(160, 81, 255, 255)
            } else if is_hovered {
                Color::from_rgba(120, 150, 255, 255)
            } else {
                Color::from_rgba(100, 180, 255, 255)
            };
            draw_rectangle(x, track_y, fill_width, track_height, fill_color);
        }

        // Draw handle
        let handle_color = if is_dragging {
            Color::from_rgba(220, 220, 255, 255)
        } else if is_focused {
            Color::from_rgba(200, 200, 255, 255)
        } else if is_hovered {
            Color::from_rgba(240, 240, 255, 255)
        } else {
            Color::from_rgba(255, 255, 255, 255)
        };
        draw_rectangle(handle_x, handle_y, handle_size, handle_size, handle_color);

        // Draw focus indicator
        if is_focused {
            draw_rectangle_lines(
                handle_x - 3.0,
                handle_y - 3.0,
                handle_size + 6.0,
                handle_size + 6.0,
                2.0,
                Color::from_rgba(160, 81, 255, 255),
            );
        }

        // Register slider area for click detection
        self.slider_areas.push(SliderArea {
            id: id.to_string(),
            x,
            y,
            width,
            height,
            min,
            max,
            step,
            value,
        });

        (value, is_hovered || is_dragging)
    }

    /// Draw a stepper control with left/right arrows
    fn draw_stepper(
        &mut self,
        x: f32,
        y: f32,
        button_size: f32,
        current_value: &str,
        options: &[String],
        current_index: usize,
        id: &str,
        is_cyclic: bool,
        is_focused: bool,
    ) -> (bool, bool) {
        use macroquad::prelude::*;

        let gap = 5.0;
        let left_x = x;
        let right_x = x + button_size + gap;
        let value_x = right_x + button_size + gap;

        // Check if mouse is over buttons
        let (mouse_x, mouse_y) = mouse_position();
        let left_hovered = mouse_x >= left_x
            && mouse_x <= left_x + button_size
            && mouse_y >= y
            && mouse_y <= y + button_size;
        let right_hovered = mouse_x >= right_x
            && mouse_x <= right_x + button_size
            && mouse_y >= y
            && mouse_y <= y + button_size;

        // Determine if left button should be disabled
        let left_disabled = !is_cyclic && current_index == 0;
        // Determine if right button should be disabled
        let right_disabled = !is_cyclic && current_index >= options.len().saturating_sub(1);

        // Draw left button
        let left_color = if left_disabled {
            Color::from_rgba(50, 50, 55, 255)
        } else if is_focused && left_hovered {
            Color::from_rgba(160, 81, 255, 255)
        } else if left_hovered {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        draw_rectangle(left_x, y, button_size, button_size, left_color);
        draw_text(
            "◀",
            left_x + button_size / 2.0 - 8.0,
            y + button_size / 2.0 + 5.0,
            14.0,
            if left_disabled {
                Color::from_rgba(100, 100, 100, 255)
            } else {
                Color::from_rgba(255, 255, 255, 255)
            },
        );

        // Draw right button
        let right_color = if right_disabled {
            Color::from_rgba(50, 50, 55, 255)
        } else if is_focused && right_hovered {
            Color::from_rgba(160, 81, 255, 255)
        } else if right_hovered {
            Color::from_rgba(100, 80, 120, 255)
        } else {
            Color::from_rgba(74, 68, 88, 255)
        };
        draw_rectangle(right_x, y, button_size, button_size, right_color);
        draw_text(
            "▶",
            right_x + button_size / 2.0 - 8.0,
            y + button_size / 2.0 + 5.0,
            14.0,
            if right_disabled {
                Color::from_rgba(100, 100, 100, 255)
            } else {
                Color::from_rgba(255, 255, 255, 255)
            },
        );

        // Draw current value
        draw_text(
            current_value,
            value_x,
            y + button_size / 2.0 + 5.0,
            14.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        // Draw focus indicator
        if is_focused {
            let total_width = button_size * 2.0 + gap + 100.0; // Approximate width
            draw_rectangle_lines(
                left_x - 2.0,
                y - 2.0,
                total_width + 4.0,
                button_size + 4.0,
                2.0,
                Color::from_rgba(160, 81, 255, 255),
            );
        }

        // Register stepper area for click detection
        self.stepper_areas.push(StepperArea {
            id: id.to_string(),
            x: left_x,
            y,
            width: button_size * 2.0 + gap,
            height: button_size,
            options: options.to_vec(),
            current_index,
            is_cyclic,
        });

        (
            left_hovered && !left_disabled,
            right_hovered && !right_disabled,
        )
    }

    fn draw_mini_keyboard_preview(
        &self,
        ctx: &MacroquadContext,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        use crate::render::ply::piano_keyboard::KeyboardTheme;
        use macroquad::prelude::*;

        let theme_name = ctx.config.piano_theme_name();
        let theme = KeyboardTheme::get_theme(theme_name).unwrap_or_else(|| KeyboardTheme::modern());

        draw_rectangle(x, y, width, height, Color::from_rgba(30, 28, 35, 255));

        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let is_sharp = [
            false, true, false, true, false, false, true, false, true, false, true, false,
        ];

        let num_white_keys = 14;
        let white_key_width = width / num_white_keys as f32;
        let black_key_width = white_key_width * 0.6;
        let black_key_height = height * 0.65;

        let white_key_positions: Vec<f32> = (0..num_white_keys)
            .map(|i| x + i as f32 * white_key_width)
            .collect();

        let black_key_offsets = [0.7, 1.7, 3.7, 4.7, 5.7, 7.7, 8.7, 10.7, 11.7, 12.7];
        let black_key_indices = [1, 3, 6, 8, 10, 13, 15, 18, 20, 22];

        let white_note_map = [0, 2, 4, 5, 7, 9, 11];

        for wi in 0..num_white_keys {
            let note_idx = white_note_map[wi % 7];
            let kx = white_key_positions[wi];
            let note_color = theme.octave_theme.note_color(note_idx);
            let (r, g, b) = note_color.normal;

            draw_rectangle(
                kx + 0.5,
                y + 2.0,
                white_key_width - 1.0,
                height - 4.0,
                Color::from_rgba(r, g, b, 255),
            );

            let label = note_names[note_idx];
            draw_text(
                label,
                kx + white_key_width / 2.0 - 4.0,
                y + height - 6.0,
                10.0,
                Color::from_rgba(100, 100, 100, 255),
            );
        }

        for (idx, &black_idx) in black_key_indices.iter().enumerate() {
            if idx < black_key_offsets.len() {
                let kx = x + black_key_offsets[idx] * white_key_width - black_key_width / 2.0;
                let note_color = theme.octave_theme.note_color(black_idx % 12);
                let (r, g, b) = note_color.normal;

                draw_rectangle(
                    kx,
                    y + 2.0,
                    black_key_width,
                    black_key_height,
                    Color::from_rgba(r, g, b, 255),
                );
            }
        }
    }

    /// Handle button click
    fn handle_button_click(
        &mut self,
        ctx: &mut MacroquadContext,
        id: &str,
    ) -> Option<NeothesiaEvent> {
        log::info!("🔍 DEBUG: handle_button_click called with id: '{}'", id);
        match id {
            "back" => {
                log::info!("🔍 DEBUG: Back button clicked");
                return Some(NeothesiaEvent::MainMenu(None));
            }
            "output" => {
                log::info!("🔍 DEBUG: Output button clicked");
                self.popup = SettingsPopup::OutputSelector;
                // Initialize selection to current output
                let selected_output = ctx.config.output().as_deref().unwrap_or("None").to_string();
                let outputs: Vec<String> = ctx
                    .output_manager
                    .outputs()
                    .iter()
                    .map(|o| o.to_string())
                    .collect();
                self.popup_selected_index = outputs
                    .iter()
                    .position(|o| o == &selected_output)
                    .unwrap_or(0);
            }
            "input" => {
                log::info!("🔍 DEBUG: Input button clicked");
                self.popup = SettingsPopup::InputSelector;
                // Initialize selection to current input
                let selected_input = ctx.config.input().as_deref().unwrap_or("None").to_string();

                // Build dynamic input options list
                let mut inputs = vec!["None".to_string(), "Keyboard".to_string()];

                // Get available MIDI input devices
                if let Ok(midi_input_manager) = midi_io::MidiInputManager::new() {
                    let midi_inputs = midi_input_manager.inputs();
                    for midi_input in midi_inputs {
                        inputs.push(midi_input.to_string());
                    }
                }

                self.popup_selected_index = inputs
                    .iter()
                    .position(|i| i == &selected_input)
                    .unwrap_or(0);
            }
            "piano_theme" => {
                self.popup = SettingsPopup::ThemeSelector;
                self.popup_opened_this_frame = true;
                let themes = [
                    "Classic",
                    "Modern",
                    "Classic Colors",
                    "Rainbow",
                    "Neon",
                    "Pastel",
                ];
                let current_theme = ctx.config.piano_theme_name();
                self.popup_selected_index =
                    themes.iter().position(|t| *t == current_theme).unwrap_or(0);
            }
            "add_soundfont_folder" => {
                log::info!(
                    "🔍 DEBUG: Add SoundFont folder button clicked - triggering folder picker"
                );
                self.folder_picker_request = Some(FolderPickerRequest::SoundFontFolder);
                // Try to pick folder immediately
                self.pick_soundfont_folder(ctx);
            }
            "add_song_directory" => {
                log::info!(
                    "🔍 DEBUG: Add song directory button clicked - triggering folder picker"
                );
                self.folder_picker_request = Some(FolderPickerRequest::SongDirectory);
                // Try to pick folder immediately
                self.pick_song_directory(ctx);
            }
            _ => {
                log::warn!("🔍 DEBUG: Unknown button id: '{}'", id);
            }
        }
        None
    }

    /// Handle toggle click
    fn handle_toggle_click(&mut self, ctx: &mut MacroquadContext, id: &str) {
        match id {
            "vertical_guidelines" => {
                ctx.config
                    .set_vertical_guidelines(!ctx.config.vertical_guidelines());
                ctx.config.save();
            }
            "horizontal_guidelines" => {
                ctx.config
                    .set_horizontal_guidelines(!ctx.config.horizontal_guidelines());
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
                    ctx.config
                        .set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
                }
                ctx.config.save();
            }
            "range_end" => {
                if is_plus {
                    ctx.config
                        .set_piano_range_end(ctx.config.piano_range().end() + 1);
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

    /// Handle slider drag interaction
    fn handle_slider_drag(&mut self, ctx: &mut MacroquadContext, id: &str, mouse_x: f32) {
        // Find the slider area
        let slider = match self.slider_areas.iter().find(|s| s.id == id) {
            Some(s) => s.clone(),
            None => return,
        };

        // Calculate value from mouse position
        let relative_x = (mouse_x - slider.x).clamp(0.0, slider.width);
        let ratio = relative_x / slider.width;
        let raw_value = slider.min + ratio * (slider.max - slider.min);

        // Snap to step
        let stepped_value =
            ((raw_value - slider.min) / slider.step).round() * slider.step + slider.min;
        let new_value = stepped_value.clamp(slider.min, slider.max);

        // Update the appropriate config value
        match id {
            "audio_gain_slider" => {
                ctx.config.set_audio_gain(new_value);
                ctx.config.save();
            }
            "playback_gain_slider" => {
                ctx.config.synth_config.set_playback_gain(new_value);
                ctx.config.save();
            }
            "keyboard_gain_slider" => {
                // Note: keyboard_gain is in V2 but doesn't have a setter in the current API
                // We'll skip this for now
            }
            "speed_multiplier_slider" => {
                // Note: speed_multiplier is in playback config
                // We'll need to add this to the config API
            }
            "lumi_brightness_slider" => {
                // Note: lumi_brightness is in playback config
                // We'll need to add this to the config API
            }
            _ => {}
        }
    }

    /// Handle stepper button click
    fn handle_stepper_click(&mut self, ctx: &mut MacroquadContext, id: &str, is_right: bool) {
        match id {
            "soundfont" => {
                // Handle SoundFont cycling
                if self.soundfont_files.is_empty() {
                    return;
                }

                let current_index = self.current_soundfont_index.unwrap_or(0);
                let count = self.soundfont_files.len();

                let new_index = if is_right {
                    // Move to next SoundFont (wrap around)
                    (current_index + 1) % count
                } else {
                    // Move to previous SoundFont (wrap around)
                    if current_index == 0 {
                        count - 1
                    } else {
                        current_index - 1
                    }
                };

                self.current_soundfont_index = Some(new_index);

                // Update config with new SoundFont
                if let Some(entry) = self.soundfont_files.get(new_index) {
                    let soundfont_name = entry
                        .path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown");

                    ctx.config
                        .synth_config
                        .set_soundfont_path(Some(entry.path.clone()));
                    ctx.config.synth_config.set_soundfont_index(Some(new_index));
                    ctx.config.save();

                    log::info!(
                        "SoundFont changed to: {} (index {})",
                        soundfont_name,
                        new_index
                    );
                }
            }
            "lumi_color_mode_stepper" => {
                let current = ctx.config.lumi_color_mode();
                let max_mode = 7; // Assuming modes 0-7

                let new_mode = if is_right {
                    if current + 1 > max_mode {
                        0 // Wrap around
                    } else {
                        current + 1
                    }
                } else {
                    if current == 0 {
                        max_mode // Wrap around
                    } else {
                        current - 1
                    }
                };

                ctx.config.set_lumi_color_mode(new_mode);
                ctx.config.save();
            }
            // Note: sort_preference_stepper is commented out due to type mismatch issues
            // between song_library::SortPreference and config::SortPreference
            // This can be added later once the type system is unified
            _ => {}
        }
    }

    /// Pick SoundFont folder using native file dialog
    fn pick_soundfont_folder(&mut self, ctx: &mut MacroquadContext) {
        use std::path::PathBuf;

        log::info!("Opening SoundFont folder picker");

        // Use rfd's blocking API for folder picker
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            log::info!("Selected SoundFont folder: {:?}", path);

            // Check if folder already exists
            if !self.soundfont_folders.contains(&path) {
                self.soundfont_folders.push(path.clone());

                // Re-discover SoundFonts
                self.soundfont_files =
                    crate::output_manager::discover_soundfonts(&self.soundfont_folders);

                // Select first SoundFont if available and none selected
                if self.current_soundfont_index.is_none() && !self.soundfont_files.is_empty() {
                    self.current_soundfont_index = Some(0);
                    let first_entry = &self.soundfont_files[0];
                    ctx.config
                        .synth_config
                        .set_soundfont_path(Some(first_entry.path.clone()));
                    ctx.config.synth_config.set_soundfont_index(Some(0));
                }

                // Save updated folders list
                ctx.config
                    .synth_config
                    .set_soundfont_folders(self.soundfont_folders.clone());
                ctx.config.save();

                log::info!(
                    "Successfully added SoundFont folder, now have {} folders",
                    self.soundfont_folders.len()
                );
            } else {
                log::warn!("Folder already exists in SoundFont folders");
            }
        } else {
            log::info!("User cancelled SoundFont folder picker");
        }

        // Clear the request
        self.folder_picker_request = None;
    }

    /// Pick song directory using native file dialog
    fn pick_song_directory(&mut self, ctx: &mut MacroquadContext) {
        use std::path::PathBuf;

        log::info!("Opening song directory picker");

        // Use rfd's blocking API for folder picker
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            log::info!("Selected song directory: {:?}", path);

            // Add to config
            ctx.config.add_song_directory(path.clone());
            ctx.config.save();

            // Update local list
            self.song_directories = ctx.config.song_directories().to_vec();

            log::info!(
                "Successfully added song directory, now have {} directories",
                self.song_directories.len()
            );
        } else {
            log::info!("User cancelled song directory picker");
        }

        // Clear the request
        self.folder_picker_request = None;
    }

    /// Handle keyboard navigation within popup
    fn handle_popup_keyboard(&mut self, ctx: &mut MacroquadContext) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        match &self.popup {
            SettingsPopup::None => None,
            SettingsPopup::OutputSelector => {
                // Get available outputs
                let outputs: Vec<String> = ctx
                    .output_manager
                    .outputs()
                    .iter()
                    .map(|o| o.to_string())
                    .collect();
                let selected_output = ctx.config.output().as_deref().unwrap_or("None").to_string();

                // Find current selected index
                let current_index = outputs
                    .iter()
                    .position(|o| o == &selected_output)
                    .unwrap_or(0);

                // Handle arrow key navigation
                if self.is_key_just_pressed("Up") {
                    // Set keyboard priority when navigation keys are pressed
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index > 0 {
                        self.popup_selected_index -= 1;
                    }
                }
                if self.is_key_just_pressed("Down") {
                    // Set keyboard priority when navigation keys are pressed
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index < outputs.len().saturating_sub(1) {
                        self.popup_selected_index += 1;
                    }
                }

                // Handle Enter to select
                if self.is_key_just_pressed("Enter") {
                    if let Some(output) = outputs.get(self.popup_selected_index) {
                        ctx.config.set_output(Some(output.clone()));
                        ctx.config.save();
                        self.popup = SettingsPopup::None;
                        log::info!("Output changed to: {}", output);
                    }
                }

                None
            }
            SettingsPopup::InputSelector => {
                // Build dynamic input options list
                let mut inputs = vec!["None".to_string(), "Keyboard".to_string()];

                // Get available MIDI input devices
                if let Ok(midi_input_manager) = midi_io::MidiInputManager::new() {
                    let midi_inputs = midi_input_manager.inputs();
                    for midi_input in midi_inputs {
                        inputs.push(midi_input.to_string());
                    }
                }

                let selected_input = ctx.config.input().as_deref().unwrap_or("None").to_string();

                // Find current selected index
                let current_index = inputs
                    .iter()
                    .position(|i| i == &selected_input)
                    .unwrap_or(0);

                // Handle arrow key navigation
                if self.is_key_just_pressed("Up") {
                    // Set keyboard priority when navigation keys are pressed
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index > 0 {
                        self.popup_selected_index -= 1;
                    }
                }
                if self.is_key_just_pressed("Down") {
                    // Set keyboard priority when navigation keys are pressed
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index < inputs.len().saturating_sub(1) {
                        self.popup_selected_index += 1;
                    }
                }

                // Handle Enter to select
                if self.is_key_just_pressed("Enter") {
                    if let Some(input) = inputs.get(self.popup_selected_index) {
                        ctx.config.set_input(Some(input.clone()));
                        ctx.config.save();
                        self.popup = SettingsPopup::None;
                        log::info!("Input changed to: {}", input);
                    }
                }

                None
            }
            SettingsPopup::ThemeSelector => {
                let themes = [
                    "Classic",
                    "Modern",
                    "Classic Colors",
                    "Rainbow",
                    "Neon",
                    "Pastel",
                ];

                if self.is_key_just_pressed("Up") {
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index > 0 {
                        self.popup_selected_index -= 1;
                    }
                }
                if self.is_key_just_pressed("Down") {
                    self.input_manager
                        .focus()
                        .priority()
                        .set_keyboard_priority();
                    if self.popup_selected_index < themes.len().saturating_sub(1) {
                        self.popup_selected_index += 1;
                    }
                }

                if self.is_key_just_pressed("Enter") {
                    if let Some(theme_name) = themes.get(self.popup_selected_index) {
                        ctx.config.set_piano_theme_name(theme_name.to_string());
                        ctx.config.save();
                        self.popup = SettingsPopup::None;
                    }
                }

                None
            }
        }
    }

    /// Draw popup overlay for device selection
    fn draw_popup(&mut self, ctx: &mut MacroquadContext) {
        match self.popup {
            SettingsPopup::None => {}
            SettingsPopup::OutputSelector => {
                self.draw_output_selector(ctx);
            }
            SettingsPopup::InputSelector => {
                self.draw_input_selector(ctx);
            }
            SettingsPopup::ThemeSelector => {
                self.draw_theme_selector(ctx);
            }
        }
    }

    /// Draw output selector popup
    fn draw_output_selector(&mut self, ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        let screen_w = screen_width();
        let screen_h = screen_height();

        let popup_w = 320.0;
        let popup_h = 300.0;
        let popup_x = (screen_w - popup_w) / 2.0;
        let popup_y = (screen_h - popup_h) / 2.0;

        // Draw overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::from_rgba(0, 0, 0, 200));

        // Draw popup background
        draw_rectangle(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            Color::from_rgba(45, 43, 50, 255),
        );
        draw_rectangle_lines(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            2.0,
            Color::from_rgba(160, 81, 255, 255),
        );

        // Draw title
        draw_text(
            "Select Output",
            popup_x + 10.0,
            popup_y + 10.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Get available outputs
        let outputs: Vec<String> = ctx
            .output_manager
            .outputs()
            .iter()
            .map(|o| o.to_string())
            .collect();
        let selected_output = ctx.config.output().as_deref().unwrap_or("None").to_string();

        // Draw output options
        let mut y = popup_y + 50.0;
        for (idx, output) in outputs.iter().enumerate() {
            let is_selected = selected_output == *output;
            let is_focused = idx == self.popup_selected_index;

            // Draw option background
            if is_selected {
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            } else if is_focused {
                // Draw focus indicator for keyboard navigation
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    Color::from_rgba(100, 80, 140, 255),
                );
                draw_rectangle_lines(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    2.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            }

            // Draw option text
            draw_text(
                output,
                popup_x + 20.0,
                y + 10.0,
                16.0,
                if is_selected {
                    Color::from_rgba(255, 255, 255, 255)
                } else if is_focused {
                    Color::from_rgba(220, 220, 255, 255)
                } else {
                    Color::from_rgba(200, 200, 200, 255)
                },
            );

            // Make clickable
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= popup_x + 10.0
                && mouse_x <= popup_x + popup_w - 10.0
                && mouse_y >= y
                && mouse_y <= y + 40.0
            {
                if self.popup_selected_index != idx {
                    self.popup_selected_index = idx;
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    ctx.config.set_output(Some(output.clone()));
                    ctx.config.save();
                    self.popup = SettingsPopup::None;
                    log::info!("Output changed to: {}", output);
                    return;
                }
            }

            y += 45.0;
        }

        // Close button
        let close_x = popup_x + popup_w - 40.0;
        let close_y = popup_y + 10.0;
        let (mouse_x, mouse_y) = mouse_position();
        let close_hovered = mouse_x >= close_x
            && mouse_x <= close_x + 30.0
            && mouse_y >= close_y
            && mouse_y <= close_y + 30.0;

        draw_rectangle(
            close_x,
            close_y,
            30.0,
            30.0,
            if close_hovered {
                Color::from_rgba(120, 120, 120, 255)
            } else {
                Color::from_rgba(80, 80, 80, 255)
            },
        );
        draw_text(
            "✕",
            close_x + 8.0,
            close_y + 5.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        if close_hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.popup = SettingsPopup::None;
        }

        // Draw keyboard navigation hint
        draw_text(
            "↑↓: Navigate • Enter: Select • ESC: Close",
            popup_x + 10.0,
            popup_y + popup_h - 25.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }

    /// Draw input selector popup
    fn draw_input_selector(&mut self, ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        let screen_w = screen_width();
        let screen_h = screen_height();

        let popup_w = 320.0;
        let popup_h = 300.0;
        let popup_x = (screen_w - popup_w) / 2.0;
        let popup_y = (screen_h - popup_h) / 2.0;

        // Draw overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::from_rgba(0, 0, 0, 200));

        // Draw popup background
        draw_rectangle(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            Color::from_rgba(45, 43, 50, 255),
        );
        draw_rectangle_lines(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            2.0,
            Color::from_rgba(160, 81, 255, 255),
        );

        // Draw title
        draw_text(
            "Select Input",
            popup_x + 10.0,
            popup_y + 10.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Build dynamic input options list
        let mut inputs = vec!["None".to_string(), "Keyboard".to_string()];

        // Get available MIDI input devices
        if let Ok(midi_input_manager) = midi_io::MidiInputManager::new() {
            let midi_inputs = midi_input_manager.inputs();
            for midi_input in midi_inputs {
                inputs.push(midi_input.to_string());
            }
        }

        let selected_input = ctx.config.input().as_deref().unwrap_or("None").to_string();

        // Draw input options
        let mut y = popup_y + 50.0;
        for (idx, input) in inputs.iter().enumerate() {
            let is_selected = selected_input == *input;
            let is_focused = idx == self.popup_selected_index;

            // Draw option background
            if is_selected {
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            } else if is_focused {
                // Draw focus indicator for keyboard navigation
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    Color::from_rgba(100, 80, 140, 255),
                );
                draw_rectangle_lines(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    40.0,
                    2.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            }

            // Draw option text
            draw_text(
                input,
                popup_x + 20.0,
                y + 10.0,
                16.0,
                if is_selected {
                    Color::from_rgba(255, 255, 255, 255)
                } else if is_focused {
                    Color::from_rgba(220, 220, 255, 255)
                } else {
                    Color::from_rgba(200, 200, 200, 255)
                },
            );

            // Make clickable
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= popup_x + 10.0
                && mouse_x <= popup_x + popup_w - 10.0
                && mouse_y >= y
                && mouse_y <= y + 40.0
            {
                if self.popup_selected_index != idx {
                    self.popup_selected_index = idx;
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    ctx.config.set_input(Some(input.clone()));
                    ctx.config.save();
                    self.popup = SettingsPopup::None;
                    log::info!("Input changed to: {}", input);
                    return;
                }
            }

            y += 45.0;
        }

        // Close button
        let close_x = popup_x + popup_w - 40.0;
        let close_y = popup_y + 10.0;
        let (mouse_x, mouse_y) = mouse_position();
        let close_hovered = mouse_x >= close_x
            && mouse_x <= close_x + 30.0
            && mouse_y >= close_y
            && mouse_y <= close_y + 30.0;

        draw_rectangle(
            close_x,
            close_y,
            30.0,
            30.0,
            if close_hovered {
                Color::from_rgba(120, 120, 120, 255)
            } else {
                Color::from_rgba(80, 80, 80, 255)
            },
        );
        draw_text(
            "✕",
            close_x + 8.0,
            close_y + 5.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        if close_hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.popup = SettingsPopup::None;
        }

        // Draw keyboard navigation hint
        draw_text(
            "↑↓: Navigate • Enter: Select • ESC: Close",
            popup_x + 10.0,
            popup_y + popup_h - 25.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }

    fn draw_theme_selector(&mut self, ctx: &mut MacroquadContext) {
        use crate::render::ply::piano_keyboard::KeyboardTheme;
        use macroquad::prelude::*;

        let skip_clicks = self.popup_opened_this_frame;
        self.popup_opened_this_frame = false;

        let screen_w = screen_width();
        let screen_h = screen_height();

        let popup_w = 400.0;
        let popup_h = 420.0;
        let popup_x = (screen_w - popup_w) / 2.0;
        let popup_y = (screen_h - popup_h) / 2.0;

        let themes = [
            "Classic",
            "Modern",
            "Classic Colors",
            "Rainbow",
            "Neon",
            "Pastel",
        ];
        let descriptions = [
            "Traditional black and white piano",
            "Clean design with green highlights",
            "Classic look with subtle color on pressed keys",
            "Each note has a unique spectral color",
            "Dark background with bright glowing colors",
            "Soft, muted colors",
        ];

        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::from_rgba(0, 0, 0, 200));

        draw_rectangle(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            Color::from_rgba(45, 43, 50, 255),
        );
        draw_rectangle_lines(
            popup_x,
            popup_y,
            popup_w,
            popup_h,
            2.0,
            Color::from_rgba(160, 81, 255, 255),
        );

        draw_text(
            "Select Piano Theme",
            popup_x + 10.0,
            popup_y + 10.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        let current_theme = ctx.config.piano_theme_name();
        let mut y = popup_y + 45.0;

        for (idx, theme_name) in themes.iter().enumerate() {
            let is_selected = current_theme == *theme_name;
            let is_focused = idx == self.popup_selected_index;
            let theme =
                KeyboardTheme::get_theme(theme_name).unwrap_or_else(|| KeyboardTheme::modern());

            if is_selected {
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    65.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            } else if is_focused {
                draw_rectangle(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    65.0,
                    Color::from_rgba(100, 80, 140, 255),
                );
                draw_rectangle_lines(
                    popup_x + 10.0,
                    y,
                    popup_w - 20.0,
                    65.0,
                    2.0,
                    Color::from_rgba(160, 81, 255, 255),
                );
            }

            draw_text(
                theme_name,
                popup_x + 20.0,
                y + 5.0,
                14.0,
                if is_selected || is_focused {
                    Color::from_rgba(255, 255, 255, 255)
                } else {
                    Color::from_rgba(200, 200, 200, 255)
                },
            );

            let note_names = [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
            ];
            let white_key_map = [0, 2, 4, 5, 7, 9, 11];
            let preview_w = popup_w - 40.0;
            let preview_h = 25.0;
            let preview_x = popup_x + 20.0;
            let preview_y = y + 20.0;
            let white_key_w = preview_w / 7.0;

            for wi in 0..7 {
                let note_idx = white_key_map[wi];
                let kx = preview_x + wi as f32 * white_key_w;
                let note_color = theme.octave_theme.note_color(note_idx);
                let (r, g, b) = note_color.normal;
                draw_rectangle(
                    kx + 0.5,
                    preview_y,
                    white_key_w - 1.0,
                    preview_h,
                    Color::from_rgba(r, g, b, 255),
                );
                draw_text(
                    note_names[note_idx],
                    kx + white_key_w / 2.0 - 4.0,
                    preview_y + preview_h - 4.0,
                    8.0,
                    Color::from_rgba(100, 100, 100, 255),
                );
            }

            let black_key_offsets = [0.65, 1.65, 3.65, 4.65, 5.65];
            let black_key_indices = [1, 3, 6, 8, 10];
            let black_key_w = white_key_w * 0.5;
            let black_key_h = preview_h * 0.65;

            for (bi, &note_idx) in black_key_indices.iter().enumerate() {
                let kx = preview_x + black_key_offsets[bi] * white_key_w - black_key_w / 2.0;
                let note_color = theme.octave_theme.note_color(note_idx);
                let (r, g, b) = note_color.normal;
                draw_rectangle(
                    kx,
                    preview_y,
                    black_key_w,
                    black_key_h,
                    Color::from_rgba(r, g, b, 255),
                );
            }

            draw_text(
                descriptions[idx],
                popup_x + 20.0,
                y + 50.0,
                10.0,
                if is_selected {
                    Color::from_rgba(230, 230, 230, 255)
                } else {
                    Color::from_rgba(150, 150, 150, 255)
                },
            );

            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= popup_x + 10.0
                && mouse_x <= popup_x + popup_w - 10.0
                && mouse_y >= y
                && mouse_y <= y + 65.0
            {
                if self.input_manager.has_mouse_priority()
                    || self.input_manager.get_priority() == InputPriority::None
                {
                    self.input_manager
                        .focus()
                        .priority()
                        .update_mouse_position(mouse_x, mouse_y);
                    if self.popup_selected_index != idx {
                        self.popup_selected_index = idx;
                    }
                }

                if !skip_clicks && is_mouse_button_pressed(MouseButton::Left) {
                    ctx.config.set_piano_theme_name(theme_name.to_string());
                    ctx.config.save();
                    self.popup = SettingsPopup::None;
                    return;
                }
            }

            y += 70.0;
        }

        let close_x = popup_x + popup_w - 40.0;
        let close_y = popup_y + 10.0;
        let (mouse_x, mouse_y) = mouse_position();
        let close_hovered = mouse_x >= close_x
            && mouse_x <= close_x + 30.0
            && mouse_y >= close_y
            && mouse_y <= close_y + 30.0;

        draw_rectangle(
            close_x,
            close_y,
            30.0,
            30.0,
            if close_hovered {
                Color::from_rgba(120, 120, 120, 255)
            } else {
                Color::from_rgba(80, 80, 80, 255)
            },
        );
        draw_text(
            "✕",
            close_x + 8.0,
            close_y + 5.0,
            18.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        if !skip_clicks && close_hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.popup = SettingsPopup::None;
        }

        draw_text(
            "↑↓: Navigate • Enter: Select • ESC: Close",
            popup_x + 10.0,
            popup_y + popup_h - 25.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}

impl PlyScene for PlySettingsScene {
    fn update(&mut self, ctx: &mut MacroquadContext, delta: Duration) -> Option<NeothesiaEvent> {
        use macroquad::prelude::*;

        // NOTE: Don't clear button_areas here - they are populated by render() and need to persist
        // to the next frame's update() for mouse click detection to work.
        // button_areas are cleared at the START of render() instead.
        // NOTE: Don't clear interactive_settings - they persist across frames to maintain focus

        // Update the unified input manager
        self.input_manager.update(delta.as_secs_f64());

        log::debug!(
            "🔍 DEBUG: Update() - button_areas has {} entries from previous render",
            self.button_areas.len()
        );

        // Handle keyboard navigation
        if self.is_key_just_pressed("Escape") {
            if self.popup != SettingsPopup::None {
                self.popup = SettingsPopup::None;
            } else {
                return Some(NeothesiaEvent::ResumeFromSettings);
            }
        }

        // Handle popup keyboard navigation
        if self.popup != SettingsPopup::None {
            return self.handle_popup_keyboard(ctx);
        }

        // Tab navigation between settings
        if self.is_key_just_pressed("Tab") {
            // Check if shift is held (for reverse tab)
            let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            if shift_held {
                log::debug!("🔍 DEBUG: Shift+Tab pressed - focusing previous");
                self.focus_previous();
            } else {
                log::debug!("🔍 DEBUG: Tab pressed - focusing next");
                self.focus_next();
            }
            if let Some(focused) = self.focused_setting() {
                log::info!(
                    "🔍 DEBUG: Now focused on: {} ({})",
                    focused.label,
                    focused.id
                );
            }
        }

        // Arrow key navigation
        if self.is_key_just_pressed("Down") {
            self.focus_next();
        }
        if self.is_key_just_pressed("Up") {
            self.focus_previous();
        }

        // Activate focused setting with Enter or Space
        if self.is_key_just_pressed("Enter") || self.is_key_just_pressed("Space") {
            log::debug!("🔍 DEBUG: Enter/Space pressed - activating focused setting");
            if let Some(focused) = self.focused_setting() {
                log::info!("🔍 DEBUG: Activating: {} ({})", focused.label, focused.id);
            }
            if let Some(event) = self.activate_focused(ctx) {
                return Some(event);
            }
        }

        // Adjust spinner and stepper values with left/right arrows
        if self.is_key_just_pressed("Right") {
            self.adjust_focused_value(ctx, 1);
        }
        if self.is_key_just_pressed("Left") {
            self.adjust_focused_value(ctx, -1);
        }

        // Handle scroll
        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            self.scroll_offset = (self.scroll_offset - mouse_wheel.1 * 20.0).max(0.0);
        }

        // Update mouse position in the unified input manager
        let (mouse_x, mouse_y) = mouse_position();
        self.input_manager
            .focus()
            .priority()
            .update_mouse_position(mouse_x, mouse_y);

        // Handle mouse clicks
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            log::debug!(
                "🔍 DEBUG: Mouse click at ({}, {}) - checking {} button areas",
                mouse_x,
                mouse_y,
                self.button_areas.len()
            );

            // Collect button clicks first to avoid borrow checker issues
            let mut button_click = None;
            for button in &self.button_areas {
                if self.is_inside(
                    mouse_x,
                    mouse_y,
                    button.x,
                    button.y,
                    button.width,
                    button.height,
                ) {
                    button_click = Some(button.id.clone());
                    log::info!(
                        "🔍 DEBUG: Button clicked: {} at ({}, {})",
                        button.id,
                        button.x,
                        button.y
                    );
                    break;
                }
            }
            if let Some(id) = button_click {
                return self.handle_button_click(ctx, &id);
            } else {
                log::debug!("🔍 DEBUG: No button matched the click position");
            }

            // Collect toggle clicks first to avoid borrow checker issues
            let mut toggle_click = None;
            for toggle in &self.toggle_areas {
                if self.is_inside(
                    mouse_x,
                    mouse_y,
                    toggle.x,
                    toggle.y,
                    toggle.width,
                    toggle.height,
                ) {
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

            // Handle slider clicks - start dragging
            let mut slider_click = None;
            for slider in &self.slider_areas {
                if self.is_inside(
                    mouse_x,
                    mouse_y,
                    slider.x,
                    slider.y,
                    slider.width,
                    slider.height,
                ) {
                    slider_click = Some(slider.id.clone());
                    break;
                }
            }
            if let Some(id) = slider_click {
                // Clone id before moving it to dragged_slider
                self.dragged_slider = Some(id.clone());
                // Update value immediately on click
                self.handle_slider_drag(ctx, &id, mouse_x);
            }

            // Handle stepper clicks
            let mut stepper_click = None;
            for stepper in &self.stepper_areas {
                if self.is_inside(
                    mouse_x,
                    mouse_y,
                    stepper.x,
                    stepper.y,
                    stepper.width,
                    stepper.height,
                ) {
                    // Determine which button was clicked (left or right)
                    let button_width = stepper.width / 2.0;
                    let is_right = mouse_x >= stepper.x + button_width;
                    stepper_click = Some((stepper.id.clone(), is_right));
                    break;
                }
            }
            if let Some((id, is_right)) = stepper_click {
                self.handle_stepper_click(ctx, &id, is_right);
            }
        }

        // Handle slider dragging (continuous update while mouse is held)
        if is_mouse_button_down(MouseButton::Left) {
            // Clone the slider_id to avoid borrow checker issues
            if let Some(slider_id) = self.dragged_slider.clone() {
                let (mouse_x, _) = mouse_position();
                self.handle_slider_drag(ctx, &slider_id, mouse_x);
            }
        } else {
            // Stop dragging when mouse is released
            self.dragged_slider = None;
        }

        None
    }

    fn render(&mut self, ctx: &mut MacroquadContext) {
        use macroquad::prelude::*;

        // Clear interactive areas at the START of render() before populating them
        // This ensures button_areas from the previous frame are available in update()
        self.clear_areas();

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

        // DEBUG: Log mouse position and focus state
        let (mouse_x, mouse_y) = mouse_position();
        log::debug!(
            "Mouse position: ({}, {}), Focused setting: {:?}",
            mouse_x,
            mouse_y,
            self.focused_setting_index
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

        // Check for mouse hover focus - update focused_setting_index when hovering over a setting
        // Note: y_position is updated each frame by register_setting() to reflect the current rendered position
        let mut found_hover = false;
        for (idx, setting) in self.interactive_settings.iter().enumerate() {
            // Check if mouse is over this setting (using approximate height)
            let setting_height = 55.0;
            if mouse_y >= setting.y_position && mouse_y <= setting.y_position + setting_height {
                if self.focused_setting_index != Some(idx) {
                    self.focused_setting_index = Some(idx);
                    log::debug!(
                        "Focus changed on hover to: {} (index {})",
                        setting.label,
                        idx
                    );
                }
                found_hover = true;
                break;
            }
        }

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
            Some("output"),
            SettingType::Picker,
        );

        // Register Output button area for click detection
        self.button_areas.push(ButtonArea {
            id: "output".to_string(),
            x: margin_x,
            y: current_y,
            width: 650.0,
            height: row_height,
        });
        current_y += row_height + 10.0;

        // Check if synth output is selected
        let output_str = output.to_string();
        let is_synth = output_str.eq_ignore_ascii_case("Synth") || output_str.contains("Synth");

        log::debug!(
            "🔍 DEBUG: Output is '{}', is_synth={}",
            output_str,
            is_synth
        );

        if is_synth {
            log::debug!("🔍 DEBUG: Rendering SoundFont folder button (synth output selected)");
            // SoundFont selection - now with stepper control
            let current_soundfont_name = if let Some(index) = self.current_soundfont_index {
                if let Some(sf) = self.soundfont_files.get(index) {
                    // Extract filename from path for display
                    sf.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string()
                } else {
                    "No SoundFont".to_string()
                }
            } else {
                "No SoundFont".to_string()
            };

            // Get the list of SoundFont names for the stepper
            let soundfont_options: Vec<String> = self
                .soundfont_files
                .iter()
                .map(|sf| {
                    sf.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string()
                })
                .collect();

            let current_index = self.current_soundfont_index.unwrap_or(0);

            // Register the SoundFont selector as a picker type for keyboard navigation
            self.register_setting(
                "soundfont".to_string(),
                "SoundFont".to_string(),
                SettingType::Picker,
                current_y,
            );

            // Check if this stepper is focused
            let soundfont_focused = self
                .focused_setting()
                .map(|focused| focused.id == "soundfont")
                .unwrap_or(false);

            // Draw settings row background
            self.draw_settings_row(
                margin_x,
                current_y,
                650.0,
                row_height,
                "SoundFont",
                &current_soundfont_name,
                false,
                Some("soundfont"),
                SettingType::Picker,
            );

            // Draw stepper control on the right side
            let stepper_x = margin_x + 650.0 - 200.0;
            let stepper_y = current_y + (row_height - 24.0) / 2.0;
            let button_size = 24.0;

            self.draw_stepper(
                stepper_x,
                stepper_y,
                button_size,
                &current_soundfont_name,
                &soundfont_options,
                current_index,
                "soundfont",
                true, // is_cyclic - wrap around
                soundfont_focused,
            );

            current_y += row_height + 10.0;

            // SoundFont Folders - Add button
            let add_folder_btn_x = margin_x + 650.0 - 115.0;
            let add_folder_btn_y = current_y;
            let add_folder_btn_w = 115.0;
            let add_folder_btn_h = 31.0;

            // Register add folder button
            self.register_setting(
                "add_soundfont_folder".to_string(),
                "Add SoundFont Folder".to_string(),
                SettingType::Button,
                add_folder_btn_y,
            );

            // Check if this button is focused
            let add_folder_focused = self
                .focused_setting()
                .map(|focused| focused.id == "add_soundfont_folder")
                .unwrap_or(false);

            let add_folder_hovered = self.draw_button(
                add_folder_btn_x,
                add_folder_btn_y,
                add_folder_btn_w,
                add_folder_btn_h,
                "+ Add Folder",
                mouse_x >= add_folder_btn_x
                    && mouse_x <= add_folder_btn_x + add_folder_btn_w
                    && mouse_y >= add_folder_btn_y
                    && mouse_y <= add_folder_btn_y + add_folder_btn_h,
                add_folder_focused,
            );

            // Register button area for click detection
            log::debug!(
                "🔍 DEBUG: Adding SoundFont folder button to button_areas at ({}, {})",
                add_folder_btn_x,
                add_folder_btn_y
            );
            self.button_areas.push(ButtonArea {
                id: "add_soundfont_folder".to_string(),
                x: add_folder_btn_x,
                y: add_folder_btn_y,
                width: add_folder_btn_w,
                height: add_folder_btn_h,
            });

            // Draw label
            draw_text(
                "SoundFont Folders",
                margin_x + 15.0,
                add_folder_btn_y + 12.0,
                16.0,
                Color::from_rgba(255, 255, 255, 255),
            );
            draw_text(
                &format!("{} folders", self.soundfont_folders.len()),
                margin_x + 15.0,
                add_folder_btn_y + 32.0,
                12.0,
                Color::from_rgba(150, 150, 150, 255),
            );

            current_y += row_height + 10.0;

            // Audio Gain - Using slider instead of spin buttons
            let gain = ctx.config.audio_gain();
            let gain_min = 0.0;
            let gain_max = 1.0;
            let gain_step = 0.05;

            // Register the slider for keyboard navigation
            self.register_setting(
                "audio_gain_slider".to_string(),
                "Audio Gain".to_string(),
                SettingType::Slider,
                current_y,
            );

            // Check if this slider is focused
            let gain_focused = self
                .focused_setting()
                .map(|focused| focused.id == "audio_gain_slider")
                .unwrap_or(false);

            // Draw settings row background
            self.draw_settings_row(
                margin_x,
                current_y,
                650.0,
                row_height,
                "Audio Gain",
                &format!("{:.2}", gain),
                false,
                Some("audio_gain_slider"),
                SettingType::Slider,
            );

            // Draw slider on the right side
            let slider_x = margin_x + 650.0 - 200.0;
            let slider_y = current_y + (row_height - 30.0) / 2.0;
            let slider_w = 180.0;
            let slider_h = 30.0;

            self.draw_slider(
                slider_x,
                slider_y,
                slider_w,
                slider_h,
                gain,
                gain_min,
                gain_max,
                gain_step,
                "audio_gain_slider",
                gain_focused,
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
            Some("input"),
            SettingType::Picker,
        );

        // Register Input button area for click detection
        self.button_areas.push(ButtonArea {
            id: "input".to_string(),
            x: margin_x,
            y: current_y,
            width: 650.0,
            height: row_height,
        });
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
            Some("range_start"),
            SettingType::Spinner,
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
            Some("range_end"),
            SettingType::Spinner,
        );
        current_y += row_height + section_gap;

        // THEME SECTION
        draw_text(
            "PIANO THEME",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;

        let current_theme = ctx.config.piano_theme_name();
        self.register_setting(
            "piano_theme".to_string(),
            "Theme".to_string(),
            SettingType::Picker,
            current_y,
        );

        let theme_focused = self
            .focused_setting()
            .map(|focused| focused.id == "piano_theme")
            .unwrap_or(false);

        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Theme",
            current_theme,
            theme_focused,
            Some("piano_theme"),
            SettingType::Picker,
        );

        self.button_areas.push(ButtonArea {
            id: "piano_theme".to_string(),
            x: margin_x,
            y: current_y,
            width: 650.0,
            height: row_height,
        });
        current_y += row_height + 10.0;

        self.draw_mini_keyboard_preview(ctx, margin_x, current_y, 650.0, 50.0);
        current_y += 50.0 + section_gap;

        // RENDER SECTION
        draw_text(
            "RENDER",
            margin_x,
            current_y,
            22.0,
            Color::from_rgba(160, 81, 255, 255),
        );
        current_y += 30.0;

        // Draw Vertical Guidelines toggle
        let toggle_x = margin_x + 650.0 - 50.0;
        let toggle_y = current_y + (row_height - 20.0) / 2.0;
        let toggle_w = 40.0;
        let toggle_h = 20.0;

        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Vertical Guidelines",
            if ctx.config.vertical_guidelines() {
                "On"
            } else {
                "Off"
            },
            false,
            Some("vertical_guidelines"),
            SettingType::Toggle,
        );

        // Draw toggle widget on the right side
        let is_hovered = mouse_x >= toggle_x
            && mouse_x <= toggle_x + toggle_w
            && mouse_y >= toggle_y
            && mouse_y <= toggle_y + toggle_h;
        self.draw_toggle(
            toggle_x,
            toggle_y,
            toggle_w,
            toggle_h,
            ctx.config.vertical_guidelines(),
            is_hovered,
        );

        // Register toggle area for click detection
        self.toggle_areas.push(ToggleArea {
            id: "vertical_guidelines".to_string(),
            x: toggle_x,
            y: toggle_y,
            width: toggle_w,
            height: toggle_h,
            value: ctx.config.vertical_guidelines(),
        });

        current_y += row_height + 10.0;

        // Draw Horizontal Guidelines toggle
        let toggle_x = margin_x + 650.0 - 50.0;
        let toggle_y = current_y + (row_height - 20.0) / 2.0;
        let toggle_w = 40.0;
        let toggle_h = 20.0;

        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Horizontal Guidelines",
            if ctx.config.horizontal_guidelines() {
                "On"
            } else {
                "Off"
            },
            false,
            Some("horizontal_guidelines"),
            SettingType::Toggle,
        );

        // Draw toggle widget on the right side
        let is_hovered = mouse_x >= toggle_x
            && mouse_x <= toggle_x + toggle_w
            && mouse_y >= toggle_y
            && mouse_y <= toggle_y + toggle_h;
        self.draw_toggle(
            toggle_x,
            toggle_y,
            toggle_w,
            toggle_h,
            ctx.config.horizontal_guidelines(),
            is_hovered,
        );

        // Register toggle area for click detection
        self.toggle_areas.push(ToggleArea {
            id: "horizontal_guidelines".to_string(),
            x: toggle_x,
            y: toggle_y,
            width: toggle_w,
            height: toggle_h,
            value: ctx.config.horizontal_guidelines(),
        });

        current_y += row_height + 10.0;

        // Draw Glow toggle
        let toggle_x = margin_x + 650.0 - 50.0;
        let toggle_y = current_y + (row_height - 20.0) / 2.0;
        let toggle_w = 40.0;
        let toggle_h = 20.0;

        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Glow",
            if ctx.config.glow() { "On" } else { "Off" },
            false,
            Some("glow"),
            SettingType::Toggle,
        );

        // Draw toggle widget on the right side
        let is_hovered = mouse_x >= toggle_x
            && mouse_x <= toggle_x + toggle_w
            && mouse_y >= toggle_y
            && mouse_y <= toggle_y + toggle_h;
        self.draw_toggle(
            toggle_x,
            toggle_y,
            toggle_w,
            toggle_h,
            ctx.config.glow(),
            is_hovered,
        );

        // Register toggle area for click detection
        self.toggle_areas.push(ToggleArea {
            id: "glow".to_string(),
            x: toggle_x,
            y: toggle_y,
            width: toggle_w,
            height: toggle_h,
            value: ctx.config.glow(),
        });

        current_y += row_height + 10.0;

        // Draw Note Labels toggle
        let toggle_x = margin_x + 650.0 - 50.0;
        let toggle_y = current_y + (row_height - 20.0) / 2.0;
        let toggle_w = 40.0;
        let toggle_h = 20.0;

        self.draw_settings_row(
            margin_x,
            current_y,
            650.0,
            row_height,
            "Note Labels",
            if ctx.config.note_labels() {
                "On"
            } else {
                "Off"
            },
            false,
            Some("note_labels"),
            SettingType::Toggle,
        );

        // Draw toggle widget on the right side
        let is_hovered = mouse_x >= toggle_x
            && mouse_x <= toggle_x + toggle_w
            && mouse_y >= toggle_y
            && mouse_y <= toggle_y + toggle_h;
        self.draw_toggle(
            toggle_x,
            toggle_y,
            toggle_w,
            toggle_h,
            ctx.config.note_labels(),
            is_hovered,
        );

        // Register toggle area for click detection
        self.toggle_areas.push(ToggleArea {
            id: "note_labels".to_string(),
            x: toggle_x,
            y: toggle_y,
            width: toggle_w,
            height: toggle_h,
            value: ctx.config.note_labels(),
        });

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
            None, // Not interactive
            SettingType::Button,
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
            None, // Not interactive
            SettingType::Button,
        );
        current_y += row_height + 10.0;

        // Add Song Directory button
        let add_dir_btn_x = margin_x + 650.0 - 137.0;
        let add_dir_btn_y = current_y;
        let add_dir_btn_w = 137.0;
        let add_dir_btn_h = 31.0;

        // Register add directory button
        self.register_setting(
            "add_song_directory".to_string(),
            "Add Song Directory".to_string(),
            SettingType::Button,
            add_dir_btn_y,
        );

        // Check if this button is focused
        let add_dir_focused = self
            .focused_setting()
            .map(|focused| focused.id == "add_song_directory")
            .unwrap_or(false);

        let add_dir_hovered = self.draw_button(
            add_dir_btn_x,
            add_dir_btn_y,
            add_dir_btn_w,
            add_dir_btn_h,
            "+ Add Directory",
            mouse_x >= add_dir_btn_x
                && mouse_x <= add_dir_btn_x + add_dir_btn_w
                && mouse_y >= add_dir_btn_y
                && mouse_y <= add_dir_btn_y + add_dir_btn_h,
            add_dir_focused,
        );

        // Register button area for click detection
        log::debug!(
            "🔍 DEBUG: Adding song directory button to button_areas at ({}, {})",
            add_dir_btn_x,
            add_dir_btn_y
        );
        self.button_areas.push(ButtonArea {
            id: "add_song_directory".to_string(),
            x: add_dir_btn_x,
            y: add_dir_btn_y,
            width: add_dir_btn_w,
            height: add_dir_btn_h,
        });

        // Draw label
        draw_text(
            "Add new song directory",
            margin_x + 15.0,
            add_dir_btn_y + 12.0,
            16.0,
            Color::from_rgba(255, 255, 255, 255),
        );
        draw_text(
            "Browse to add MIDI files",
            margin_x + 15.0,
            add_dir_btn_y + 32.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );

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

        // Register back button for keyboard navigation
        self.register_setting(
            "back".to_string(),
            "Back".to_string(),
            SettingType::Button,
            bar_y + 10.0,
        );

        // IMPORTANT: Register back button in button_areas for click detection
        self.button_areas.push(ButtonArea {
            id: "back".to_string(),
            x: back_btn_x,
            y: back_btn_y,
            width: back_btn_w,
            height: back_btn_h,
        });

        let (back_mouse_x, back_mouse_y) = mouse_position();
        let back_hovered = back_mouse_x >= back_btn_x
            && back_mouse_x <= back_btn_x + back_btn_w
            && back_mouse_y >= back_btn_y
            && back_mouse_y <= back_btn_y + back_btn_h;

        // Check if back button is focused
        let back_focused = self
            .focused_setting()
            .map(|focused| focused.id == "back")
            .unwrap_or(false);

        // Draw back button with focus indicator
        let back_bg_color = if back_focused {
            Color::from_rgba(160, 81, 255, 255)
        } else if back_hovered {
            Color::from_rgba(100, 80, 120, 255)
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

        // Center text
        let back_text = "← Back";
        let back_text_width = measure_text(back_text, None, 14, 1.0).width;
        draw_text(
            back_text,
            back_btn_x + (back_btn_w - back_text_width) / 2.0,
            back_btn_y + (back_btn_h - 14.0) / 2.0 + 10.0,
            14.0,
            Color::from_rgba(255, 255, 255, 255),
        );

        // Draw instructions with keyboard controls
        draw_text(
            "↑↓: Navigate • Enter/Space: Activate • ←→: Adjust • ESC: Back",
            screen_w / 2.0 - 220.0,
            bar_y + 25.0,
            14.0,
            Color::from_rgba(150, 150, 150, 255),
        );

        // Draw focus indicator
        if let Some(focused) = self.focused_setting() {
            draw_text(
                &format!("Focused: {}", focused.label),
                screen_w - 200.0,
                bar_y + 25.0,
                14.0,
                Color::from_rgba(160, 81, 255, 255),
            );
        }

        // Draw popup overlay if active
        self.draw_popup(ctx);
    }
}
