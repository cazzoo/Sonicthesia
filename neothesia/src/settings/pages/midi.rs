use crate::scene::ply_fonts;
use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::{Dropdown, GlassPanel, SectionHeader, Slider, ToggleSwitch};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

pub struct MidiPage {
    scroll_offset: f32,
    input_open: bool,
    output_open: bool,
    pub input_devices: Vec<String>,
    pub output_devices: Vec<String>,
    pub pressure_history: Vec<f32>,
    pub active_pressure: f32,
}

impl MidiPage {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            input_open: false,
            output_open: false,
            input_devices: Vec::new(),
            output_devices: Vec::new(),
            pressure_history: vec![0.0; 128],
            active_pressure: 0.0,
        }
    }

    pub fn close_dropdowns(&mut self) {
        self.input_open = false;
        self.output_open = false;
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "MIDI Setup",
            x,
            y + 28.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let line_x = x + 120.0;
        let line_w = width - 120.0;
        let (line_r, line_g, line_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            line_x,
            y + 22.0,
            line_w,
            1.0,
            Color::new(line_r, line_g, line_b, 0.2),
        );

        y + 56.0
    }

    fn render_device_section(
        &mut self,
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
        ply_fonts::draw_headline(
            "Device Selection",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let row_y = y + 60.0;
        let row_h = 56.0;
        let row_width = width - spacing::XL * 2.0;

        let input = config.input().unwrap_or("None");
        let input_devices = &self.input_devices;

        let (input_interaction, input_height) = self.render_dropdown_with_list(
            x + spacing::XL,
            row_y,
            row_width,
            row_h,
            "Input Device",
            input,
            input_devices,
            mx,
            my,
            mouse_pressed,
            self.input_open,
        );

        let output = config.output().unwrap_or("None");
        let output_devices = &self.output_devices;

        let (output_interaction, _output_height) = self.render_dropdown_with_list(
            x + spacing::XL,
            row_y + row_h + spacing::SM + input_height,
            row_width,
            row_h,
            "Output Device",
            output,
            &output_devices,
            mx,
            my,
            mouse_pressed,
            self.output_open,
        );

        let mut interaction = SettingsInteraction::None;
        if let Some(idx) = input_interaction {
            if idx < input_devices.len() {
                interaction = SettingsInteraction::InputDeviceSelected(input_devices[idx].clone());
                self.input_open = false;
            } else {
                self.input_open = !self.input_open;
                self.output_open = false;
            }
        }
        if let Some(idx) = output_interaction {
            if idx < output_devices.len() {
                interaction =
                    SettingsInteraction::OutputDeviceSelected(output_devices[idx].clone());
                self.output_open = false;
            } else {
                self.output_open = !self.output_open;
                self.input_open = false;
            }
        }

        (y + 280.0 + spacing::LG, interaction)
    }

    fn render_dropdown_with_list(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        label: &str,
        value: &str,
        options: &[String],
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        is_open: bool,
    ) -> (Option<usize>, f32) {
        let is_hovered = mx >= x && mx <= x + width && my >= y && my <= y + height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
        draw_rectangle(
            x,
            y,
            width,
            height,
            Color::new(
                bg_r,
                bg_g,
                bg_b,
                if is_hovered || is_open { 0.8 } else { 0.4 },
            ),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_body(
            label,
            x + spacing::MD,
            y + 22.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            value,
            x + spacing::MD,
            y + 42.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        let arrow = if is_open { "▲" } else { "▼" };
        ply_fonts::draw_body(
            arrow,
            x + width - 24.0,
            y + 32.0,
            12.0,
            Color::new(value_r, value_g, value_b, 0.5),
        );

        let mut clicked: Option<usize> = None;

        if is_hovered && mouse_pressed && !is_open {
            clicked = Some(usize::MAX);
        }

        let mut extra_height = 0.0;

        if is_open && !options.is_empty() {
            let item_h = 32.0;
            let list_h = options.len() as f32 * item_h;
            extra_height = list_h;

            let list_y = y + height;

            let (list_bg_r, list_bg_r2, list_bg_r3) =
                colors::to_normalized(colors::SURFACE_CONTAINER);
            draw_rectangle(
                x,
                list_y,
                width,
                list_h,
                Color::new(list_bg_r, list_bg_r2, list_bg_r3, 0.95),
            );

            let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
            draw_rectangle_lines(
                x,
                list_y,
                width,
                list_h,
                1.0,
                Color::new(border_r, border_g, border_b, 0.3),
            );

            for (i, option) in options.iter().enumerate() {
                let item_y = list_y + i as f32 * item_h;
                let item_hovered =
                    mx >= x && mx <= x + width && my >= item_y && my <= item_y + item_h;
                let is_selected = option == value;

                if item_hovered {
                    let (hover_r, hover_g, hover_b) =
                        colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
                    draw_rectangle(
                        x + 4.0,
                        item_y,
                        width - 8.0,
                        item_h,
                        Color::new(hover_r, hover_g, hover_b, 0.5),
                    );
                }

                let (text_r, text_g, text_b) = if is_selected {
                    colors::to_normalized(colors::PRIMARY)
                } else if item_hovered {
                    colors::to_normalized(colors::ON_SURFACE)
                } else {
                    colors::to_normalized(colors::ON_SURFACE_VARIANT)
                };

                ply_fonts::draw_body(
                    option,
                    x + spacing::MD,
                    item_y + item_h / 2.0 + 4.0,
                    12.0,
                    Color::new(text_r, text_g, text_b, 1.0),
                );

                if item_hovered && mouse_pressed {
                    clicked = Some(i);
                }
            }

            if mx < x || mx > x + width || my < y || my > list_y + list_h {
                if mouse_pressed {
                    clicked = Some(usize::MAX);
                }
            }
        }

        (clicked, extra_height)
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
        ply_fonts::draw_headline(
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
        ply_fonts::draw_body(
            "12ms",
            x + spacing::XL,
            slider_y - 8.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        // Range labels
        let (range_r, range_g, range_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "0ms",
            x + spacing::XL,
            slider_y + 44.0,
            12.0,
            Color::new(range_r, range_g, range_b, 1.0),
        );
        ply_fonts::draw_body(
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
        ply_fonts::draw_headline(
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

    fn render_velocity_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> (f32, SettingsInteraction) {
        let enabled = config.velocity_enabled();
        let panel_h = if enabled { 340.0 } else { 120.0 };
        let panel = GlassPanel::new(x, y, width, panel_h);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        ply_fonts::draw_headline(
            "Pressure Sensitivity",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "Control volume and expression from key pressure",
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let row_y = y + 60.0;
        let row_w = width - spacing::XL * 2.0;
        let row_h = 40.0;

        let toggle_result = self.render_toggle_row(
            x + spacing::XL,
            row_y,
            row_w,
            row_h,
            "Enable Pressure Sensitivity",
            enabled,
            mx,
            my,
            mouse_pressed,
        );

        let mut interaction = match toggle_result {
            Some(val) => SettingsInteraction::VelocityEnabledToggled(val),
            None => SettingsInteraction::None,
        };

        if enabled {
            let slider_x = x + spacing::XL;
            let slider_w = row_w;
            let slider_y = row_y + row_h + spacing::LG;

            let (_, base_interaction) = self.render_hslider(
                slider_x,
                slider_y,
                slider_w,
                "Base Volume",
                config.velocity_min(),
                mx,
                my,
                mouse_pressed,
                mouse_down,
            );
            if let Some(v) = base_interaction {
                interaction = SettingsInteraction::VelocityMinChanged(v);
            }

            let (_, sens_interaction) = self.render_hslider(
                slider_x,
                slider_y + 56.0,
                slider_w,
                "Pressure Sensitivity",
                config.pressure_sensitivity() / 2.0,
                mx,
                my,
                mouse_pressed,
                mouse_down,
            );
            if let Some(v) = sens_interaction {
                interaction = SettingsInteraction::PressureSensitivityChanged(v * 2.0);
            }

            let graph_x = slider_x;
            let graph_y = slider_y + 120.0;
            let graph_w = slider_w;
            let graph_h = 80.0;

            self.render_pressure_graph(graph_x, graph_y, graph_w, graph_h);
        }

        (y + panel_h + spacing::LG, interaction)
    }

    fn render_pressure_graph(&self, x: f32, y: f32, w: f32, h: f32) {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(x, y, w, h, Color::new(bg_r, bg_g, bg_b, 0.6));

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle_lines(
            x,
            y,
            w,
            h,
            1.0,
            Color::new(border_r, border_g, border_b, 0.2),
        );

        let history = &self.pressure_history;
        if history.len() < 2 {
            return;
        }

        let (curve_r, curve_g, curve_b) = colors::to_normalized(colors::SECONDARY);
        let step = w / (history.len() - 1) as f32;

        let points: Vec<(f32, f32)> = history
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let px = x + i as f32 * step;
                let py = y + h - v * h;
                (px, py.clamp(y, y + h))
            })
            .collect();

        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::SECONDARY);
        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            let base_y = y + h;
            draw_triangle(
                Vec2::new(x1, y1),
                Vec2::new(x2, y2),
                Vec2::new(x1, base_y),
                Color::new(fill_r, fill_g, fill_b, 0.12),
            );
            draw_triangle(
                Vec2::new(x2, y2),
                Vec2::new(x2, base_y),
                Vec2::new(x1, base_y),
                Color::new(fill_r, fill_g, fill_b, 0.12),
            );
        }

        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            draw_line(
                x1,
                y1,
                x2,
                y2,
                2.0,
                Color::new(curve_r, curve_g, curve_b, 0.9),
            );
        }

        if let Some(&(_, last_py)) = points.last() {
            if self.active_pressure > 0.01 {
                draw_circle(
                    x + w,
                    last_py,
                    4.0,
                    Color::new(curve_r, curve_g, curve_b, 1.0),
                );

                let pct = format!("{:.0}%", self.active_pressure * 100.0);
                let pct_w = measure_text(&pct, ply_fonts::body_font(), 10, 1.0).width;
                ply_fonts::draw_body(
                    &pct,
                    x + w - pct_w - 4.0,
                    last_py - 8.0,
                    10.0,
                    Color::new(curve_r, curve_g, curve_b, 1.0),
                );
            }
        }

        let (axis_r, axis_g, axis_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "127",
            x - 24.0,
            y + 8.0,
            8.0,
            Color::new(axis_r, axis_g, axis_b, 0.4),
        );
        ply_fonts::draw_body(
            "0",
            x - 10.0,
            y + h - 4.0,
            8.0,
            Color::new(axis_r, axis_g, axis_b, 0.4),
        );
    }

    fn render_hslider(
        &self,
        x: f32,
        y: f32,
        width: f32,
        label: &str,
        value: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> (f32, Option<f32>) {
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        let pct = format!("{}%", (value * 100.0).round() as u8);
        ply_fonts::draw_body(
            &format!("{}: {}", label, pct),
            x,
            y,
            13.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let track_y = y + 20.0;
        let track_h = 6.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            x,
            track_y,
            width,
            track_h,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let fill_w = value * width;
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::SECONDARY);
        draw_rectangle(
            x,
            track_y,
            fill_w,
            track_h,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let thumb_x = x + fill_w;
        let (thumb_r, thumb_g, thumb_b) = colors::to_normalized(colors::SECONDARY);
        draw_circle(
            thumb_x,
            track_y + track_h / 2.0,
            8.0,
            Color::new(thumb_r, thumb_g, thumb_b, 1.0),
        );

        let is_hovered = mx >= x - 8.0
            && mx <= x + width + 8.0
            && my >= track_y - 12.0
            && my <= track_y + track_h + 12.0;

        if (is_hovered && mouse_pressed) || (is_hovered && mouse_down) {
            let new_val = ((mx - x) / width).clamp(0.0, 1.0);
            (new_val, Some(new_val))
        } else {
            (value, None)
        }
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
        ply_fonts::draw_body(
            label,
            x + spacing::MD,
            y + 22.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            value,
            x + spacing::MD,
            y + 42.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        ply_fonts::draw_body(
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
        ply_fonts::draw_body(
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
        let my = my + self.scroll_offset;
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

        let (next_y, interaction) = self.render_velocity_section(
            content_x,
            current_y,
            content_width,
            config,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }
        current_y = next_y;

        let (_, interaction) =
            self.render_pedal_section(content_x, current_y, content_width, mx, my, mouse_pressed);
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }

        SettingsInteraction::None
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
