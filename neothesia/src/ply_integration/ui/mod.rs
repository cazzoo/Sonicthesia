//! PLY UI Integration Module
//! 
//! This module provides a PLY-based UI framework that mirrors the functionality
//! of the existing Nuon UI framework, enabling gradual migration from Nuon to PLY.

use std::collections::HashMap;
use std::hash::Hash;

pub mod widgets;
pub mod layout;
pub mod input;

#[cfg(test)]
mod tests;

use widgets::*;
use layout::*;

/// PLY-based UI state manager
pub struct PlyUi {
    /// Current layout stack
    layout_stack: Vec<LayoutState>,
    
    /// Widget state tracking
    widget_states: HashMap<u64, WidgetState>,
    
    /// Current pointer position
    pointer_pos: (f32, f32),
    
    /// Mouse state
    mouse_pressed: bool,
    mouse_down: bool,
    
    /// Hovered widget
    hovered: Option<u64>,
    
    /// Active widget (being interacted with)
    active: Option<u64>,
    
    /// Render commands
    commands: Vec<RenderCommand>,
}

/// Layout state for positioning widgets
#[derive(Debug, Clone)]
struct LayoutState {
    /// Current offset
    offset: (f32, f32),
    /// Scissor rect (x, y, width, height)
    scissor_rect: Option<(f32, f32, f32, f32)>,
    /// Layer depth
    layer_depth: u32,
}

/// Widget interaction state
#[derive(Debug, Clone, Default)]
struct WidgetState {
    hovered: bool,
    pressed: bool,
    clicked: bool,
}

impl PlyUi {
    /// Create a new PLY UI instance
    pub fn new() -> Self {
        Self {
            layout_stack: vec![LayoutState::default()],
            widget_states: HashMap::new(),
            pointer_pos: (-1.0, -1.0),
            mouse_pressed: false,
            mouse_down: false,
            hovered: None,
            active: None,
            commands: Vec::new(),
        }
    }
    
    /// Begin a new frame
    pub fn begin_frame(&mut self, width: f32, height: f32) {
        self.layout_stack = vec![LayoutState::default()];
        self.commands.clear();
        
        // Reset transient states
        for state in self.widget_states.values_mut() {
            state.clicked = false;
        }
        
        self.mouse_pressed = false;
    }
    
    /// End the current frame and get render data
    pub fn end_frame(&mut self) -> Vec<RenderCommand> {
        self.commands.clone()
    }
    
    /// Handle mouse movement
    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.pointer_pos = (x, y);
    }
    
    /// Handle mouse button press
    pub fn mouse_down(&mut self) {
        self.mouse_pressed = true;
        self.mouse_down = true;
    }
    
    /// Handle mouse button release
    pub fn mouse_up(&mut self) {
        self.mouse_pressed = false;
        self.mouse_down = false;
        self.active = None;
    }
    
    /// Push a new layout layer
    pub fn push_layer(&mut self) {
        let current = self.layout_stack.last().cloned().unwrap_or_default();
        self.layout_stack.push(current);
    }
    
    /// Pop the current layout layer
    pub fn pop_layer(&mut self) {
        if self.layout_stack.len() > 1 {
            self.layout_stack.pop();
        }
    }
    
    /// Translate the current position
    pub fn translate(&mut self, dx: f32, dy: f32) {
        if let Some(layer) = self.layout_stack.last_mut() {
            layer.offset.0 += dx;
            layer.offset.1 += dy;
        }
    }
    
    /// Set scissor rect for clipping
    pub fn set_scissor_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if let Some(layer) = self.layout_stack.last_mut() {
            layer.scissor_rect = Some((x, y, width, height));
        }
    }
    
    /// Get current offset
    pub fn current_offset(&self) -> (f32, f32) {
        self.layout_stack
            .last()
            .map(|l| l.offset)
            .unwrap_or((0.0, 0.0))
    }
    
    /// Check if a point is in the scissor rect
    pub fn in_scissor_rect(&self, x: f32, y: f32) -> bool {
        if let Some(layer) = self.layout_stack.last() {
            if let Some((sx, sy, sw, sh)) = layer.scissor_rect {
                return x >= sx && x <= sx + sw && y >= sy && y <= sy + sh;
            }
        }
        true
    }
    
    /// Update widget state based on interaction
    pub fn update_widget_state(&mut self, id: u64, rect: (f32, f32, f32, f32)) -> WidgetState {
        let (x, y, w, h) = rect;
        let (px, py) = self.pointer_pos;
        
        // Check if mouse is over widget (in scissor rect)
        let mouseover = self.in_scissor_rect(px, py) && px >= x && px <= x + w && py >= y && py <= y + h;
        
        // Update hovered state
        if mouseover {
            self.hovered = Some(id);
        } else if self.hovered == Some(id) {
            self.hovered = None;
        }
        
        // Check for click/press
        let is_hovered = self.hovered == Some(id);
        let is_active = self.active == Some(id);
        
        let clicked = if self.mouse_pressed && mouseover && self.active.is_none() {
            self.active = Some(id);
            false
        } else if !self.mouse_down && is_active {
            self.active = None;
            mouseover
        } else {
            false
        };
        
        let state = WidgetState {
            hovered: is_hovered && !is_active,
            pressed: is_active,
            clicked,
        };
        
        self.widget_states.insert(id, state.clone());
        state
    }
    
    /// Get widget state
    pub fn get_widget_state(&self, id: u64) -> WidgetState {
        self.widget_states.get(&id).cloned().unwrap_or_default()
    }
    
    /// Add a render command
    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}

impl Default for PlyUi {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            offset: (0.0, 0.0),
            scissor_rect: None,
            layer_depth: 0,
        }
    }
}

/// Render command for drawing UI elements
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Draw a rectangle
    Quad {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [u8; 4],
        border_radius: [f32; 4],
    },
    /// Draw text
    Text {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        text: String,
        size: f32,
        color: [u8; 4],
        font: String,
        alignment: TextAlignment,
    },
    /// Draw an icon
    Icon {
        x: f32,
        y: f32,
        size: f32,
        icon: String,
        color: [u8; 4],
    },
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

// Re-export commonly used types
pub use input::PlyInputHandler;

/// Helper function to center content horizontally
pub fn center_x(container_w: f32, item_w: f32) -> f32 {
    container_w / 2.0 - item_w / 2.0
}

/// Helper function to center content vertically
pub fn center_y(container_h: f32, item_h: f32) -> f32 {
    container_h / 2.0 - item_h / 2.0
}

/// Convert color to RGBA
pub fn color_to_rgba(color: [u8; 3], alpha: u8) -> [u8; 4] {
    [color[0], color[1], color[2], alpha]
}
