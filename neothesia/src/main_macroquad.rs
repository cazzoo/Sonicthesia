//! Macroquad-based Neothesia main entry point
//!
//! This file replaces the winit+WGPU system with macroquad for full PLY rendering integration.

use macroquad::prelude::*;
use std::time::Duration;

/// Create the Macroquad configuration with anti-aliasing enabled
fn create_conf() -> Conf {
    Conf {
        window_title: "Neothesia".to_string(),
        window_width: 1280,
        window_height: 720,
        sample_count: 4,
        ..Default::default()
    }
}

// Use existing modules from crate root
use crate::context_macroquad::MacroquadContext;
use crate::song::Song;
use crate::scene::{PlyScene, PlyMenuScene, PlyPlayingScene, PlyFreeplayScene, PlyScoreScene, PlySettingsScene, PlySongLibraryScene, PlyNewSongLibraryScene, PlySongSelectedScene};
use crate::NeothesiaEvent;

struct MacroquadNeothesia {
    context: MacroquadContext,
    current_scene: Box<dyn PlyScene>,
    event_queue: Vec<NeothesiaEvent>,
}

impl MacroquadNeothesia {
    fn new() -> Self {
        let mut context = MacroquadContext::new();
        let song = Song::from_env_macroquad(&context);
        
        let mut scene = PlyNewSongLibraryScene::new(song);
        scene.load_songs(&mut context);
        let current_scene = Box::new(scene) as Box<dyn PlyScene>;

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
                log::info!("🎯 EVENT: Play song '{}'", song.file.name);
                self.context.resume_playback_time = None;
                self.current_scene = Box::new(PlyPlayingScene::new(song)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ResumePlay(song, resume_time) => {
                log::info!("🎯 EVENT: ResumePlay song '{}' at {:.1}s", song.file.name, resume_time);
                self.current_scene = Box::new(PlyPlayingScene::new_resumed(song, resume_time)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::FreePlay(song) => {
                log::info!("🎯 EVENT: FreePlay with song: {:?}", song.as_ref().map(|s| &s.file.name));
                self.context.resume_playback_time = None;
                self.current_scene = Box::new(PlyFreeplayScene::new(song, &mut self.context)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::MainMenu(song) => {
                log::info!("🎯 EVENT: MainMenu with song: {:?}", song.as_ref().map(|s| &s.file.name));
                let playback_gain = self.context.config.playback_gain();
                self.context.output_manager.connection().set_gain(playback_gain);
                self.current_scene = Box::new(PlyMenuScene::new(song)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ShowSettings => {
                let mut scene = PlySettingsScene::new();
                scene.initialize(&mut self.context);
                self.current_scene = Box::new(scene) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ResumeFromSettings => {
                // Return directly to playing scene with resume state
                let playback_gain = self.context.config.playback_gain();
                self.context.output_manager.connection().set_gain(playback_gain);
                
                if let (Some(song), Some(resume_time)) = 
                    (self.context.resume_song.take(), self.context.resume_playback_time.take()) {
                    log::info!("🎯 EVENT: ResumeFromSettings to play at {:.1}s", resume_time);
                    self.current_scene = Box::new(PlyPlayingScene::new_resumed(song, resume_time)) as Box<dyn PlyScene>;
                } else {
                    // Fallback to menu if no resume state
                    log::warn!("🎯 EVENT: ResumeFromSettings but no resume state, going to menu");
                    self.current_scene = Box::new(PlyMenuScene::new(None)) as Box<dyn PlyScene>;
                }
            }
            NeothesiaEvent::ShowSongLibrary(song) => {
                let mut scene = PlyNewSongLibraryScene::new(song);
                scene.load_songs(&mut self.context);
                self.current_scene = Box::new(scene) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ShowSongSelected { song, entry: _ } => {
                let scene = PlySongSelectedScene::new(song);
                self.current_scene = Box::new(scene) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::ShowScore { song, score_data } => {
                self.context.resume_playback_time = None;
                self.current_scene = Box::new(PlyScoreScene::new(song, score_data)) as Box<dyn PlyScene>;
            }
            NeothesiaEvent::MidiInput { channel, message } => {
                self.current_scene.handle_midi_event(channel, &message);
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

    // Initialize custom fonts for Sonic Obsidian design
    crate::scene::ply_fonts::init_fonts().await;

    let mut app = MacroquadNeothesia::new();
    let mut last_frame_time = std::time::Instant::now();
    let mut should_exit = false;
    let mut frame_count: u32 = 0;

    loop {
        let delta = last_frame_time.elapsed();
        last_frame_time = std::time::Instant::now();

        // Log window and input state periodically
        frame_count += 1;
        if frame_count % 60 == 0 {
            // Every 60 frames (~1 second at 60fps)
            let screen_w = screen_width();
            let screen_h = screen_height();
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_left = is_mouse_button_pressed(MouseButton::Left);
            let mouse_down = is_mouse_button_down(MouseButton::Left);
            
            log::info!(
                "[MAIN] Frame {}: Screen={:.0}x{:.0}, Mouse=({:.1},{:.1}), Left: pressed={} down={}",
                frame_count, screen_w, screen_h, mouse_x, mouse_y, mouse_left, mouse_down
            );
        }

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
