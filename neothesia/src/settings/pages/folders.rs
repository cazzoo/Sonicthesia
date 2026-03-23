use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::{GlassPanel, StorageIndicator};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

pub struct FoldersPage {
    scroll_offset: f32,
}

impl FoldersPage {
    pub fn new() -> Self {
        Self { scroll_offset: 0.0 }
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Library Management",
            x,
            y + 28.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let line_x = x + 200.0;
        let line_w = width - 200.0;
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

    fn render_midi_directories_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let directories = config.song_directories();
        let section_height = 80.0 + directories.len() as f32 * 72.0 + 60.0;

        let panel = GlassPanel::new(x, y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "MIDI Song Directories",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &format!("{} directories configured", directories.len()),
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let mut item_y = y + 70.0;
        let mut remove_index = None;

        for (idx, dir) in directories.iter().enumerate() {
            let is_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= item_y
                && my <= item_y + 64.0;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            draw_rectangle(
                x + spacing::XL,
                item_y,
                width - spacing::XL * 2.0,
                64.0,
                Color::new(bg_r, bg_g, bg_b, if is_hovered { 0.8 } else { 0.5 }),
            );

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::PRIMARY);
            draw_text(
                "📁",
                x + spacing::XL + spacing::MD,
                item_y + 36.0,
                22.0,
                Color::new(icon_r, icon_g, icon_b, 1.0),
            );

            let path_str = dir.to_string_lossy();
            let max_chars = 50;
            let display_path = if path_str.len() > max_chars {
                format!("...{}", &path_str[path_str.len() - max_chars..])
            } else {
                path_str.to_string()
            };

            let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
            draw_text(
                &display_path,
                x + spacing::XL + 40.0,
                item_y + 28.0,
                13.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            let (meta_r, meta_g, meta_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_text(
                "Default source for recordings and imports",
                x + spacing::XL + 40.0,
                item_y + 46.0,
                12.0,
                Color::new(meta_r, meta_g, meta_b, 1.0),
            );

            if is_hovered {
                let del_x = x + width - spacing::XL - 32.0;
                let del_y = item_y + 20.0;
                let is_del_hovered =
                    mx >= del_x && mx <= del_x + 24.0 && my >= del_y && my <= del_y + 24.0;

                let (del_r, del_g, del_b) = colors::to_normalized(colors::ERROR);
                draw_text(
                    "×",
                    del_x,
                    del_y + 18.0,
                    20.0,
                    Color::new(del_r, del_g, del_b, if is_del_hovered { 1.0 } else { 0.6 }),
                );

                if is_del_hovered && mouse_pressed {
                    remove_index = Some(idx);
                }
            }

            item_y += 72.0;
        }

        // Add MIDI folder button
        let btn_x = x + spacing::XL;
        let btn_y = item_y + spacing::SM;
        let btn_w = 180.0;
        let btn_h = 40.0;

        let is_btn_hovered =
            mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (btn_r, btn_g, btn_b) = colors::to_normalized(if is_btn_hovered {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE_VARIANT
        });
        draw_rectangle_lines(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(btn_r, btn_g, btn_b, 0.3),
        );
        draw_text(
            "+ Add MIDI Folder",
            btn_x + spacing::MD,
            btn_y + 26.0,
            14.0,
            Color::new(btn_r, btn_g, btn_b, 1.0),
        );

        let interaction = if let Some(idx) = remove_index {
            SettingsInteraction::RemoveSongDirectory(idx)
        } else if is_btn_hovered && mouse_pressed {
            SettingsInteraction::AddSongDirectory
        } else {
            SettingsInteraction::None
        };

        (y + section_height + spacing::LG, interaction)
    }

    fn render_soundfont_directories_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let soundfont_folders = config.synth_config.soundfont_folders();
        let section_height = 80.0 + soundfont_folders.len() as f32 * 72.0 + 60.0;

        let panel = GlassPanel::new(x, y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "SoundFont Libraries",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            &format!("{} folders configured", soundfont_folders.len()),
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let mut item_y = y + 70.0;
        let mut remove_index = None;

        for (idx, folder) in soundfont_folders.iter().enumerate() {
            let is_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= item_y
                && my <= item_y + 64.0;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            draw_rectangle(
                x + spacing::XL,
                item_y,
                width - spacing::XL * 2.0,
                64.0,
                Color::new(bg_r, bg_g, bg_b, if is_hovered { 0.8 } else { 0.5 }),
            );

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::SECONDARY);
            draw_text(
                "🎵",
                x + spacing::XL + spacing::MD,
                item_y + 36.0,
                22.0,
                Color::new(icon_r, icon_g, icon_b, 1.0),
            );

            let path_str = folder.to_string_lossy();
            let max_chars = 50;
            let display_path = if path_str.len() > max_chars {
                format!("...{}", &path_str[path_str.len() - max_chars..])
            } else {
                path_str.to_string()
            };

            let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
            draw_text(
                &display_path,
                x + spacing::XL + 40.0,
                item_y + 28.0,
                13.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            let (meta_r, meta_g, meta_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            draw_text(
                "Global SoundFont directory (.sf2)",
                x + spacing::XL + 40.0,
                item_y + 46.0,
                12.0,
                Color::new(meta_r, meta_g, meta_b, 1.0),
            );

            if is_hovered {
                let del_x = x + width - spacing::XL - 32.0;
                let del_y = item_y + 20.0;
                let is_del_hovered =
                    mx >= del_x && mx <= del_x + 24.0 && my >= del_y && my <= del_y + 24.0;

                let (del_r, del_g, del_b) = colors::to_normalized(colors::ERROR);
                draw_text(
                    "×",
                    del_x,
                    del_y + 18.0,
                    20.0,
                    Color::new(del_r, del_g, del_b, if is_del_hovered { 1.0 } else { 0.6 }),
                );

                if is_del_hovered && mouse_pressed {
                    remove_index = Some(idx);
                }
            }

            item_y += 72.0;
        }

        // Add soundfont pack button
        let btn_x = x + spacing::XL;
        let btn_y = item_y + spacing::SM;
        let btn_w = 200.0;
        let btn_h = 40.0;

        let is_btn_hovered =
            mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (btn_r, btn_g, btn_b) = colors::to_normalized(if is_btn_hovered {
            colors::SECONDARY
        } else {
            colors::ON_SURFACE_VARIANT
        });
        draw_rectangle_lines(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(btn_r, btn_g, btn_b, 0.3),
        );
        draw_text(
            "+ Import SoundFont Pack",
            btn_x + spacing::MD,
            btn_y + 26.0,
            14.0,
            Color::new(btn_r, btn_g, btn_b, 1.0),
        );

        let interaction = if let Some(idx) = remove_index {
            SettingsInteraction::RemoveSoundFontFolder(idx)
        } else if is_btn_hovered && mouse_pressed {
            SettingsInteraction::AddSoundFontFolder
        } else {
            SettingsInteraction::None
        };

        (y + section_height + spacing::LG, interaction)
    }

    fn render_global_operations_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 120.0);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Global Operations",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let btn_y = y + 60.0;
        let btn_w = 140.0;
        let btn_h = 36.0;

        let scan_x = x + spacing::XL;
        let is_scan_hovered =
            mx >= scan_x && mx <= scan_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (scan_r, scan_g, scan_b) = colors::to_normalized(colors::SECONDARY);
        draw_rectangle(
            scan_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(
                scan_r,
                scan_g,
                scan_b,
                if is_scan_hovered { 0.3 } else { 0.15 },
            ),
        );
        let scan_text_w = measure_text("Scan All", None, 13, 1.0).width;
        draw_text(
            "Scan All",
            scan_x + (btn_w - scan_text_w) / 2.0,
            btn_y + 24.0,
            13.0,
            Color::new(scan_r, scan_g, scan_b, 1.0),
        );

        let clear_x = scan_x + btn_w + spacing::LG;
        let is_clear_hovered =
            mx >= clear_x && mx <= clear_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (clear_r, clear_g, clear_b) = colors::to_normalized(colors::TERTIARY);
        draw_rectangle(
            clear_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(
                clear_r,
                clear_g,
                clear_b,
                if is_clear_hovered { 0.3 } else { 0.15 },
            ),
        );
        let clear_text_w = measure_text("Clear Cache", None, 13, 1.0).width;
        draw_text(
            "Clear Cache",
            clear_x + (btn_w - clear_text_w) / 2.0,
            btn_y + 24.0,
            13.0,
            Color::new(clear_r, clear_g, clear_b, 1.0),
        );

        let optimize_x = clear_x + btn_w + spacing::LG;
        let is_optimize_hovered =
            mx >= optimize_x && mx <= optimize_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (opt_r, opt_g, opt_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_rectangle_lines(
            optimize_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(opt_r, opt_g, opt_b, 0.3),
        );
        let opt_text_w = measure_text("Optimize", None, 13, 1.0).width;
        draw_text(
            "Optimize",
            optimize_x + (btn_w - opt_text_w) / 2.0,
            btn_y + 24.0,
            13.0,
            Color::new(opt_r, opt_g, opt_b, 1.0),
        );

        (y + 120.0 + spacing::LG, SettingsInteraction::None)
    }

    fn render_footer_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 100.0);
        panel.render();

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Last scan:",
            x + spacing::XL,
            y + 28.0,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "2 minutes ago",
            x + spacing::XL + 70.0,
            y + 28.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        draw_text(
            "Status:",
            x + spacing::XL,
            y + 48.0,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (status_r, status_g, status_b) = colors::to_normalized(colors::SECONDARY);
        draw_text(
            "● All synced",
            x + spacing::XL + 50.0,
            y + 48.0,
            12.0,
            Color::new(status_r, status_g, status_b, 1.0),
        );

        draw_text(
            "Files indexed:",
            x + spacing::XL,
            y + 68.0,
            12.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        draw_text(
            "1,247 MIDI files",
            x + spacing::XL + 90.0,
            y + 68.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        (y + 100.0 + spacing::LG, SettingsInteraction::None)
    }
}

impl SettingsPage for FoldersPage {
    fn title(&self) -> &str {
        "Library Management"
    }

    fn description(&self) -> &str {
        "Manage MIDI song directories and library settings"
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

        let (next_y, interaction) = self.render_midi_directories_section(
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

        let (next_y, interaction) = self.render_soundfont_directories_section(
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

        let (next_y, interaction) = self.render_global_operations_section(
            content_x,
            current_y,
            content_width,
            mx,
            my,
            mouse_pressed,
        );
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }
        current_y = next_y;

        let (_, interaction) =
            self.render_footer_section(content_x, current_y, content_width, mx, my);
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
