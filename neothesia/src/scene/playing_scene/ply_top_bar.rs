//! PLY-based Top Bar Implementation
//!
//! This module demonstrates how to migrate the in-game top bar from Nuon to PLY UI.

use crate::context::Context;
use crate::ply_integration::ui::{PlyUi, center_x, center_y, TextAlignment};
use crate::ply_integration::ui::widgets::{Button, Label, Quad, ClickArea};
use std::time::Duration;

/// PLY-based top bar state
pub struct PlyTopBar {
    ui: PlyUi,
    is_expanded: bool,
    settings_active: bool,
    looper_active: bool,
    loop_start: Duration,
    loop_end: Duration,
    animation_progress: f32,
}

impl PlyTopBar {
    /// Create a new PLY-based top bar
    pub fn new() -> Self {
        Self {
            ui: PlyUi::new(),
            is_expanded: true,
            settings_active: false,
            looper_active: false,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,
            animation_progress: 1.0,
        }
    }
    
    /// Update the top bar UI
    pub fn update(&mut self, ctx: &mut Context) -> TopBarAction {
        let win_w = ctx.window_state.logical_size.width;
        
        // Begin frame
        self.ui.begin_frame(win_w, 75.0);
        
        let mut action = TopBarAction::None;
        
        // Build top bar UI
        self.build_top_bar(ctx, &mut action);
        
        // End frame
        let commands = self.ui.end_frame();
        
        // TODO: Render commands using PLY renderer
        log::debug!("PLY Top Bar UI generated {} render commands", commands.len());
        
        action
    }
    
    /// Build the top bar UI
    fn build_top_bar(&mut self, ctx: &mut Context, action: &mut TopBarAction) {
        let win_w = ctx.window_state.logical_size.width;
        
        // Calculate animated Y position
        let y_offset = if self.animation_progress < 1.0 {
            -75.0 * (1.0 - self.animation_progress)
        } else {
            0.0
        };
        
        self.ui.translate(0.0, y_offset);
        
        // Draw top bar background
        Quad::new()
            .pos(0.0, 0.0)
            .size(win_w, 30.0 + 45.0)
            .color([37, 35, 42])
            .build(&mut self.ui);
        
        // Draw left panel (back button)
        self.draw_left_panel(ctx, action);
        
        // Draw center panel (speed and gain controls)
        self.draw_center_panel(ctx, action);
        
        // Draw right panel (control buttons)
        self.draw_right_panel(ctx, action);
        
        // Draw progress bar
        self.ui.translate(0.0, 30.0);
        self.draw_progress_bar(ctx, action);
    }
    
    /// Draw left panel with back button
    fn draw_left_panel(&mut self, _ctx: &Context, action: &mut TopBarAction) {
        if Button::new()
            .pos(0.0, 0.0)
            .size(30.0, 30.0)
            .icon("←")
            .border_radius([5.0; 4])
            .build(&mut self.ui)
        {
            *action = TopBarAction::GoBack;
        }
    }
    
    /// Draw center panel with speed and gain controls
    fn draw_center_panel(&mut self, ctx: &mut Context, action: &mut TopBarAction) {
        let win_w = ctx.window_state.logical_size.width;
        
        // Each group: label (50px) + minus (35px) + value (50px) + plus (35px) = 170px
        // Two groups = 340px, gap = 20px, total = 360px
        let group_w = 170.0;
        let gap = 20.0;
        let total_w = group_w * 2.0 + gap;
        let start_x = (win_w - total_w) / 2.0;
        
        // Speed group
        let speed_x = start_x;
        self.ui.translate(speed_x, 5.0);
        
        // Speed label
        Label::new()
            .text("Speed")
            .pos(0.0, 0.0)
            .size(50.0, 20.0)
            .build(&mut self.ui);
        
        // Speed minus button
        if Button::new()
            .id("speed_minus")
            .pos(50.0, 0.0)
            .size(35.0, 20.0)
            .icon("-")
            .color([67, 67, 67])
            .hover_color([87, 87, 87])
            .pressed_color([97, 97, 97])
            .border_radius([10.0, 0.0, 0.0, 10.0])
            .text_alignment(TextAlignment::Left)
            .build(&mut self.ui)
        {
            *action = TopBarAction::AdjustSpeed(-0.1);
        }
        
        // Speed value
        let speed_percent = (ctx.config.speed_multiplier() * 100.0).round();
        Label::new()
            .text(&format!("{}%", speed_percent))
            .pos(85.0, 0.0)
            .size(50.0, 20.0)
            .bold(true)
            .build(&mut self.ui);
        
        // Speed plus button
        if Button::new()
            .id("speed_plus")
            .pos(135.0, 0.0)
            .size(35.0, 20.0)
            .icon("+")
            .color([67, 67, 67])
            .hover_color([87, 87, 87])
            .pressed_color([97, 97, 97])
            .border_radius([0.0, 10.0, 10.0, 0.0])
            .text_alignment(TextAlignment::Right)
            .build(&mut self.ui)
        {
            *action = TopBarAction::AdjustSpeed(0.1);
        }
        
        self.ui.translate(-speed_x, -5.0);
        
        // Gain group
        let gain_x = start_x + group_w + gap;
        self.ui.translate(gain_x, 5.0);
        
        // Gain label
        Label::new()
            .text("Gain")
            .pos(0.0, 0.0)
            .size(50.0, 20.0)
            .build(&mut self.ui);
        
        // Gain minus button
        if Button::new()
            .id("gain_minus")
            .pos(50.0, 0.0)
            .size(35.0, 20.0)
            .icon("-")
            .color([67, 67, 67])
            .hover_color([87, 87, 87])
            .pressed_color([97, 97, 97])
            .border_radius([10.0, 0.0, 0.0, 10.0])
            .text_alignment(TextAlignment::Left)
            .build(&mut self.ui)
        {
            *action = TopBarAction::AdjustGain(-0.1);
        }
        
        // Gain value
        let gain_percent = 100.0; // TODO: Get actual gain value
        Label::new()
            .text(&format!("{}%", gain_percent))
            .pos(85.0, 0.0)
            .size(50.0, 20.0)
            .bold(true)
            .build(&mut self.ui);
        
        // Gain plus button
        if Button::new()
            .id("gain_plus")
            .pos(135.0, 0.0)
            .size(35.0, 20.0)
            .icon("+")
            .color([67, 67, 67])
            .hover_color([87, 87, 87])
            .pressed_color([97, 97, 97])
            .border_radius([0.0, 10.0, 10.0, 0.0])
            .text_alignment(TextAlignment::Right)
            .build(&mut self.ui)
        {
            *action = TopBarAction::AdjustGain(0.1);
        }
        
        self.ui.translate(-gain_x, -5.0);
    }
    
    /// Draw right panel with control buttons
    fn draw_right_panel(&mut self, _ctx: &Context, action: &mut TopBarAction) {
        let win_w = self.ui.current_offset().0; // Get current window width
        
        self.ui.translate(win_w, 0.0);
        self.ui.translate(-30.0, 0.0);
        
        // Wait mode button
        if Button::new()
            .size(30.0, 30.0)
            .icon("⏳")
            .color([56, 145, 255]) // Always blue for now
            .border_radius([5.0; 4])
            .build(&mut self.ui)
        {
            *action = TopBarAction::ToggleWaitMode;
        }
        
        self.ui.translate(-30.0, 0.0);
        
        // Settings button
        if Button::new()
            .size(30.0, 30.0)
            .icon(if self.settings_active { "⚙" } else { "⚙" })
            .border_radius([5.0; 4])
            .build(&mut self.ui)
        {
            *action = TopBarAction::ToggleSettings;
        }
        
        self.ui.translate(-30.0, 0.0);
        
        // Looper button
        if Button::new()
            .size(30.0, 30.0)
            .icon("🔁")
            .border_radius([5.0; 4])
            .build(&mut self.ui)
        {
            *action = TopBarAction::ToggleLooper;
        }
        
        self.ui.translate(-30.0, 0.0);
        
        // Play/Pause button
        if Button::new()
            .size(30.0, 30.0)
            .icon("▶") // Always show play for now
            .border_radius([5.0; 4])
            .build(&mut self.ui)
        {
            *action = TopBarAction::TogglePlayPause;
        }
    }
    
    /// Draw progress bar
    fn draw_progress_bar(&mut self, ctx: &mut Context, action: &mut TopBarAction) {
        let win_w = ctx.window_state.logical_size.width;
        let h = 45.0;
        
        // Draw progress bar background
        Quad::new()
            .pos(0.0, 0.0)
            .size(win_w, h)
            .color([56, 145, 255])
            .build(&mut self.ui);
        
        // Handle click on progress bar
        match ClickArea::new("progress_bar")
            .pos(0.0, 0.0)
            .size(win_w, h)
            .build(&mut self.ui)
        {
            crate::ply_integration::ui::widgets::ClickAreaEvent::Clicked => {
                // TODO: Calculate new position and seek
                *action = TopBarAction::Seek;
            }
            crate::ply_integration::ui::widgets::ClickAreaEvent::Pressed => {
                *action = TopBarAction::StartSeek;
            }
            _ => {}
        }
        
        // Draw looper if active
        if self.looper_active {
            self.draw_looper(win_w, h, action);
        }
    }
    
    /// Draw looper controls
    fn draw_looper(&mut self, win_w: f32, h: f32, action: &mut TopBarAction) {
        let loop_h = h + 10.0;
        
        // Draw looper background
        Quad::new()
            .pos(0.0, 0.0)
            .size(win_w, loop_h)
            .color([255, 56, 187, 90])
            .build(&mut self.ui);
        
        // TODO: Draw loop start/end handles and handle dragging
        // For now, just draw the looper area
    }
    
    /// Handle mouse movement
    pub fn mouse_move(&mut self, x: f32, y: f32) {
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
    
    /// Set expansion state
    pub fn set_expanded(&mut self, expanded: bool) {
        self.is_expanded = expanded;
    }
    
    /// Set settings active state
    pub fn set_settings_active(&mut self, active: bool) {
        self.settings_active = active;
    }
    
    /// Set looper active state
    pub fn set_looper_active(&mut self, active: bool) {
        self.looper_active = active;
    }
    
    /// Update animation progress
    pub fn update_animation(&mut self, progress: f32) {
        self.animation_progress = progress.clamp(0.0, 1.0);
    }
}

/// Top bar action returned by PLY UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopBarAction {
    None,
    GoBack,
    AdjustSpeed(f32),
    AdjustGain(f32),
    ToggleWaitMode,
    ToggleSettings,
    ToggleLooper,
    TogglePlayPause,
    Seek,
    StartSeek,
}

impl Default for PlyTopBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_top_bar_creation() {
        let top_bar = PlyTopBar::new();
        assert!(top_bar.is_expanded);
        assert_eq!(top_bar.animation_progress, 1.0);
    }
    
    #[test]
    fn test_top_bar_action_equality() {
        assert_eq!(TopBarAction::None, TopBarAction::None);
        assert_ne!(TopBarAction::GoBack, TopBarAction::TogglePlayPause);
        assert_eq!(
            TopBarAction::AdjustSpeed(0.1),
            TopBarAction::AdjustSpeed(0.1)
        );
    }
    
    #[test]
    fn test_set_expanded() {
        let mut top_bar = PlyTopBar::new();
        top_bar.set_expanded(false);
        assert!(!top_bar.is_expanded);
    }
    
    #[test]
    fn test_update_animation() {
        let mut top_bar = PlyTopBar::new();
        top_bar.update_animation(0.5);
        assert_eq!(top_bar.animation_progress, 0.5);
    }
}
