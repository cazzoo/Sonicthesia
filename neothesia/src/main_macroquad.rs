//! Macroquad-based Neothesia main entry point
//!
//! This file replaces the winit+WGPU system with macroquad for full PLY rendering integration.

use macroquad::prelude::*;
use std::sync::Arc;

// Use existing modules from crate root
use crate::context_macroquad::MacroquadContext;
use crate::song::Song;

#[derive(Debug)]
pub enum NeothesiaEvent {
    Play(Song),
    FreePlay(Option<Song>),
    MainMenu(Option<Song>),
    ShowScore {
        song: Song,
        score_data: crate::scene::playing_scene::midi_player::ScoreData,
    },
    MidiInput {
        channel: u8,
        message: midi_file::midly::MidiMessage,
    },
    Exit,
}

struct MacroquadNeothesia {
    context: MacroquadContext,
    // For now, we'll use a simple placeholder instead of the full scene system
    // The full scene system integration will come later
}

impl MacroquadNeothesia {
    fn new() -> Self {
        let context = MacroquadContext::new();
        let _song = Song::from_env_macroquad(&context);

        Self {
            context,
        }
    }

    fn update(&mut self, _delta: std::time::Duration) {
        // Update PLY input handler
        self.context.ply_input_handler.update();

        // Update window state
        self.context.window_state.update();
    }

    fn render(&mut self) {
        clear_background(BLACK);

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

        draw_text(
            "PLY Migration in Progress",
            10.0,
            60.0,
            16.0,
            Color::from_rgba(255, 200, 0, 255),
        );

        draw_text(
            "Scene system integration coming soon",
            10.0,
            85.0,
            14.0,
            Color::from_rgba(200, 200, 200, 255),
        );
    }
}

#[macroquad::main("Neothesia")]
pub async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    let mut app = MacroquadNeothesia::new();
    let mut last_frame_time = std::time::Instant::now();

    loop {
        let delta = last_frame_time.elapsed();
        last_frame_time = std::time::Instant::now();

        // Handle window events
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Update
        app.update(delta);

        // Render
        app.render();

        // Request next frame
        next_frame().await;
    }
}
