//! PLY-based Settings Menu Implementation
//!
//! This module demonstrates how to migrate the settings menu from Nuon to PLY UI.

use crate::context::Context;
use crate::ply_integration::ui::{PlyUi, center_x, center_y, TextAlignment};
use crate::ply_integration::ui::widgets::{Button, Label, Quad};
use crate::ply_integration::ui::layout::{SettingsSection, SettingsRow, Scroll};

/// PLY-based settings menu state
pub struct PlySettingsMenu {
    ui: PlyUi,
    scroll_state: f32,
}

impl PlySettingsMenu {
    /// Create a new PLY-based settings menu
    pub fn new() -> Self {
        Self {
            ui: PlyUi::new(),
            scroll_state: 0.0,
        }
    }
    
    /// Update the settings menu UI
    pub fn update(&mut self, ctx: &mut Context) -> SettingsAction {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        // Begin frame
        self.ui.begin_frame(win_w, win_h);
        
        let mut action = SettingsAction::None;
        
        // Build settings menu UI
        self.build_settings_menu(ctx, &mut action);
        
        // End frame
        let commands = self.ui.end_frame();
        
        // TODO: Render commands using PLY renderer
        log::debug!("PLY Settings UI generated {} render commands", commands.len());
        
        action
    }
    
    /// Build the settings menu UI
    fn build_settings_menu(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let bottom_bar_h = 60.0;
        let margin_top = 40.0;
        let body_w = 650.0;
        
        // Draw bottom bar
        self.draw_bottom_bar(ctx, action);
        
        // Draw settings sections in scrollable area
        let scroll_height = (win_h - bottom_bar_h).max(0.0);
        
        Scroll::new()
            .pos(0.0, 0.0)
            .size(win_w, scroll_height)
            .scroll(crate::ply_integration::ui::widgets::ScrollState {
                value: self.scroll_state,
                max: 0.0, // Will be calculated by content
            })
            .build(&mut self.ui, |ui| {
                ui.translate(center_x(win_w, body_w), margin_top);
                
                // Output Section
                SettingsSection::new("Output")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        Self::output_section(ctx, ui, rows, action);
                    });
                
                // Input Section
                SettingsSection::new("Input")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        Self::input_section(ctx, ui, rows, action);
                    });
                
                // Note Range Section
                SettingsSection::new("Note Range")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        Self::note_range_section(ctx, ui, rows, action);
                    });
                
                // Render Section
                SettingsSection::new("Render")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        Self::render_section(ctx, ui, rows, action);
                    });
            });
    }
    
    /// Draw the bottom bar with back button
    fn draw_bottom_bar(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let padding = 10.0;
        let w = 80.0;
        let h = 60.0;
        
        // Draw bottom bar background
        Quad::new()
            .pos(0.0, win_h - padding - h)
            .size(win_w, h)
            .color([37, 35, 42])
            .build(&mut self.ui);
        
        // Draw back button
        if Button::new()
            .pos(padding, win_h - padding - h)
            .size(w, h)
            .label("←")
            .build(&mut self.ui)
        {
            *action = SettingsAction::GoBack;
        }
    }
    
    /// Output settings section
    fn output_section(ctx: &Context, ui: &mut PlyUi, rows: &dyn Fn(&mut PlyUi, SettingsRow), action: &mut SettingsAction) {
        let selected_output = ctx.config.output().as_deref().unwrap_or("None");
        
        SettingsRow::new()
            .title("Output")
            .subtitle(selected_output)
            .build(ui, |ui, row_w, row_h| {
                let btn_w = 320.0;
                let btn_h = 31.0;
                
                if Button::new()
                    .pos(row_w - btn_w, center_y(row_h, btn_h))
                    .size(btn_w, btn_h)
                    .label(selected_output)
                    .text_alignment(TextAlignment::Left)
                    .build(ui)
                {
                    *action = SettingsAction::ShowOutputPicker;
                }
            })
            .build(ui, rows);
    }
    
    /// Input settings section
    fn input_section(ctx: &Context, ui: &mut PlyUi, rows: &dyn Fn(&mut PlyUi, SettingsRow), action: &mut SettingsAction) {
        let selected_input = ctx.config.input().as_deref().unwrap_or("None");
        
        SettingsRow::new()
            .title("Input")
            .subtitle(selected_input)
            .build(ui, |ui, row_w, row_h| {
                let btn_w = 320.0;
                let btn_h = 31.0;
                
                if Button::new()
                    .pos(row_w - btn_w, center_y(row_h, btn_h))
                    .size(btn_w, btn_h)
                    .label(selected_input)
                    .text_alignment(TextAlignment::Left)
                    .build(ui)
                {
                    *action = SettingsAction::ShowInputPicker;
                }
            })
            .build(ui, rows);
    }
    
    /// Note range settings section
    fn note_range_section(ctx: &Context, ui: &mut PlyUi, rows: &dyn Fn(&mut PlyUi, SettingsRow), action: &mut SettingsAction) {
        let range = ctx.config.piano_range();
        
        SettingsRow::new()
            .title("Start")
            .subtitle(range.start().to_string())
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "range_start", action);
            })
            .build(ui, rows);
        
        // Spacer
        Quad::new()
            .size(650.0, 1.0)
            .color([0, 0, 0])
            .build(ui);
        
        ui.translate(0.0, 1.0);
        
        SettingsRow::new()
            .title("End")
            .subtitle(range.end().to_string())
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "range_end", action);
            })
            .build(ui, rows);
    }
    
    /// Render settings section
    fn render_section(ctx: &Context, ui: &mut PlyUi, rows: &dyn Fn(&mut PlyUi, SettingsRow), action: &mut SettingsAction) {
        // Vertical Guidelines
        SettingsRow::new()
            .title("Vertical Guidelines")
            .subtitle("Display octave indicators")
            .build(ui, |ui, row_w, row_h| {
                Self::draw_toggle(ui, row_w, row_h, ctx.config.vertical_guidelines(), "vertical_guidelines");
            })
            .build(ui, rows);
        
        // Horizontal Guidelines
        SettingsRow::new()
            .title("Horizontal Guidelines")
            .subtitle("Display measure/bar indicators")
            .build(ui, |ui, row_w, row_h| {
                Self::draw_toggle(ui, row_w, row_h, ctx.config.horizontal_guidelines(), "horizontal_guidelines");
            })
            .build(ui, rows);
        
        // Glow
        SettingsRow::new()
            .title("Glow")
            .subtitle("Key glow effect")
            .build(ui, |ui, row_w, row_h| {
                Self::draw_toggle(ui, row_w, row_h, ctx.config.glow(), "glow");
            })
            .build(ui, rows);
        
        // Note Labels
        SettingsRow::new()
            .title("Note Labels")
            .subtitle("Display waterfall note labels")
            .build(ui, |ui, row_w, row_h| {
                Self::draw_toggle(ui, row_w, row_h, ctx.config.note_labels(), "note_labels");
            })
            .build(ui, rows);
    }
    
    /// Draw spin buttons (plus/minus)
    fn draw_spin_buttons(ui: &mut PlyUi, row_w: f32, row_h: f32, id: &str, action: &mut SettingsAction) {
        let w = 30.0;
        let h = 30.0;
        let gap = 10.0;
        
        ui.translate(row_w - w, center_y(row_h, h));
        
        // Plus button
        if Button::new()
            .id(&format!("{}_plus", id))
            .size(w, h)
            .icon("+")
            .build(ui)
        {
            *action = SettingsAction::Increment(id.to_string());
        }
        
        ui.translate(-w, 0.0);
        ui.translate(-gap, 0.0);
        
        // Minus button
        if Button::new()
            .id(&format!("{}_minus", id))
            .size(w, h)
            .icon("-")
            .build(ui)
        {
            *action = SettingsAction::Decrement(id.to_string());
        }
    }
    
    /// Draw a toggle button
    fn draw_toggle(ui: &mut PlyUi, row_w: f32, row_h: f32, value: bool, id: &str) {
        let w = 30.0;
        let h = 15.0;
        
        Button::new()
            .id(id)
            .pos(row_w - w, center_y(row_h, h))
            .size(w, h)
            .color(if value { [160, 81, 255] } else { [74, 68, 88] })
            .border_radius([8.0; 4])
            .build(ui);
        
        // Draw toggle thumb
        let head_w = 12.0;
        let head_h = 12.0;
        let gap = 2.0;
        
        Quad::new()
            .pos(
                if value {
                    row_w - head_w - gap
                } else {
                    row_w - w + gap
                },
                center_y(row_h, head_h)
            )
            .size(head_w, head_h)
            .color([255, 255, 255])
            .border_radius([5.0; 4])
            .build(ui);
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
    
    /// Handle scroll
    pub fn scroll(&mut self, delta: f32) {
        self.scroll_state = (self.scroll_state - delta).max(0.0);
    }
}

/// Settings action returned by PLY UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    GoBack,
    ShowOutputPicker,
    ShowInputPicker,
    Increment(String),
    Decrement(String),
    Toggle(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_settings_creation() {
        let settings = PlySettingsMenu::new();
        assert_eq!(settings.scroll_state, 0.0);
    }
    
    #[test]
    fn test_settings_action_equality() {
        assert_eq!(SettingsAction::None, SettingsAction::None);
        assert_ne!(SettingsAction::GoBack, SettingsAction::ShowOutputPicker);
        assert_eq!(
            SettingsAction::Increment("test".to_string()),
            SettingsAction::Increment("test".to_string())
        );
    }
}
