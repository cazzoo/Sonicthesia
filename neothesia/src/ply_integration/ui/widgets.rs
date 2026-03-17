//! PLY UI Widgets
//!
//! This module provides PLY-based implementations of common UI widgets
//! that mirror the functionality of the Nuon UI framework.

use super::{PlyUi, WidgetState, center_x, center_y, color_to_rgba, TextAlignment};
use std::hash::Hash;

/// Button widget builder
pub struct Button {
    id: Option<String>,
    label: String,
    icon: Option<String>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [u8; 3],
    hover_color: [u8; 3],
    pressed_color: [u8; 3],
    border_radius: [f32; 4],
    text_alignment: TextAlignment,
    font_size: f32,
}

impl Button {
    /// Create a new button
    pub fn new() -> Self {
        Self {
            id: None,
            label: String::new(),
            icon: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 40.0,
            color: [67, 67, 67],
            hover_color: [87, 87, 87],
            pressed_color: [97, 97, 97],
            border_radius: [5.0; 4],
            text_alignment: TextAlignment::Center,
            font_size: 16.0,
        }
    }
    
    /// Set button ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set button label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    
    /// Set button icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// Set button position
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set button size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Set button color
    pub fn color(mut self, color: [u8; 3]) -> Self {
        self.color = color;
        self
    }
    
    /// Set hover color
    pub fn hover_color(mut self, color: [u8; 3]) -> Self {
        self.hover_color = color;
        self
    }
    
    /// Set pressed color
    pub fn pressed_color(mut self, color: [u8; 3]) -> Self {
        self.pressed_color = color;
        self
    }
    
    /// Set border radius
    pub fn border_radius(mut self, radius: [f32; 4]) -> Self {
        self.border_radius = radius;
        self
    }
    
    /// Set text alignment
    pub fn text_alignment(mut self, alignment: TextAlignment) -> Self {
        self.text_alignment = alignment;
        self
    }
    
    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    
    /// Build and render the button, returning whether it was clicked
    pub fn build(self, ui: &mut PlyUi) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let id = self.id.unwrap_or_else(|| {
            if !self.label.is_empty() {
                format!("btn_{}", self.label)
            } else if let Some(ref icon) = self.icon {
                format!("btn_icon_{}", icon)
            } else {
                "btn_anon".to_string()
            }
        });
        
        // Generate a hash for the widget ID
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let widget_id = hasher.finish();
        
        let (offset_x, offset_y) = ui.current_offset();
        let x = self.x + offset_x;
        let y = self.y + offset_y;
        
        let state = ui.update_widget_state(widget_id, (x, y, self.width, self.height));
        
        // Determine background color based on state
        let bg_color = if state.pressed {
            self.pressed_color
        } else if state.hovered {
            self.hover_color
        } else {
            self.color
        };
        
        // Draw button background
        ui.add_command(super::RenderCommand::Quad {
            x,
            y,
            width: self.width,
            height: self.height,
            color: color_to_rgba(bg_color, 255),
            border_radius: self.border_radius,
        });
        
        // Draw label or icon
        if !self.label.is_empty() {
            ui.add_command(super::RenderCommand::Text {
                x,
                y,
                width: self.width,
                height: self.height,
                text: self.label,
                size: self.font_size,
                color: [255, 255, 255, 255],
                font: "Roboto".to_string(),
                alignment: self.text_alignment,
            });
        } else if let Some(ref icon) = self.icon {
            let icon_size = self.font_size;
            let icon_x = match self.text_alignment {
                TextAlignment::Left => x + 5.0,
                TextAlignment::Right => x + self.width - icon_size - 5.0,
                TextAlignment::Center => x + center_x(self.width, icon_size),
            };
            let icon_y = y + center_y(self.height, icon_size);
            ui.add_command(super::RenderCommand::Icon {
                x: icon_x,
                y: icon_y,
                size: icon_size,
                icon: icon.clone(),
                color: [255, 255, 255, 255],
            });
        }
        
        state.clicked
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

/// Label widget builder
pub struct Label {
    text: String,
    icon: Option<String>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [u8; 3],
    font_size: f32,
    bold: bool,
    alignment: TextAlignment,
    font_family: String,
}

impl Label {
    /// Create a new label
    pub fn new() -> Self {
        Self {
            text: String::new(),
            icon: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
            color: [255, 255, 255],
            font_size: 14.0,
            bold: false,
            alignment: TextAlignment::Center,
            font_family: "Roboto".to_string(),
        }
    }
    
    /// Set label text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
    
    /// Set label icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// Set label position
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set label size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Set text color
    pub fn color(mut self, color: [u8; 3]) -> Self {
        self.color = color;
        self
    }
    
    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    
    /// Set bold
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
    
    /// Set text alignment
    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set font family
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }
    
    /// Build and render the label
    pub fn build(self, ui: &mut PlyUi) {
        let (offset_x, offset_y) = ui.current_offset();
        let x = self.x + offset_x;
        let y = self.y + offset_y;
        
        // Draw text
        if !self.text.is_empty() {
            ui.add_command(super::RenderCommand::Text {
                x,
                y,
                width: self.width,
                height: self.height,
                text: self.text,
                size: self.font_size,
                color: color_to_rgba(self.color, 255),
                font: self.font_family,
                alignment: self.alignment,
            });
        }
        
        // Draw icon
        if let Some(ref icon) = self.icon {
            let icon_size = self.font_size;
            let icon_x = match self.alignment {
                TextAlignment::Left => x + 5.0,
                TextAlignment::Right => x + self.width - icon_size - 5.0,
                TextAlignment::Center => x + center_x(self.width, icon_size),
            };
            let icon_y = y + center_y(self.height, icon_size);
            ui.add_command(super::RenderCommand::Icon {
                x: icon_x,
                y: icon_y,
                size: icon_size,
                icon: icon.clone(),
                color: color_to_rgba(self.color, 255),
            });
        }
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

/// Quad (rectangle) widget builder
pub struct Quad {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [u8; 3],
    border_radius: [f32; 4],
}

impl Quad {
    /// Create a new quad
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: [100, 100, 100],
            border_radius: [0.0; 4],
        }
    }
    
    /// Set quad position
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set quad size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Set quad color
    pub fn color(mut self, color: [u8; 3]) -> Self {
        self.color = color;
        self
    }
    
    /// Set border radius
    pub fn border_radius(mut self, radius: [f32; 4]) -> Self {
        self.border_radius = radius;
        self
    }
    
    /// Build and render the quad
    pub fn build(self, ui: &mut PlyUi) {
        let (offset_x, offset_y) = ui.current_offset();
        let x = self.x + offset_x;
        let y = self.y + offset_y;
        
        ui.add_command(super::RenderCommand::Quad {
            x,
            y,
            width: self.width,
            height: self.height,
            color: color_to_rgba(self.color, 255),
            border_radius: self.border_radius,
        });
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self::new()
    }
}

/// Click area (invisible interactive region)
pub struct ClickArea {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ClickArea {
    /// Create a new click area
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
    
    /// Set click area position
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set click area size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Build and return click event
    pub fn build(self, ui: &mut PlyUi) -> ClickAreaEvent {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        let widget_id = hasher.finish();
        
        let (offset_x, offset_y) = ui.current_offset();
        let x = self.x + offset_x;
        let y = self.y + offset_y;
        
        let state = ui.update_widget_state(widget_id, (x, y, self.width, self.height));
        
        if state.clicked {
            ClickAreaEvent::Clicked
        } else if state.pressed {
            ClickAreaEvent::Pressed
        } else if state.hovered {
            ClickAreaEvent::Hovered
        } else {
            ClickAreaEvent::Idle
        }
    }
}

/// Click area event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAreaEvent {
    Idle,
    Hovered,
    Pressed,
    Clicked,
}

impl ClickAreaEvent {
    /// Check if clicked
    pub fn is_clicked(&self) -> bool {
        *self == Self::Clicked
    }
    
    /// Check if pressed
    pub fn is_pressed(&self) -> bool {
        *self == Self::Pressed
    }
    
    /// Check if hovered
    pub fn is_hovered(&self) -> bool {
        *self == Self::Hovered
    }
}

/// Scroll state
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    value: f32,
    max: f32,
}

impl ScrollState {
    /// Create a new scroll state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get current scroll value
    pub fn value(&self) -> f32 {
        self.value
    }
    
    /// Set scroll value
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, self.max);
    }
    
    /// Update scroll by delta
    pub fn update(&mut self, delta: f32) {
        self.value = (self.value - delta).clamp(0.0, self.max);
    }
    
    /// Set maximum scroll value
    pub fn set_max(&mut self, max: f32) {
        self.max = max.max(0.0);
        self.value = self.value.clamp(0.0, self.max);
    }
}

/// Scroll container
pub struct Scroll {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    scroll_state: ScrollState,
}

impl Scroll {
    /// Create a new scroll container
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
            scroll_state: ScrollState::new(),
        }
    }
    
    /// Set scroll position
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set scroll size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Set scroll state
    pub fn scroll(mut self, state: ScrollState) -> Self {
        self.scroll_state = state;
        self
    }
    
    /// Build scroll container and return updated scroll state
    pub fn build<F>(self, ui: &mut PlyUi, content: F) -> ScrollState
    where
        F: FnOnce(&mut PlyUi),
    {
        let (offset_x, offset_y) = ui.current_offset();
        let x = self.x + offset_x;
        let y = self.y + offset_y;
        
        // Push layer with scissor rect
        ui.push_layer();
        ui.set_scissor_rect(x, y, self.width, self.height);
        
        // Apply scroll offset
        ui.translate(0.0, -self.scroll_state.value());
        
        // Render content
        content(ui);
        
        // Pop layer
        ui.pop_layer();
        
        // Draw scroll bar if needed
        if self.scroll_state.max > 0.0 {
            self.draw_scrollbar(ui, x, y);
        }
        
        self.scroll_state
    }
    
    fn draw_scrollbar(&self, ui: &mut PlyUi, x: f32, y: f32) {
        let bar_width = 10.0;
        let bar_height = self.height;
        let bar_x = x + self.width - bar_width;
        
        // Calculate thumb size and position
        let content_ratio = self.height / (self.height + self.scroll_state.max);
        let thumb_height = (bar_height * content_ratio).max(20.0);
        let thumb_y = y + (self.scroll_state.value() / self.scroll_state.max) * (bar_height - thumb_height);
        
        // Draw track
        ui.add_command(super::RenderCommand::Quad {
            x: bar_x,
            y,
            width: bar_width,
            height: bar_height,
            color: [37, 35, 42, 255],
            border_radius: [5.0; 4],
        });
        
        // Draw thumb
        ui.add_command(super::RenderCommand::Quad {
            x: bar_x,
            y: thumb_y,
            width: bar_width,
            height: thumb_height,
            color: [74, 68, 88, 255],
            border_radius: [5.0; 4],
        });
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}
