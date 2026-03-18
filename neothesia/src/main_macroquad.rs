//! Macroquad-based Neothesia main entry point
//!
//! This file replaces the winit+WGPU system with macroquad for full PLY rendering integration.

use macroquad::prelude::*;
use std::time::Duration;

/// Create the Macroquad configuration with anti-aliasing enabled
fn create_conf() -> Conf {
    Conf {
        window_title: "Neothesia".to_string(),
        sample_count: 4, // Enable 4x MSAA anti-aliasing
        ..Default::default()
    }
}

// Use existing modules from crate root
use crate::context_macroquad::MacroquadContext;
use crate::song::Song;
use crate::scene::{PlyScene, PlyMenuScene, PlyPlayingScene, PlyFreeplayScene, PlyScoreScene, PlySettingsScene};
use crate::NeothesiaEvent;

struct MacroquadNeothesia {
    context: MacroquadContext,
    current_scene: Box<dyn PlyScene>,
    event_queue: Vec<NeothesiaEvent>,
}

impl MacroquadNeothesia {
    fn new() -> Self {
        let context = MacroquadContext::new();
        let song = Song::from_env_macroquad(&context);
        
        // Start with the menu scene
        let current_scene = Box::new(PlyMenuScene::new(song)) as Box<dyn PlyScene>;

        Self {
            context,
            current_scene,
            event_queue: Vec::new(),
        }
    }

    fn update(&mut self, delta: Duration) {
        // Update PLY input handler
        self.context.ply_input_handler.update();

        // Update window state
        self.context.window_state.update();
        
        // Update current scene and check for events
        if let Some(event) = self.current_scene.update(&mut self.context, delta) {
            self.event_queue.push(event);
        }
        
        // Process events
        while let Some(event) = self.event_queue.pop() {
            self.handle_event(event);
        }
    }
    
    fn handle_event(&mut self, event: NeothesiaEvent) {
        match event {
            NeothesiaEvent::Play(song) => {
                self.current_scene = Box::new(PlyPlayingScene::new(song)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::FreePlay(song) => {
                self.current_scene = Box::new(PlyFreeplayScene::new(song)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::MainMenu(song) => {
                self.current_scene = Box::new(PlyMenuScene::new(song)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ShowSettings => {
                self.current_scene = Box::new(PlySettingsScene::new()) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ShowScore { song, score_data } => {
                self.current_scene = Box::new(PlyScoreScene::new(song, score_data)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::MidiInput { .. } => {
                // TODO: Handle MIDI input in scenes
                log::debug!("MIDI input received (not yet handled in PLY scenes)");
            }
            NeothesiaEvent::Exit => {
                // Will be handled in main loop
            }
        }
    }

    fn render(&mut self) {
        // Render current scene
        self.current_scene.render(&mut self.context);
    }
}

#[macroquad::main(create_conf)]
pub async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    let mut app = MacroquadNeothesia::new();
    let mut last_frame_time = std::time::Instant::now();
    let mut should_exit = false;

    loop {
        let delta = last_frame_time.elapsed();
        last_frame_time = std::time::Instant::now();

        // Check for exit event in queue
        if app.event_queue.iter().any(|e| matches!(e, NeothesiaEvent::Exit)) {
            should_exit = true;
        }

        if should_exit {
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
