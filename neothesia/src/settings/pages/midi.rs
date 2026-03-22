use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::GlassPanel;
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

pub struct MidiPage {
    scroll_offset: f32,
}

impl MidiPage {
    pub fn new() -> Self {
        Self { scroll_offset: 0.0 }
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "MIDI Setup",
            x,
            y + 32.0,
            32.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Configure MIDI input/output devices and latency settings",
            x,
            y + 56.0,
            16.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );
        y + 80.0
    }

    fn render_device_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 280.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Device Selection",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let row_y = y + 60.0;
        let row_h = 56.0;
        let row_width = width - spacing::XL * 2.0;

        // Input device dropdown
        let input = config.input().unwrap_or("None");
        let input_hovered = self.render_dropdown_row(
            x + spacing::XL,
            row_y,
            row_width,
            row_h,
            "Input Device",
            input,
            mx,
            my,
        );

        // Output device dropdown
        let output = config.output().unwrap_or("None");
        let output_hovered = self.render_dropdown_row(
            x + spacing::XL,
            row_y + row_h + spacing::SM,
            row_width,
            row_h,
            "Output Device",
            output,
            mx,
            my,
        );

        // Separate channels toggle
        let channels = config.separate_channels();
        let channels_toggled = self.render_toggle_row(
            x + spacing::XL,
            row_y + (row_h + spacing::SM) * 2.0,
            row_width,
            row_h,
            "Separate Channels",
            channels,
            mx,
            my,
            mouse_pressed,
        );

        let interaction = if input_hovered && mouse_pressed {
            SettingsInteraction::OpenPopup("input".to_string())
        } else if output_hovered && mouse_pressed {
            SettingsInteraction::OpenPopup("output".to_string())
        } else if let Some(new_val) = channels_toggled {
            SettingsInteraction::SeparateChannelsToggled(new_val)
        } else {
            SettingsInteraction::None
        };

        (y + 280.0 + spacing::LG, interaction)
    }

    fn render_latency_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 180.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Latency Compensation",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let slider_y = y + 70.0;
        let slider_w = width - spacing::XL * 2.0;
        let slider_h = 40.0;

        // Track background
        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            x + spacing::XL,
            slider_y + 18.0,
            slider_w,
            4.0,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        // Filled portion (12ms / 100ms = 12%)
        let fill_w = slider_w * 0.12;
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::SECONDARY);
        draw_rectangle(
            x + spacing::XL,
            slider_y + 18.0,
            fill_w,
            4.0,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        // Thumb
        let thumb_x = x + spacing::XL + fill_w - 8.0;
        let (thumb_r, thumb_g, thumb_b) = colors::to_normalized(colors::SECONDARY);
        draw_circle(
            thumb_x,
            slider_y + 20.0,
            8.0,
            Color::new(thumb_r, thumb_g, thumb_b, 1.0),
        );

        // Label
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "12ms",
            x + spacing::XL,
            slider_y - 8.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        // Range labels
        let (range_r, range_g, range_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "0ms",
            x + spacing::XL,
            slider_y + 44.0,
            12.0,
            Color::new(range_r, range_g, range_b, 1.0),
        );
        draw_text(
            "100ms",
            x + width - spacing::XL - 40.0,
            slider_y + 44.0,
            12.0,
            Color::new(range_r, range_g, range_b, 1.0),
        );

        (y + 180.0 + spacing::LG, SettingsInteraction::None)
    }

    fn render_pedal_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 140.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Pedal Response",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let row_y = y + 60.0;
        let row_h = 40.0;
        let row_width = width - spacing::XL * 2.0;

        let sustain_toggled = self.render_toggle_row(
            x + spacing::XL,
            row_y,
            row_width,
            row_h,
            "Invert Sustain (CC64)",
            false,
            mx,
            my,
            mouse_pressed,
        );

        let expression_toggled = self.render_toggle_row(
            x + spacing::XL,
            row_y + row_h + spacing::SM,
            row_width,
            row_h,
            "Continuous Expression",
            true,
            mx,
            my,
            mouse_pressed,
        );

        (y + 140.0 + spacing::LG, SettingsInteraction::None)
    }

    fn render_dropdown_row(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        label: &str,
        value: &str,
        mx: f32,
        my: f32,
    ) -> bool {
        let is_hovered = mx >= x && mx <= x + width && my >= y && my <= y + height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
        draw_rectangle(
            x,
            y,
            width,
            height,
            Color::new(bg_r, bg_g, bg_b, if is_hovered { 0.8 } else { 0.4 }),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            label,
            x + spacing::MD,
            y + 22.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            value,
            x + spacing::MD,
            y + 42.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        draw_text(
            "▼",
            x + width - 24.0,
            y + 32.0,
            12.0,
            Color::new(value_r, value_g, value_b, 0.5),
        );

        is_hovered
    }

    fn render_toggle_row(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        label: &str,
        value: bool,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> Option<bool> {
        let is_hovered = mx >= x && mx <= x + width && my >= y && my <= y + height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
        draw_rectangle(
            x,
            y,
            width,
            height,
            Color::new(bg_r, bg_g, bg_b, if is_hovered { 0.8 } else { 0.4 }),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            label,
            x + spacing::MD,
            y + 26.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        // Toggle switch
        let toggle_x = x + width - 50.0;
        let toggle_y = y + (height - 20.0) / 2.0;
        let toggle_w = 40.0;
        let toggle_h = 20.0;

        let (track_r, track_g, track_b) = colors::to_normalized(if value {
            colors::PRIMARY
        } else {
            colors::SURFACE_CONTAINER_HIGHEST
        });
        draw_rectangle(
            toggle_x,
            toggle_y,
            toggle_w,
            toggle_h,
            Color::new(track_r, track_g, track_b, if value { 0.3 } else { 1.0 }),
        );

        let thumb_x = if value {
            toggle_x + toggle_w - 18.0
        } else {
            toggle_x + 2.0
        };
        let (thumb_r, thumb_g, thumb_b) = colors::to_normalized(if value {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE_VARIANT
        });
        draw_circle(
            thumb_x + 8.0,
            toggle_y + 10.0,
            8.0,
            Color::new(thumb_r, thumb_g, thumb_b, 1.0),
        );

        // Check for click on toggle
        let toggle_hovered = mx >= toggle_x
            && mx <= toggle_x + toggle_w
            && my >= toggle_y
            && my <= toggle_y + toggle_h;

        if toggle_hovered && mouse_pressed {
            Some(!value)
        } else {
            None
        }
    }
}

impl SettingsPage for MidiPage {
    fn title(&self) -> &str {
        "MIDI Setup"
    }

    fn description(&self) -> &str {
        "Configure MIDI input/output devices and latency settings"
    }

    fn render(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> SettingsInteraction {
        let content_x = x + spacing::XL;
        let content_width = width - spacing::XL * 2.0;
        let mut current_y = self.render_header(content_x, y - self.scroll_offset, content_width);

        let (next_y, interaction) = self.render_device_section(
            content_x,
            current_y,
            content_width,
            config,
            mx,
            my,
            mouse_pressed,
        );
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }
        current_y = next_y;

        let (next_y, interaction) =
            self.render_latency_section(content_x, current_y, content_width, mx, my, mouse_pressed);
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }
        current_y = next_y;

        let (_, interaction) =
            self.render_pedal_section(content_x, current_y, content_width, mx, my, mouse_pressed);
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
