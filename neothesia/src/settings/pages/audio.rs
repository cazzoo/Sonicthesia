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
    soundfont_paths: Vec<std::path::PathBuf>,
    needs_refresh: bool,
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
            soundfont_paths: Vec::new(),
            needs_refresh: true,
            main_volume: 0.75,
            metronome_volume: 0.5,
            midi_gain: 0.8,
            synth_volume: 0.7,
            pressed_keys: vec![false; 12],
        }
    }

    pub fn refresh_soundfonts(&mut self, config: &Config) {
        let folders = config.synth_config.soundfont_folders();
        let soundfonts = crate::output_manager::discover_soundfonts(folders);

        self.soundfont_names.clear();
        self.soundfont_paths.clear();

        for sf in &soundfonts {
            let name = sf
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            self.soundfont_names.push(name);
            self.soundfont_paths.push(sf.path.clone());
        }

        self.current_soundfont_index = config.synth_config.soundfont_index().unwrap_or(0);
        if self.current_soundfont_index >= self.soundfont_names.len()
            && !self.soundfont_names.is_empty()
        {
            self.current_soundfont_index = 0;
        }
        self.needs_refresh = false;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
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

        let section_height = 280.0;
        let panel = GlassPanel::new(x, y, width, section_height);
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

        let selector_y = y + 60.0;
        let selector_width = width - spacing::XL * 2.0;
        let selector_height = 60.0;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
        draw_rectangle(
            x + spacing::XL,
            selector_y,
            selector_width,
            selector_height,
            Color::new(bg_r, bg_g, bg_b, 0.7),
        );

        let border_color = if !self.soundfont_names.is_empty() {
            colors::to_normalized(colors::SECONDARY)
        } else {
            colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST)
        };
        draw_rectangle_lines(
            x + spacing::XL,
            selector_y,
            selector_width,
            selector_height,
            1.0,
            Color::new(border_color.0, border_color.1, border_color.2, 0.5),
        );

        let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::SECONDARY);
        draw_text(
            "🎵",
            x + spacing::XL + spacing::MD,
            selector_y + 40.0,
            32.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            &current_name,
            x + spacing::XL + 56.0,
            selector_y + 30.0,
            18.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        let index_text = if self.soundfont_names.is_empty() {
            "Add a folder to get started".to_string()
        } else {
            format!(
                "SoundFont {} of {}",
                self.current_soundfont_index + 1,
                self.soundfont_names.len()
            )
        };
        draw_text(
            &index_text,
            x + spacing::XL + 56.0,
            selector_y + 50.0,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let arrow_btn_size = 40.0;
        let arrow_y = selector_y + (selector_height - arrow_btn_size) / 2.0;
        let left_arrow_x = x + width - spacing::XL - arrow_btn_size * 2.0 - spacing::SM;
        let right_arrow_x = x + width - spacing::XL - arrow_btn_size;

        let left_disabled = self.soundfont_names.is_empty() || self.current_soundfont_index == 0;
        let right_disabled = self.soundfont_names.is_empty()
            || self.current_soundfont_index >= self.soundfont_names.len().saturating_sub(1);

        let left_hovered = !left_disabled
            && mx >= left_arrow_x
            && mx <= left_arrow_x + arrow_btn_size
            && my >= arrow_y
            && my <= arrow_y + arrow_btn_size;
        let right_hovered = !right_disabled
            && mx >= right_arrow_x
            && mx <= right_arrow_x + arrow_btn_size
            && my >= arrow_y
            && my <= arrow_y + arrow_btn_size;

        let left_bg = if left_disabled {
            colors::SURFACE_CONTAINER_LOW
        } else if left_hovered {
            colors::PRIMARY
        } else {
            colors::SURFACE_CONTAINER_HIGHEST
        };
        let (lbg_r, lbg_g, lbg_b) = colors::to_normalized(left_bg);
        draw_rectangle(
            left_arrow_x,
            arrow_y,
            arrow_btn_size,
            arrow_btn_size,
            Color::new(lbg_r, lbg_g, lbg_b, if left_disabled { 0.3 } else { 0.8 }),
        );
        let left_text_color = if left_disabled {
            colors::ON_SURFACE_VARIANT
        } else if left_hovered {
            colors::BLACK
        } else {
            colors::ON_SURFACE
        };
        let (lt_r, lt_g, lt_b) = colors::to_normalized(left_text_color);
        draw_text(
            "◀",
            left_arrow_x + 14.0,
            arrow_y + 27.0,
            18.0,
            Color::new(lt_r, lt_g, lt_b, if left_disabled { 0.3 } else { 1.0 }),
        );

        let right_bg = if right_disabled {
            colors::SURFACE_CONTAINER_LOW
        } else if right_hovered {
            colors::PRIMARY
        } else {
            colors::SURFACE_CONTAINER_HIGHEST
        };
        let (rbg_r, rbg_g, rbg_b) = colors::to_normalized(right_bg);
        draw_rectangle(
            right_arrow_x,
            arrow_y,
            arrow_btn_size,
            arrow_btn_size,
            Color::new(rbg_r, rbg_g, rbg_b, if right_disabled { 0.3 } else { 0.8 }),
        );
        let right_text_color = if right_disabled {
            colors::ON_SURFACE_VARIANT
        } else if right_hovered {
            colors::BLACK
        } else {
            colors::ON_SURFACE
        };
        let (rt_r, rt_g, rt_b) = colors::to_normalized(right_text_color);
        draw_text(
            "▶",
            right_arrow_x + 14.0,
            arrow_y + 27.0,
            18.0,
            Color::new(rt_r, rt_g, rt_b, if right_disabled { 0.3 } else { 1.0 }),
        );

        let mut interaction = SettingsInteraction::None;

        if left_hovered && mouse_pressed {
            self.current_soundfont_index -= 1;
            interaction = SettingsInteraction::SoundFontSelected(self.current_soundfont_index);
        }
        if right_hovered && mouse_pressed {
            self.current_soundfont_index += 1;
            interaction = SettingsInteraction::SoundFontSelected(self.current_soundfont_index);
        }

        let kb_label_y = selector_y + selector_height + spacing::MD;
        draw_text(
            "Test SoundFont",
            x + spacing::XL,
            kb_label_y,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let kb_x = x + spacing::XL;
        let kb_y = kb_label_y + 4.0;
        let kb_w = width - spacing::XL * 2.0;
        let kb_h = 50.0;

        let (kb_bg_r, kb_bg_g, kb_bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            kb_x,
            kb_y,
            kb_w,
            kb_h,
            Color::new(kb_bg_r, kb_bg_g, kb_bg_b, 0.8),
        );

        let white_key_count = 7;
        let white_key_w = kb_w / white_key_count as f32;
        let black_key_positions = [0, 1, 3, 4, 5];
        let black_key_w = white_key_w * 0.6;
        let black_key_h = kb_h * 0.6;

        for i in 0..white_key_count {
            let kx = kb_x + i as f32 * white_key_w;
            let is_hovered =
                mx >= kx && mx <= kx + white_key_w - 1.0 && my >= kb_y && my <= kb_y + kb_h;

            let key_white = colors::to_normalized(colors::ON_SURFACE);
            let key_bg = colors::to_normalized(colors::SURFACE_CONTAINER);

            draw_rectangle(
                kx + 1.0,
                kb_y + 1.0,
                white_key_w - 2.0,
                kb_h - 2.0,
                Color::new(key_bg.0, key_bg.1, key_bg.2, 0.3),
            );

            if is_hovered {
                let (h_r, h_g, h_b) = colors::to_normalized(colors::SECONDARY);
                draw_rectangle(
                    kx + 1.0,
                    kb_y + 1.0,
                    white_key_w - 2.0,
                    kb_h - 2.0,
                    Color::new(h_r, h_g, h_b, 0.3),
                );
            }

            draw_rectangle_lines(
                kx + 1.0,
                kb_y + 1.0,
                white_key_w - 2.0,
                kb_h - 2.0,
                1.0,
                Color::new(key_white.0, key_white.1, key_white.2, 0.2),
            );

            let note_names = ["C", "D", "E", "F", "G", "A", "B"];
            let (nr, ng, nb) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_text(
                note_names[i],
                kx + white_key_w / 2.0 - 4.0,
                kb_y + kb_h - 12.0,
                12.0,
                Color::new(nr, ng, nb, 0.5),
            );

            if is_hovered && mouse_pressed {
                interaction = SettingsInteraction::SoundFontSelected(self.current_soundfont_index);
            }
        }

        for &pos in &black_key_positions {
            let kx = kb_x + pos as f32 * white_key_w + white_key_w - black_key_w / 2.0;
            let is_hovered =
                mx >= kx && mx <= kx + black_key_w && my >= kb_y && my <= kb_y + black_key_h;

            let (bk_r, bk_g, bk_b) = if is_hovered {
                colors::to_normalized(colors::SECONDARY)
            } else {
                colors::to_normalized(colors::SURFACE_CONTAINER_LOWEST)
            };

            draw_rectangle(
                kx,
                kb_y,
                black_key_w,
                black_key_h,
                Color::new(bk_r, bk_g, bk_b, if is_hovered { 0.9 } else { 0.95 }),
            );

            let (border_r, border_g, border_b) =
                colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            draw_rectangle_lines(
                kx,
                kb_y,
                black_key_w,
                black_key_h,
                1.0,
                Color::new(border_r, border_g, border_b, 0.3),
            );

            if is_hovered && mouse_pressed {
                interaction = SettingsInteraction::SoundFontSelected(self.current_soundfont_index);
            }
        }

        let add_btn_y = kb_y + kb_h + spacing::LG;
        let add_btn_w = 140.0;
        let add_btn_h = 36.0;
        let add_btn_x = x + spacing::XL;

        let add_hovered = mx >= add_btn_x
            && mx <= add_btn_x + add_btn_w
            && my >= add_btn_y
            && my <= add_btn_y + add_btn_h;

        let (add_bg_r, add_bg_g, add_bg_b) = if add_hovered {
            colors::to_normalized(colors::PRIMARY)
        } else {
            colors::to_normalized(colors::SURFACE_CONTAINER_HIGH)
        };
        draw_rectangle(
            add_btn_x,
            add_btn_y,
            add_btn_w,
            add_btn_h,
            Color::new(
                add_bg_r,
                add_bg_g,
                add_bg_b,
                if add_hovered { 0.8 } else { 0.5 },
            ),
        );
        draw_rectangle_lines(
            add_btn_x,
            add_btn_y,
            add_btn_w,
            add_btn_h,
            1.0,
            Color::new(add_bg_r, add_bg_g, add_bg_b, 0.6),
        );

        let (add_text_r, add_text_g, add_text_b) = if add_hovered {
            colors::to_normalized(colors::BLACK)
        } else {
            colors::to_normalized(colors::ON_SURFACE)
        };
        draw_text(
            "+ Add Folder",
            add_btn_x + spacing::MD,
            add_btn_y + 24.0,
            14.0,
            Color::new(add_text_r, add_text_g, add_text_b, 1.0),
        );

        if add_hovered && mouse_pressed {
            interaction = SettingsInteraction::AddSoundFontFolder;
        }

        (y + section_height + spacing::LG, interaction)
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

    fn render_folders_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let folders = config.synth_config.soundfont_folders();
        let section_height = 60.0 + folders.len() as f32 * 36.0 + 20.0;

        let panel = GlassPanel::new(x, y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "SoundFont Folders",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &format!("{} folder(s) configured", folders.len()),
            x + spacing::XL,
            y + spacing::XL + 34.0,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let mut item_y = y + 60.0;
        let mut interaction = SettingsInteraction::None;
        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);

        if folders.is_empty() {
            draw_text(
                "No folders configured. Add a folder to scan for .sf2 files.",
                x + spacing::XL,
                item_y + 16.0,
                12.0,
                Color::new(sub_r, sub_g, sub_b, 0.6),
            );
        }

        for (idx, folder) in folders.iter().enumerate() {
            let item_h = 32.0;
            let folder_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= item_y
                && my <= item_y + item_h;

            let (item_bg_r, item_bg_g, item_bg_b) =
                colors::to_normalized(colors::SURFACE_CONTAINER);
            draw_rectangle(
                x + spacing::XL,
                item_y,
                width - spacing::XL * 2.0,
                item_h,
                Color::new(
                    item_bg_r,
                    item_bg_g,
                    item_bg_b,
                    if folder_hovered { 0.6 } else { 0.3 },
                ),
            );

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::SECONDARY);
            draw_text(
                "📁",
                x + spacing::XL + spacing::SM,
                item_y + 21.0,
                14.0,
                Color::new(icon_r, icon_g, icon_b, 0.9),
            );

            let folder_str = folder.to_string_lossy();
            let max_chars = ((width - spacing::XL * 2.0 - 60.0) / 7.0) as usize;
            let display = if folder_str.len() > max_chars {
                format!("...{}", &folder_str[folder_str.len() - max_chars + 3..])
            } else {
                folder_str.to_string()
            };

            draw_text(
                &display,
                x + spacing::XL + 28.0,
                item_y + 20.0,
                11.0,
                Color::new(
                    text_r,
                    text_g,
                    text_b,
                    if folder_hovered { 0.9 } else { 0.7 },
                ),
            );

            if folder_hovered {
                let del_x = x + width - spacing::XL - 24.0;
                let del_hovered =
                    mx >= del_x && mx <= del_x + 20.0 && my >= item_y + 6.0 && my <= item_y + 26.0;
                let (del_r, del_g, del_b) = colors::to_normalized(colors::ERROR);
                draw_text(
                    "×",
                    del_x,
                    item_y + 21.0,
                    16.0,
                    Color::new(del_r, del_g, del_b, if del_hovered { 1.0 } else { 0.5 }),
                );

                if del_hovered && mouse_pressed {
                    interaction = SettingsInteraction::RemoveSoundFontFolder(idx);
                }
            }

            item_y += item_h + 4.0;
        }

        (y + section_height + spacing::LG, interaction)
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
        if self.needs_refresh {
            self.refresh_soundfonts(config);
        }

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

        let (next_y, interaction) = self.render_mixer_section(
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

        let (_, interaction) = self.render_folders_section(
            content_x,
            current_y,
            content_width,
            config,
            mx,
            my,
            mouse_pressed,
        );
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
