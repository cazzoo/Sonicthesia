#![allow(clippy::collapsible_match, clippy::single_match)]

mod common;
mod context_macroquad;
mod icons;
mod input_manager;
mod input_stubs;
mod output_manager;
mod scene;
mod song;
mod song_library;
mod scoring_data;
mod utils;
mod lumi_controller;

mod virtual_resolution;
mod render;
mod scoring;
mod achievements;
mod challenges;
mod learning;
mod effects;
mod settings;
mod ui;

pub use common::NeothesiaEvent;
pub use common::PlayMode;
pub use common::SessionConfig;
pub use common::HandSelection;
pub use common::DifficultyLevel;
pub use song::Song;

// PLY Rendering (Macroquad) - DEFAULT
mod main_macroquad;
use main_macroquad::main as ply_main;

// ============================================================================
// PLY RENDERING ENTRY POINT (DEFAULT)
// ============================================================================

fn main() {
    ply_main();
}
