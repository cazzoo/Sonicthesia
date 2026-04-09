use crate::scene::ply_fonts;
use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::status_chip::ChipVariant;
use crate::ui::components::{
    GlassPanel, SectionHeader, SettingCard, StatusChip, StorageIndicator, ThemeCard,
};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing, themes};

pub struct GeneralPage {
    scroll_offset: f32,
    theme_cards: Vec<ThemeCard>,
    hovered_button: Option<String>,
}

impl GeneralPage {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            theme_cards: Vec::new(),
            hovered_button: None,
        }
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "General Settings",
            x,
            y + 28.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let line_x = x + 180.0;
        let line_w = width - 180.0;
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

    fn render_status_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let section_y = y;

        let header = SectionHeader::new(x, section_y, width, "Quick Settings");
        let content_y = header.render();

        let audio_gain_str = format!("{:.0}%", config.audio_gain() * 100.0);
        let glow_str = if config.glow() { "On" } else { "Off" };
        let note_labels_str = if config.note_labels() { "On" } else { "Off" };

        let items: Vec<(&str, &str, &str)> = vec![
            ("MIDI Input", config.input().unwrap_or("None"), "MIDI"),
            ("MIDI Output", config.output().unwrap_or("None"), "MIDI"),
            ("Theme", config.piano_theme_name(), "Themes"),
            ("Audio Gain", &audio_gain_str, "Audio"),
            ("Glow", &glow_str, "MIDI"),
            ("Note Labels", &note_labels_str, "MIDI"),
        ];

        let mut item_y = section_y + 70.0;
        let item_h = 32.0;
        let mut clicked_redirect = None;

        for (label, value, redirect) in items {
            let is_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= item_y
                && my <= item_y + item_h;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            draw_rectangle(
                x + spacing::XL,
                item_y,
                width - spacing::XL * 2.0,
                item_h,
                Color::new(bg_r, bg_g, bg_b, if is_hovered { 0.6 } else { 0.3 }),
            );

            let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
            ply_fonts::draw_body(
                label,
                x + spacing::LG,
                item_y + 21.0,
                13.0,
                Color::new(label_r, label_g, label_b, 1.0),
            );

            let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            ply_fonts::draw_body(
                value,
                x + width - spacing::XL - 100.0,
                item_y + 21.0,
                13.0,
                Color::new(value_r, value_g, value_b, 1.0),
            );

            if is_hovered {
                let arrow_x = x + width - spacing::XL - 20.0;
                ply_fonts::draw_body(
                    "›",
                    arrow_x,
                    item_y + 22.0,
                    16.0,
                    Color::new(value_r, value_g, value_b, 0.5),
                );

                if mouse_pressed {
                    clicked_redirect = Some(redirect);
                }
            }

            item_y += item_h + 4.0;
        }

        let interaction = match clicked_redirect {
            Some("MIDI") => SettingsInteraction::OpenPopup("goto_midi".to_string()),
            Some("Audio") => SettingsInteraction::OpenPopup("goto_audio".to_string()),
            Some("Themes") => SettingsInteraction::OpenPopup("goto_themes".to_string()),
            _ => SettingsInteraction::None,
        };

        (section_y + 280.0 + spacing::LG, interaction)
    }

    fn render_theme_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let section_y = y;
        let card_width = (width - spacing::XL * 2.0 - spacing::LG * 3.0) / 4.0;
        let card_height = card_width / sizes::THEME_CARD_ASPECT_RATIO;
        let section_height = 60.0 + card_height + spacing::XL * 2.0;

        let panel = GlassPanel::new(x, section_y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        ply_fonts::draw_headline(
            "Theme Presets",
            x + spacing::XL,
            section_y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "Choose a preset theme for the application",
            x + spacing::XL,
            section_y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        self.theme_cards.clear();
        let current_theme = config.piano_theme_name();
        let mut card_x = x + spacing::XL;
        let mut clicked_theme = None;

        for (i, theme) in themes::ALL_THEMES.iter().enumerate() {
            let mut card = ThemeCard::from_preset(theme, card_x, section_y + 60.0, card_width);
            let is_active = current_theme == theme.id;
            card = card.active(is_active);
            card.render(mx, my);

            if card.was_clicked(mx, my, mouse_pressed) {
                clicked_theme = Some(theme.id.to_string());
            }

            self.theme_cards.push(card);
            card_x += card_width + spacing::LG;
        }

        let interaction = if let Some(theme_id) = clicked_theme {
            SettingsInteraction::ThemeSelected(theme_id)
        } else {
            SettingsInteraction::None
        };

        (section_y + section_height + spacing::LG, interaction)
    }

    fn render_directories_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let section_y = y;
        let directories = config.song_directories();
        let section_height = 80.0 + directories.len() as f32 * 64.0 + 60.0;

        let panel = GlassPanel::new(x, section_y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        ply_fonts::draw_headline(
            "Song Directories",
            x + spacing::XL,
            section_y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            &format!("{} directories configured", directories.len()),
            x + spacing::XL,
            section_y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let mut item_y = section_y + 70.0;
        let mut remove_index = None;

        for (idx, dir) in directories.iter().enumerate() {
            let is_hovered = mx >= x + spacing::XL
                && mx <= x + width - spacing::XL
                && my >= item_y
                && my <= item_y + 56.0;

            let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGH);
            let opacity = if is_hovered { 1.0 } else { 0.5 };
            draw_rectangle(
                x + spacing::XL,
                item_y,
                width - spacing::XL * 2.0,
                56.0,
                Color::new(bg_r, bg_g, bg_b, opacity),
            );

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::PRIMARY);
            ply_fonts::draw_body(
                "📁",
                x + spacing::XL + spacing::MD,
                item_y + 32.0,
                20.0,
                Color::new(icon_r, icon_g, icon_b, 1.0),
            );

            let path_str = dir.to_string_lossy();
            let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
            ply_fonts::draw_body(
                &path_str,
                x + spacing::XL + 40.0,
                item_y + 28.0,
                14.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            let (meta_r, meta_g, meta_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
            ply_fonts::draw_body(
                "MIDI files",
                x + spacing::XL + 40.0,
                item_y + 44.0,
                12.0,
                Color::new(meta_r, meta_g, meta_b, 1.0),
            );

            // Delete button (X) on hover
            if is_hovered {
                let del_x = x + width - spacing::XL - 32.0;
                let del_y = item_y + 16.0;
                let is_del_hovered =
                    mx >= del_x && mx <= del_x + 24.0 && my >= del_y && my <= del_y + 24.0;

                let (del_r, del_g, del_b) = colors::to_normalized(colors::ERROR);
                ply_fonts::draw_body(
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

            item_y += 64.0;
        }

        // Add directory button
        let btn_x = x + spacing::XL;
        let btn_y = item_y + spacing::SM;
        let btn_w = 180.0;
        let btn_h = 40.0;

        let is_btn_hovered =
            mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (btn_bg_r, btn_bg_g, btn_bg_b) = colors::to_normalized(if is_btn_hovered {
            colors::PRIMARY
        } else {
            colors::SURFACE_CONTAINER_HIGH
        });
        draw_rectangle(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(
                btn_bg_r,
                btn_bg_g,
                btn_bg_b,
                if is_btn_hovered { 0.2 } else { 1.0 },
            ),
        );

        let (btn_text_r, btn_text_g, btn_text_b) = colors::to_normalized(if is_btn_hovered {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE
        });
        ply_fonts::draw_body(
            "+ Add Directory",
            btn_x + spacing::MD,
            btn_y + 26.0,
            14.0,
            Color::new(btn_text_r, btn_text_g, btn_text_b, 1.0),
        );

        let interaction = if let Some(idx) = remove_index {
            SettingsInteraction::RemoveSongDirectory(idx)
        } else if is_btn_hovered && mouse_pressed {
            SettingsInteraction::AddSongDirectory
        } else {
            SettingsInteraction::None
        };

        (section_y + section_height + spacing::LG, interaction)
    }

    fn render_actions_section(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let section_y = y;
        let panel = GlassPanel::new(x, section_y, width, 80.0);
        panel.render();

        let btn_w = 140.0;
        let btn_h = 40.0;
        let btn_y = section_y + 20.0;

        // Reset button
        let reset_x = x + spacing::XL;
        let is_reset_hovered =
            mx >= reset_x && mx <= reset_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (reset_r, reset_g, reset_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_rectangle_lines(
            reset_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(reset_r, reset_g, reset_b, 0.3),
        );
        ply_fonts::draw_body(
            "Reset Defaults",
            reset_x + spacing::MD,
            btn_y + 26.0,
            14.0,
            Color::new(reset_r, reset_g, reset_b, 1.0),
        );

        // Save button
        let save_x = x + width - spacing::XL - btn_w;
        let is_save_hovered =
            mx >= save_x && mx <= save_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (save_r, save_g, save_b) = colors::to_normalized(colors::PRIMARY);
        let save_opacity = if is_save_hovered { 1.0 } else { 0.8 };
        draw_rectangle(
            save_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(save_r, save_g, save_b, save_opacity),
        );
        ply_fonts::draw_body(
            "Save Changes",
            save_x + spacing::MD,
            btn_y + 26.0,
            14.0,
            Color::new(0.0, 0.0, 0.0, 1.0),
        );

        let interaction = if is_reset_hovered && mouse_pressed {
            SettingsInteraction::ResetToDefaults
        } else if is_save_hovered && mouse_pressed {
            SettingsInteraction::SaveChanges
        } else {
            SettingsInteraction::None
        };

        (section_y + 80.0 + spacing::LG, interaction)
    }

    fn render_setting_row(
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
        let opacity = if is_hovered { 0.8 } else { 0.4 };
        draw_rectangle(x, y, width, height, Color::new(bg_r, bg_g, bg_b, opacity));

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_body(
            label,
            x + spacing::MD,
            y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let (value_r, value_g, value_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            value,
            x + spacing::MD,
            y + 38.0,
            12.0,
            Color::new(value_r, value_g, value_b, 1.0),
        );

        let arrow_x = x + width - 24.0;
        let arrow_y = y + height / 2.0;
        ply_fonts::draw_body(
            ">",
            arrow_x,
            arrow_y + 5.0,
            14.0,
            Color::new(value_r, value_g, value_b, 0.5),
        );

        is_hovered
    }
}

impl SettingsPage for GeneralPage {
    fn title(&self) -> &str {
        "General"
    }

    fn description(&self) -> &str {
        "Configure MIDI devices, audio output, and application preferences"
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

        let (next_y, interaction) = self.render_status_section(
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

        let (_, interaction) =
            self.render_actions_section(content_x, current_y, content_width, mx, my, mouse_pressed);
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
