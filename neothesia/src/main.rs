#![allow(clippy::collapsible_match, clippy::single_match)]

mod common;
mod context;
mod context_macroquad;
mod icons;
mod input_manager;
mod output_manager;
mod scene;
mod song;
mod song_library;
mod utils;
mod lumi_controller;

mod ply_integration;
mod render;
mod scoring;
mod achievements;
mod challenges;
mod learning;
mod effects;

// Re-export common types for use throughout the codebase
pub use common::NeothesiaEvent;
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
