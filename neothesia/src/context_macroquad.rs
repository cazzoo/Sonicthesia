//! Macroquad-based context for PLY rendering
//!
//! This replaces the WGPU-dependent Context with a macroquad-based version.

use std::sync::Arc;
use std::time::Duration;

use crate::{
    output_manager::OutputManager,
    song_library::{SongRepository, SongLibraryDatabase, default_db_path},
    NeothesiaEvent,
};

use crate::utils::window::WindowState;
use neothesia_core::config::Config;

/// Simplified PLY input handler for macroquad (doesn't use winit)
pub struct MacroquadPlyInputHandler {
    // For now, this is a placeholder
    // In the future, we'll integrate macroquad's input system here
}

impl MacroquadPlyInputHandler {
    pub fn new() -> Self {
        log::info!("🎯 PLY Input Handler initialized (Macroquad version)");
        Self {}
    }

    pub fn update(&mut self) {
        // Update input state using macroquad's input system
        // For now, this is a placeholder
    }
}

/// Macroquad-based context, replacing WGPU with macroquad rendering
pub struct MacroquadContext {
    /// Window state (adapted for macroquad)
    pub window_state: MacroquadWindowState,

    /// Output manager (audio/MIDI)
    pub output_manager: OutputManager,

    /// PLY input handler (macroquad version)
    pub ply_input_handler: MacroquadPlyInputHandler,

    /// Application configuration
    pub config: Config,

    /// Song library database
    pub song_library_db: SongLibraryDatabase,

    /// Last frame timestamp
    pub frame_timestamp: std::time::Instant,

    #[cfg(debug_assertions)]
    pub fps_ticker: MacroquadFpsTicker,
}

impl Drop for MacroquadContext {
    fn drop(&mut self) {
        self.config.save();
    }
}

impl MacroquadContext {
    pub fn new() -> Self {
        let config = Config::new();

        let song_library_db = SongLibraryDatabase::with_default_path()
            .unwrap_or_else(|e| {
                log::error!("Failed to initialize song library: {}. Song library features will be disabled.", e);
                SongLibraryDatabase::new(std::path::PathBuf::from("/tmp/neothesia_song_library_disabled.db"))
                    .unwrap_or_else(|_| {
                        log::error!("Completely unable to initialize any song library database");
                        std::process::exit(1);
                    })
            });

        let window_state = MacroquadWindowState::new();
        let ply_input_handler = MacroquadPlyInputHandler::new();

        Self {
            window_state,
            output_manager: Default::default(),
            ply_input_handler,
            config,
            song_library_db,
            frame_timestamp: std::time::Instant::now(),

            #[cfg(debug_assertions)]
            fps_ticker: MacroquadFpsTicker::new(),
        }
    }

    pub fn resize(&mut self) {
        // Macroquad handles window resizing automatically
        // Just update our state tracking
        self.window_state.update();
    }

    pub fn load_song_from_library(&mut self, song_id: i64) -> Option<crate::Song> {
        use crate::song_library::SongRepository;

        let entry = self.song_library_db.get_song(song_id).ok()??;

        let midi = midi_file::MidiFile::new(&entry.file_path).ok()?;

        self.config
            .set_last_opened_song(Some(entry.file_path.clone()));
        self.config.save();

        let mut song = crate::Song::new(midi);
        song.song_id = Some(song_id);
        Some(song)
    }

    pub fn refresh_song_library(&self) -> Result<(), crate::song_library::Error> {
        use crate::song_library::SongRepository;

        let song_dirs = self.config.song_directories();
        self.song_library_db.scan_directories(&song_dirs)?;
        Ok(())
    }
}

/// Window state adapted for macroquad
pub struct MacroquadWindowState {
    pub logical_size: macroquad::math::Vec2,
    pub physical_size: macroquad::math::Vec2,
    pub scale_factor: f32,
}

impl MacroquadWindowState {
    pub fn new() -> Self {
        Self {
            logical_size: macroquad::math::Vec2::new(800.0, 600.0), // Default size
            physical_size: macroquad::math::Vec2::new(800.0, 600.0),
            scale_factor: 1.0,
        }
    }

    pub fn update(&mut self) {
        // Macroquad handles screen size automatically
        // We'll update our tracking
        let width = macroquad::prelude::screen_width();
        let height = macroquad::prelude::screen_height();
        self.logical_size = macroquad::math::Vec2::new(width, height);
        self.physical_size = macroquad::math::Vec2::new(width, height);
    }

    pub fn width(&self) -> f32 {
        self.logical_size.x
    }

    pub fn height(&self) -> f32 {
        self.logical_size.y
    }
}

/// FPS ticker for debug builds
#[cfg(debug_assertions)]
pub struct MacroquadFpsTicker {
    fps: f32,
}

#[cfg(debug_assertions)]
impl MacroquadFpsTicker {
    pub fn new() -> Self {
        Self { fps: 60.0 }
    }

    pub fn tick(&mut self) {
        // get_fps() returns i32, convert to f32
        self.fps = macroquad::prelude::get_fps() as f32;
    }

    pub fn avg(&self) -> f32 {
        self.fps
    }
}

/// Wrapper to make MacroquadContext compatible with Scene trait
pub struct MacroquadContextWrapper<'a>(&'a mut MacroquadContext);

impl<'a> MacroquadContextWrapper<'a> {
    // This wrapper provides a compatibility layer
    // For now, scenes will need to be updated to work with MacroquadContext directly
    // This is a transitional type during the migration
}
