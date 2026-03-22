use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::GlassPanel;
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

pub struct AudioPage {
    scroll_offset: f32,
    current_soundfont_index: usize,
    soundfont_names: Vec<String>,
    main_volume: f32,
    metronome_volume: f32,
    midi_gain: f32,
    synth_volume: f32,
    pressed_keys: Vec<bool>,
}

impl AudioPage {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            current_soundfont_index: 0,
            soundfont_names: Vec::new(),
            main_volume: 0.75,
            metronome_volume: 0.5,
            midi_gain: 0.8,
            synth_volume: 0.7,
            pressed_keys: vec![false; 12],
        }
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Audio",
            x,
            y + 32.0,
            32.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (desc_r, desc_g, desc_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Configure audio engine, soundfonts, and mixer settings",
            x,
            y + 56.0,
            16.0,
            Color::new(desc_r, desc_g, desc_b, 1.0),
        );
        y + 80.0
    }

    fn render_engine_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 140.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Audio Engine",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let row_width = width - spacing::XL * 2.0;

        // Buffer size dropdown
        let buffer_hovered = self.render_dropdown_row(
            x + spacing::XL,
            y + 60.0,
            row_width,
            56.0,
            "Buffer Size",
            "512 samples",
            mx,
            my,
        );

        let interaction = SettingsInteraction::None;

        (y + 140.0 + spacing::LG, interaction)
    }

    fn render_soundfont_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let current_name = if self.soundfont_names.is_empty() {
            "No SoundFont loaded".to_string()
        } else {
            self.soundfont_names
                .get(self.current_soundfont_index)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string())
        };

        let panel = GlassPanel::new(x, y, width, 260.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "SoundFont Manager",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &format!("{} SoundFonts available", self.soundfont_names.len()),
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let item_y = y + 70.0;
        let item_width = width - spacing::XL * 2.0;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
        draw_rectangle(
            x + spacing::XL,
            item_y,
            item_width,
            80.0,
            Color::new(bg_r, bg_g, bg_b, 0.6),
        );

        let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::SECONDARY);
        draw_text(
            "🎵",
            x + spacing::XL + spacing::MD,
            item_y + 44.0,
            28.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &current_name,
            x + spacing::XL + 50.0,
            item_y + 34.0,
            16.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        let index_text = if self.soundfont_names.is_empty() {
            "0 / 0".to_string()
        } else {
            format!(
                "{} / {}",
                self.current_soundfont_index + 1,
                self.soundfont_names.len()
            )
        };
        draw_text(
            &index_text,
            x + spacing::XL + 50.0,
            item_y + 54.0,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let arrow_size = 32.0;
        let left_arrow_x = x + width - spacing::XL - arrow_size * 2.0 - spacing::SM;
        let right_arrow_x = x + width - spacing::XL - arrow_size;
        let arrow_y = item_y + 24.0;

        let left_hovered = mx >= left_arrow_x
            && mx <= left_arrow_x + arrow_size
            && my >= arrow_y
            && my <= arrow_y + arrow_size;
        let right_hovered = mx >= right_arrow_x
            && mx <= right_arrow_x + arrow_size
            && my >= arrow_y
            && my <= arrow_y + arrow_size;

        draw_rectangle(
            left_arrow_x,
            arrow_y,
            arrow_size,
            arrow_size,
            Color::new(bg_r, bg_g, bg_b, if left_hovered { 1.0 } else { 0.6 }),
        );
        draw_text(
            "◀",
            left_arrow_x + 10.0,
            arrow_y + 22.0,
            16.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        draw_rectangle(
            right_arrow_x,
            arrow_y,
            arrow_size,
            arrow_size,
            Color::new(bg_r, bg_g, bg_b, if right_hovered { 1.0 } else { 0.6 }),
        );
        draw_text(
            "▶",
            right_arrow_x + 10.0,
            arrow_y + 22.0,
            16.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        let mut interaction = SettingsInteraction::None;

        if left_hovered && mouse_pressed && self.current_soundfont_index > 0 {
            self.current_soundfont_index -= 1;
            interaction = SettingsInteraction::AudioGainChanged(self.midi_gain);
        }
        if right_hovered
            && mouse_pressed
            && self.current_soundfont_index < self.soundfont_names.len().saturating_sub(1)
        {
            self.current_soundfont_index += 1;
            interaction = SettingsInteraction::AudioGainChanged(self.midi_gain);
        }

        let btn_y = item_y + 90.0;
        let btn_w = 120.0;
        let btn_h = 36.0;

        let add_btn_x = x + spacing::XL;
        let add_hovered =
            mx >= add_btn_x && mx <= add_btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (add_r, add_g, add_b) = colors::to_normalized(if add_hovered {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE_VARIANT
        });
        draw_rectangle_lines(
            add_btn_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(add_r, add_g, add_b, 0.4),
        );
        draw_text(
            "+ Add Folder",
            add_btn_x + spacing::SM,
            btn_y + 24.0,
            14.0,
            Color::new(add_r, add_g, add_b, 1.0),
        );

        if add_hovered && mouse_pressed {
            interaction = SettingsInteraction::AddSoundFontFolder;
        }

        let kb_x = x + spacing::XL + btn_w + spacing::LG;
        let kb_y = btn_y;
        let kb_w = width - spacing::XL * 2.0 - btn_w - spacing::LG;
        let kb_h = 36.0;
        let white_key_count = 7;
        let white_key_w = kb_w / white_key_count as f32;
        let black_key_pattern = [true, true, false, true, true, true, false];

        draw_rectangle(kb_x, kb_y, kb_w, kb_h, Color::new(bg_r, bg_g, bg_b, 0.4));

        for i in 0..white_key_count {
            let kx = kb_x + i as f32 * white_key_w;
            let is_hovered =
                mx >= kx && mx <= kx + white_key_w - 1.0 && my >= kb_y && my <= kb_y + kb_h;
            let is_pressed = self.pressed_keys.get(i * 2).copied().unwrap_or(false);

            let key_color = if is_pressed {
                colors::SECONDARY
            } else if is_hovered {
                colors::SURFACE_CONTAINER_HIGHEST
            } else {
                colors::ON_SURFACE
            };
            let (kr, kg, kb) = colors::to_normalized(key_color);

            draw_rectangle(
                kx + 0.5,
                kb_y + 0.5,
                white_key_w - 1.0,
                kb_h - 1.0,
                Color::new(kr, kg, kb, if is_pressed { 0.6 } else { 0.2 }),
            );

            if is_hovered && mouse_pressed {
                if let Some(key) = self.pressed_keys.get_mut(i * 2) {
                    *key = true;
                }
            }
        }

        let note_names = ["C", "D", "E", "F", "G", "A", "B"];
        for i in 0..white_key_count {
            let kx = kb_x + i as f32 * white_key_w;
            let (nr, ng, nb) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_text(
                note_names[i],
                kx + white_key_w / 2.0 - 3.0,
                kb_y + kb_h - 8.0,
                10.0,
                Color::new(nr, ng, nb, 0.6),
            );
        }

        let folder_y = kb_y + kb_h + spacing::MD;
        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "SoundFont Folders:",
            x + spacing::XL,
            folder_y,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let folders = config.synth_config.soundfont_folders();
        let mut folder_item_y = folder_y + 16.0;

        if folders.is_empty() {
            draw_text(
                "No folders configured",
                x + spacing::XL + 8.0,
                folder_item_y + 12.0,
                11.0,
                Color::new(label_r, label_g, label_b, 0.5),
            );
            folder_item_y += 24.0;
        }

        for (idx, folder) in folders.iter().enumerate() {
            let folder_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= folder_item_y
                && my <= folder_item_y + 28.0;

            let (bg_folder_r, bg_folder_g, bg_folder_b) =
                colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            draw_rectangle(
                x + spacing::XL,
                folder_item_y,
                width - spacing::XL * 2.0,
                28.0,
                Color::new(
                    bg_folder_r,
                    bg_folder_g,
                    bg_folder_b,
                    if folder_hovered { 0.6 } else { 0.3 },
                ),
            );

            let folder_str = folder.to_string_lossy();
            let max_chars = ((width - spacing::XL * 2.0 - 40.0) / 7.0) as usize;
            let display = if folder_str.len() > max_chars {
                format!("...{}", &folder_str[folder_str.len() - max_chars + 3..])
            } else {
                folder_str.to_string()
            };

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::SECONDARY);
            draw_text(
                "📁",
                x + spacing::XL + 4.0,
                folder_item_y + 19.0,
                12.0,
                Color::new(icon_r, icon_g, icon_b, 0.8),
            );

            draw_text(
                &display,
                x + spacing::XL + 22.0,
                folder_item_y + 18.0,
                11.0,
                Color::new(
                    text_r,
                    text_g,
                    text_b,
                    if folder_hovered { 0.9 } else { 0.6 },
                ),
            );

            if folder_hovered {
                let del_x = x + width - spacing::XL - 20.0;
                let del_hovered = mx >= del_x
                    && mx <= del_x + 16.0
                    && my >= folder_item_y + 4.0
                    && my <= folder_item_y + 24.0;
                let (del_r, del_g, del_b) = colors::to_normalized(colors::ERROR);
                draw_text(
                    "×",
                    del_x,
                    folder_item_y + 18.0,
                    14.0,
                    Color::new(del_r, del_g, del_b, if del_hovered { 1.0 } else { 0.5 }),
                );

                if del_hovered && mouse_pressed {
                    interaction = SettingsInteraction::RemoveSoundFontFolder(idx);
                }
            }

            folder_item_y += 32.0;
        }

        (
            y + 260.0 + spacing::LG + folder_item_y - folder_y,
            interaction,
        )
    }

    fn render_mixer_section(
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
        let panel = GlassPanel::new(x, y, width, 320.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Obsidian Mixer",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Real-time audio control",
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let slider_x = x + spacing::XL;
        let slider_w = width - spacing::XL * 2.0;
        let slider_spacing = 75.0;

        let main_vol_changed = self.render_vertical_slider(
            slider_x,
            y + 80.0,
            60.0,
            140.0,
            "Main",
            self.main_volume,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        if let Some(val) = main_vol_changed {
            self.main_volume = val;
        }

        let metronome_changed = self.render_vertical_slider(
            slider_x + slider_spacing,
            y + 80.0,
            60.0,
            140.0,
            "Metro",
            self.metronome_volume,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        if let Some(val) = metronome_changed {
            self.metronome_volume = val;
        }

        let midi_gain_changed = self.render_vertical_slider(
            slider_x + slider_spacing * 2.0,
            y + 80.0,
            60.0,
            140.0,
            "MIDI",
            self.midi_gain,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        if let Some(val) = midi_gain_changed {
            self.midi_gain = val;
        }

        let synth_changed = self.render_vertical_slider(
            slider_x + slider_spacing * 3.0,
            y + 80.0,
            60.0,
            140.0,
            "Synth",
            self.synth_volume,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        if let Some(val) = synth_changed {
            self.synth_volume = val;
        }

        let viz_y = y + 230.0;
        let viz_w = slider_w;
        let viz_h = 60.0;
        let (viz_r, viz_g, viz_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            slider_x,
            viz_y,
            viz_w,
            viz_h,
            Color::new(viz_r, viz_g, viz_b, 1.0),
        );

        let channel_count = 4;
        let channel_w = (viz_w - (channel_count - 1) as f32 * spacing::SM) / channel_count as f32;
        let channel_values = [
            self.main_volume,
            self.metronome_volume,
            self.midi_gain,
            self.synth_volume,
        ];
        let channel_colors = [
            colors::to_normalized(colors::PRIMARY),
            colors::to_normalized(colors::SECONDARY),
            colors::to_normalized(colors::TERTIARY),
            colors::to_normalized((100, 200, 100)),
        ];
        let channel_names = ["Main", "Metro", "MIDI", "Synth"];

        for (i, (value, (cr, cg, cb))) in
            channel_values.iter().zip(channel_colors.iter()).enumerate()
        {
            let ch_x = slider_x + i as f32 * (channel_w + spacing::SM);
            let bar_count = 8;
            let bar_w = (channel_w - (bar_count - 1) as f32 * 2.0) / bar_count as f32;

            for j in 0..bar_count {
                let intensity =
                    value * (0.5 + 0.5 * ((j as f32 * 0.8 + i as f32 * 1.2).sin() * 0.5 + 0.5));
                let bar_h = viz_h * intensity * 0.9;
                let bar_x = ch_x + j as f32 * (bar_w + 2.0);
                let bar_y = viz_y + viz_h - bar_h;

                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(*cr, *cg, *cb, 0.8));
            }

            let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_text(
                channel_names[i],
                ch_x + 2.0,
                viz_y + viz_h + 14.0,
                10.0,
                Color::new(label_r, label_g, label_b, 1.0),
            );
        }

        let interaction = if synth_changed.is_some() {
            SettingsInteraction::AudioGainChanged(self.synth_volume)
        } else if midi_gain_changed.is_some() {
            SettingsInteraction::AudioGainChanged(self.midi_gain)
        } else {
            SettingsInteraction::None
        };

        (y + 320.0 + spacing::LG, interaction)
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

    fn render_vertical_slider(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        label: &str,
        value: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> Option<f32> {
        let track_w = 8.0;
        let track_x = x + (width - track_w) / 2.0;
        let track_y = y + 20.0;
        let track_h = height - 40.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            track_x,
            track_y,
            track_w,
            track_h,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let fill_h = track_h * value.clamp(0.0, 1.0);
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::SECONDARY);
        draw_rectangle(
            track_x,
            track_y + track_h - fill_h,
            track_w,
            fill_h,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let thumb_radius = 10.0;
        let thumb_y = track_y + track_h - fill_h;
        let (thumb_r, thumb_g, thumb_b) = colors::to_normalized(colors::SECONDARY);
        draw_circle(
            track_x + track_w / 2.0,
            thumb_y,
            thumb_radius,
            Color::new(thumb_r, thumb_g, thumb_b, 1.0),
        );

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            label,
            x,
            y + height - 4.0,
            11.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let pct = format!("{}%", (value * 100.0).round());
        let pct_w = measure_text(&pct, None, 11, 1.0).width;
        draw_text(
            &pct,
            x + (width - pct_w) / 2.0,
            y + 12.0,
            11.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let is_hovered = mx >= track_x - thumb_radius
            && mx <= track_x + track_w + thumb_radius
            && my >= track_y - thumb_radius
            && my <= track_y + track_h + thumb_radius;

        if (is_hovered && mouse_pressed) || (is_hovered && mouse_down) {
            let relative_y = my - track_y;
            let new_value = 1.0 - (relative_y / track_h).clamp(0.0, 1.0);
            Some(new_value)
        } else {
            None
        }
    }
}

impl SettingsPage for AudioPage {
    fn title(&self) -> &str {
        "Audio"
    }

    fn description(&self) -> &str {
        "Configure audio engine, soundfonts, and mixer settings"
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

        let (next_y, interaction) = self.render_engine_section(
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

        let (next_y, interaction) = self.render_soundfont_section(
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

        let (_, interaction) = self.render_mixer_section(
            content_x,
            current_y,
            content_width,
            config,
            mx,
            my,
            mouse_pressed,
            mouse_down,
        );
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
