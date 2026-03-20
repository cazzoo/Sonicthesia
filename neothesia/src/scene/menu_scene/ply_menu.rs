//! PLY-based Main Menu Implementation
//!
//! This module demonstrates how to migrate the main menu from Nuon to PLY UI
//! using the unified input system for a single focus indicator.

use crate::context::Context;
use crate::ply_integration::ui::{PlyUi, center_x, center_y, TextAlignment};
use crate::ply_integration::ui::widgets::{Button, Label, Quad};
use crate::ply_integration::input::UnifiedInputManager;
use crate::song::Song;

/// PLY-based main menu state
pub struct PlyMainMenu {
    ui: PlyUi,
    song: Option<Song>,
    input_manager: UnifiedInputManager,
}

impl PlyMainMenu {
    /// Create a new PLY-based main menu with unified input system
    pub fn new(song: Option<Song>) -> Self {
        let mut input_manager = UnifiedInputManager::new();
        
        // Initialize cursor visibility callback
        input_manager.focus().priority().set_cursor_visibility_callback(Box::new(|visible| {
            if visible {
                macroquad::input::show_mouse(true);
            } else {
                macroquad::input::show_mouse(false);
            }
        }));
        
        Self {
            ui: PlyUi::new(),
            song,
            input_manager,
        }
    }
    
    /// Update the main menu UI with unified input system
    pub fn update(&mut self, ctx: &mut Context) -> MenuAction {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        // Update unified input manager
        self.input_manager.update(0.016); // ~60fps
        
        // Begin frame
        self.ui.begin_frame(win_w, win_h);
        
        let mut action = MenuAction::None;
        
        // Build main menu UI
        self.build_main_menu(ctx, &mut action);
        
        // End frame
        let commands = self.ui.end_frame();
        
        // TODO: Render commands using PLY renderer
        // For now, we'll just store them
        log::debug!("PLY UI generated {} render commands", commands.len());
        
        action
    }
    
    /// Build the main menu UI
    fn build_main_menu(&mut self, ctx: &mut Context, action: &mut MenuAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let w = 450.0;
        let h = 80.0;
        let gap = 10.0;
        
        let logo_w = 650.0;
        let logo_h = 118.0;
        let post_logo_gap = 40.0;
        
        // Center the menu
        self.ui.translate(win_w / 2.0, win_h / 5.0);
        
        // Draw logo placeholder (quad for now)
        Quad::new()
            .pos(-logo_w / 2.0, 0.0)
            .size(logo_w, logo_h)
            .color([100, 100, 100])
            .build(&mut self.ui);
        
        // Draw menu buttons
        self.ui.translate(-w / 2.0, logo_h + post_logo_gap);
        
        // Select File button
        if Button::new()
            .size(w, h)
            .label("Select File")
            .build(&mut self.ui)
        {
            *action = MenuAction::SelectFile;
        }
        
        self.ui.translate(0.0, h + gap);
        
        // Play Mode button (only show when song is loaded)
        if self.song.is_some() {
            if Button::new()
                .size(w, h)
                .label("Play Mode")
                .build(&mut self.ui)
            {
                *action = MenuAction::GoToPlayMode;
            }
            
            self.ui.translate(0.0, h + gap);
        }
        
        // Song Library button
        let song_count = 0; // TODO: Get actual song count
        let label = if song_count > 0 {
            format!("📚 Song Library ({})", song_count)
        } else {
            "📚 Song Library".to_string()
        };
        
        if Button::new()
            .size(w, h)
            .label(&label)
            .color([180, 140, 100])
            .build(&mut self.ui)
        {
            *action = MenuAction::GoToSongLibrary;
        }
        
        self.ui.translate(0.0, h + gap);
        
        // Settings button
        if Button::new()
            .size(w, h)
            .label("Settings")
            .build(&mut self.ui)
        {
            *action = MenuAction::GoToSettings;
        }
        
        self.ui.translate(0.0, h + gap);
        
        // Exit button
        if Button::new()
            .size(w, h)
            .label("Exit")
            .build(&mut self.ui)
        {
            *action = MenuAction::Exit;
        }
        
        // Draw bottom bar
        self.ui.translate(0.0, win_h);
        self.ui.translate(0.0, -60.0); // Bottom bar height
        
        // Draw song name if loaded
        if let Some(ref song) = self.song {
            Label::new()
                .text(&song.file.name)
                .size(win_w, 60.0)
                .font_size(16.0)
                .build(&mut self.ui);
        }
    }
    
    /// Handle mouse movement with unified input system
    pub fn mouse_move(&mut self, x: f32, y: f32) {
        // Update unified input manager's mouse position
        self.input_manager.focus().priority().update_mouse_position(x, y);
        self.ui.mouse_move(x, y);
    }
    
    /// Handle mouse button press
    pub fn mouse_down(&mut self) {
        self.ui.mouse_down();
    }
    
    /// Handle mouse button release
    pub fn mouse_up(&mut self) {
        self.ui.mouse_up();
    }
    
    /// Handle scroll
    pub fn scroll(&mut self, delta: f32) {
        // TODO: Handle scroll for scrollable areas
    }
    
    /// Get the unified input manager for external event handling
    pub fn input_manager(&mut self) -> &mut UnifiedInputManager {
        &mut self.input_manager
    }
}

/// Menu action returned by PLY UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    None,
    SelectFile,
    GoToPlayMode,
    GoToSongLibrary,
    GoToSettings,
    GoToTrackSelection,
    Exit,
    FreePlay,
}

/// Example of how to integrate PLY menu into the existing menu scene
///
/// ```rust
/// impl MenuScene {
///     pub fn main_page_ui_ply(&mut self, ctx: &mut Context) {
///         // Create PLY menu
///         let mut ply_menu = PlyMainMenu::new(self.state.song().cloned());
///         
///         // Update and get action
///         let action = ply_menu.update(ctx);
///         
///         // Handle action
///         match action {
///             MenuAction::SelectFile => {
///                 self.futures.push(open_midi_file_picker(&mut self.state));
///             }
///             MenuAction::GoToPlayMode => {
///                 self.state.go_to(Page::PlayMode);
///             }
///             MenuAction::GoToSettings => {
///                 self.state.go_to(Page::Settings);
///             }
///             MenuAction::Exit => {
///                 self.state.go_back();
///             }
///             _ => {}
///         }
///     }
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_menu_creation() {
        let menu = PlyMainMenu::new(None);
        assert!(menu.song.is_none());
    }
    
    #[test]
    fn test_menu_action_equality() {
        assert_eq!(MenuAction::None, MenuAction::None);
        assert_ne!(MenuAction::Exit, MenuAction::SelectFile);
    }
}
