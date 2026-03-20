//! PLY UI Layout Components
//!
//! This module provides layout components for organizing UI elements.

use super::{PlyUi, center_x, center_y, color_to_rgba, TextAlignment};

/// Card container with rounded corners
pub struct Card {
    padding: f32,
    background_color: [u8; 3],
    border_radius: [f32; 4],
}

impl Card {
    /// Create a new card
    pub fn new() -> Self {
        Self {
            padding: 15.0,
            background_color: [37, 35, 42],
            border_radius: [5.0; 4],
        }
    }
    
    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    
    /// Set background color
    pub fn background_color(mut self, color: [u8; 3]) -> Self {
        self.background_color = color;
        self
    }
    
    /// Set border radius
    pub fn border_radius(mut self, radius: [f32; 4]) -> Self {
        self.border_radius = radius;
        self
    }
    
    /// Build card with content
    pub fn build<F>(self, ui: &mut PlyUi, content: F) -> (f32, f32)
    where
        F: FnOnce(&mut PlyUi),
    {
        let (offset_x, offset_y) = ui.current_offset();

        // Draw card background
        // Note: In a real implementation, we'd measure the content size
        // For now, we'll use a default size
        let width = 400.0;
        let height = 300.0;

        ui.add_command(super::RenderCommand::Quad {
            x: offset_x,
            y: offset_y,
            width,
            height,
            color: color_to_rgba(self.background_color, 255),
            border_radius: self.border_radius,
        });

        // Push layer for content with padding
        ui.push_layer();
        ui.translate(self.padding, self.padding);

        content(ui);

        ui.pop_layer();

        (width, height)
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

/// Row group container
pub struct RowGroup {
    padding: f32,
    background_color: [u8; 3],
    border_radius: [f32; 4],
    gap: f32,
    width: f32,
}

impl RowGroup {
    /// Create a new row group
    pub fn new() -> Self {
        Self {
            padding: 15.0,
            background_color: [37, 35, 42],
            border_radius: [10.0; 4],
            gap: 10.0,
            width: 400.0,
        }
    }
    
    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    
    /// Set background color
    pub fn background_color(mut self, color: [u8; 3]) -> Self {
        self.background_color = color;
        self
    }
    
    /// Set border radius
    pub fn border_radius(mut self, radius: [f32; 4]) -> Self {
        self.border_radius = radius;
        self
    }
    
    /// Set gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Build row group with content
    pub fn build<F>(self, ui: &mut PlyUi, content: F) -> (f32, f32)
    where
        F: FnOnce(&mut PlyUi),
    {
        let (offset_x, offset_y) = ui.current_offset();

        // Draw row group background
        let width = self.width;
        let height = 200.0;

        ui.add_command(super::RenderCommand::Quad {
            x: offset_x,
            y: offset_y,
            width,
            height,
            color: color_to_rgba(self.background_color, 255),
            border_radius: self.border_radius,
        });

        // Push layer for content with padding
        ui.push_layer();
        ui.translate(self.padding, self.padding);

        content(ui);

        ui.pop_layer();

        (width, height)
    }
}

impl Default for RowGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer container for clipping
pub struct Layer {
    scissor_rect: Option<(f32, f32, f32, f32)>,
    overlay: bool,
}

impl Layer {
    /// Create a new layer
    pub fn new() -> Self {
        Self {
            scissor_rect: None,
            overlay: false,
        }
    }
    
    /// Set scissor rect
    pub fn scissor_rect(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.scissor_rect = Some((x, y, width, height));
        self
    }
    
    /// Set overlay mode
    pub fn overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }
    
    /// Build layer with content
    pub fn build<F>(self, ui: &mut PlyUi, content: F)
    where
        F: FnOnce(&mut PlyUi),
    {
        ui.push_layer();
        
        if let Some((x, y, w, h)) = self.scissor_rect {
            let (offset_x, offset_y) = ui.current_offset();
            ui.set_scissor_rect(x + offset_x, y + offset_y, w, h);
        }
        
        content(ui);
        
        ui.pop_layer();
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

/// Translate container for positioning
pub struct Translate {
    x: f32,
    y: f32,
}

impl Translate {
    /// Create a new translate
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    /// Set translation
    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    
    /// Set X translation
    pub fn x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }
    
    /// Set Y translation
    pub fn y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }
    
    /// Build translate with content
    pub fn build<F>(self, ui: &mut PlyUi, content: F)
    where
        F: FnOnce(&mut PlyUi),
    {
        ui.translate(self.x, self.y);
        content(ui);
        ui.translate(-self.x, -self.y);
    }
}

impl Default for Translate {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings section container
pub struct SettingsSection {
    label: String,
    width: f32,
}

impl SettingsSection {
    /// Create a new settings section
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            width: 650.0,
        }
    }
    
    /// Set section width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    
    /// Build settings section
    pub fn build<F>(self, ui: &mut PlyUi, content: F)
    where
        F: FnOnce(&mut PlyUi),
    {
        use super::widgets::Label;
        
        let spacer_label_h = 43.0;
        
        // Draw section label
        Label::new()
            .text(&self.label)
            .size(self.width, spacer_label_h)
            .font_size(14.6)
            .alignment(TextAlignment::Left)
            .bold(true)
            .build(ui);
        
        ui.translate(0.0, spacer_label_h);
        
        // Draw row group
        RowGroup::new()
            .width(self.width)
            .build(ui, content);
        
        ui.translate(0.0, 15.0); // Spacer after section
    }
}

/// Settings row container
pub struct SettingsRow {
    title: String,
    subtitle: String,
    width: f32,
}

impl SettingsRow {
    /// Create a new settings row
    pub fn new() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            width: 650.0,
        }
    }
    
    /// Set row title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    
    /// Set row subtitle
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }
    
    /// Set row width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    
    /// Build settings row
    pub fn build<F>(self, ui: &mut PlyUi, content: F)
    where
        F: FnOnce(&mut PlyUi, f32, f32),
    {
        use super::widgets::Label;
        
        let row_h = 54.0;
        let row_padding = 15.0;
        let row_inner_w = self.width - 2.0 * row_padding;
        
        ui.translate(row_padding, 0.0);
        
        let title_h = 14.6;
        let subtitle_h = 12.2;
        
        if self.subtitle.is_empty() {
            Label::new()
                .text(&self.title)
                .alignment(TextAlignment::Left)
                .font_size(title_h)
                .size(row_inner_w, row_h)
                .build(ui);
        } else {
            let gap = 5.0;
            let sum_h = title_h + subtitle_h + gap;
            let y = center_y(row_h, sum_h);
            
            Label::new()
                .pos(0.0, y)
                .text(&self.title)
                .alignment(TextAlignment::Left)
                .font_size(title_h)
                .size(row_inner_w, title_h)
                .build(ui);
            
            Label::new()
                .pos(0.0, y + gap + title_h)
                .text(&self.subtitle)
                .color([150, 150, 150])
                .alignment(TextAlignment::Left)
                .font_size(subtitle_h)
                .size(row_inner_w, subtitle_h)
                .build(ui);
        }
        
        content(ui, row_inner_w, row_h);
        
        ui.translate(-row_padding, row_h);
    }
}

impl Default for SettingsRow {
    fn default() -> Self {
        Self::new()
    }
}

/// Combo list for selecting items
pub fn combo_list<T>(
    ui: &mut PlyUi,
    id: &str,
    item_size: (f32, f32),
    items: &[T],
) -> Option<usize>
where
    T: std::fmt::Display + Clone,
{
    use super::widgets::{Button, Quad};
    
    let (item_w, item_h) = item_size;
    
    // Draw background
    Quad::new()
        .size(item_w, item_h * items.len() as f32)
        .color([27, 25, 32])
        .build(ui);
    
    let mut selected = None;
    
    for (nth, item) in items.iter().enumerate() {
        let item_id = format!("{}_{}", id, nth);
        
        if Button::new()
            .id(&item_id)
            .pos(0.0, item_h * nth as f32)
            .size(item_w, item_h)
            .label(&item.to_string())
            .text_alignment(TextAlignment::Left)
            .border_radius([5.0; 4])
            .hover_color([160, 81, 255])
            .pressed_color([180, 90, 255])
            .build(ui)
        {
            selected = Some(nth);
        }
    }
    
    selected
}

// Helper functions

/// Create a card
pub fn card() -> Card {
    Card::new()
}

/// Create a row group
pub fn row_group() -> RowGroup {
    RowGroup::new()
}

/// Create a layer
pub fn layer() -> Layer {
    Layer::new()
}

/// Create a translate
pub fn translate() -> Translate {
    Translate::new()
}

/// Create a settings section
pub fn settings_section(label: impl Into<String>) -> SettingsSection {
    SettingsSection::new(label)
}

/// Create a settings row
pub fn settings_row() -> SettingsRow {
    SettingsRow::new()
}
