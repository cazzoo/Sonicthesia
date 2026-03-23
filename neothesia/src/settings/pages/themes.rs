use crate::settings::interaction::SettingsInteraction;
use crate::settings::page::SettingsPage;
use crate::ui::components::{ColorPicker, GlassPanel, ThemeCard};
use macroquad::prelude::*;
use neothesia_core::config::Config;
use neothesia_core::design::{colors, effects, radius, sizes, spacing, themes};

pub struct ThemesPage {
    scroll_offset: f32,
    theme_cards: Vec<ThemeCard>,
    primary_picker: ColorPicker,
    secondary_picker: ColorPicker,
    tertiary_picker: ColorPicker,
}

impl ThemesPage {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            theme_cards: Vec::new(),
            primary_picker: ColorPicker::new(0.0, 0.0, colors::PRIMARY),
            secondary_picker: ColorPicker::new(0.0, 0.0, colors::SECONDARY),
            tertiary_picker: ColorPicker::new(0.0, 0.0, colors::TERTIARY),
        }
    }

    fn render_header(&self, x: f32, y: f32, width: f32) -> f32 {
        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Theme Settings",
            x,
            y + 28.0,
            24.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let line_x = x + 160.0;
        let line_w = width - 160.0;
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

    fn render_presets_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let card_width = (width - spacing::LG * 3.0) / 4.0;
        let card_height = card_width / sizes::THEME_CARD_ASPECT_RATIO;
        let section_height = 60.0 + card_height + spacing::XL * 2.0;

        let panel = GlassPanel::new(x, y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Keyboard Theme Presets",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Choose a preset theme for your piano keyboard",
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        self.theme_cards.clear();
        let current_theme = config.piano_theme_name();
        let mut card_x = x + spacing::XL;
        let mut clicked_theme = None;

        for theme in themes::ALL_THEMES.iter() {
            let mut card = ThemeCard::from_preset(theme, card_x, y + 60.0, card_width);
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

        (y + section_height + spacing::LG, interaction)
    }

    fn render_creator_section(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let left_width = width * 0.6;
        let right_width = width * 0.4 - spacing::LG;
        let section_height = 320.0;

        let panel = GlassPanel::new(x, y, width, section_height);
        panel.render();

        let (title_r, title_g, title_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Theme Creator",
            x + spacing::XL,
            y + spacing::XL + 16.0,
            20.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Create your own custom theme",
            x + spacing::XL,
            y + spacing::XL + 36.0,
            14.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        let form_x = x + spacing::XL;
        let mut form_y = y + 70.0;
        let form_w = left_width - spacing::XL * 2.0;
        let row_h = 48.0;

        let (label_r, label_g, label_b) = colors::to_normalized(colors::ON_SURFACE);

        // Theme name input
        draw_text(
            "Theme Name",
            form_x,
            form_y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let input_y = form_y + 28.0;
        let (input_r, input_g, input_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            form_x,
            input_y,
            form_w,
            32.0,
            Color::new(input_r, input_g, input_b, 1.0),
        );
        draw_text(
            "My Custom Theme",
            form_x + spacing::MD,
            input_y + 22.0,
            14.0,
            Color::new(label_r, label_g, label_b, 0.5),
        );

        form_y += row_h + spacing::SM;

        // Primary color picker
        draw_text(
            "Primary Color",
            form_x,
            form_y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        self.primary_picker.x = form_x + form_w - 60.0;
        self.primary_picker.y = form_y;
        self.primary_picker.render(mx, my, mouse_pressed);

        form_y += row_h + spacing::SM;

        // Secondary color picker
        draw_text(
            "Secondary Color",
            form_x,
            form_y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        self.secondary_picker.x = form_x + form_w - 60.0;
        self.secondary_picker.y = form_y;
        self.secondary_picker.render(mx, my, mouse_pressed);

        form_y += row_h + spacing::SM;

        // Accent color picker
        draw_text(
            "Accent Color",
            form_x,
            form_y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );
        self.tertiary_picker.x = form_x + form_w - 60.0;
        self.tertiary_picker.y = form_y;
        self.tertiary_picker.render(mx, my, mouse_pressed);

        form_y += row_h + spacing::SM;

        // Glow intensity slider
        draw_text(
            "Glow Intensity",
            form_x,
            form_y + 20.0,
            14.0,
            Color::new(label_r, label_g, label_b, 1.0),
        );

        let slider_x = form_x;
        let slider_y = form_y + 30.0;
        let slider_w = form_w;
        let slider_h = 30.0;

        let (track_r, track_g, track_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            slider_x,
            slider_y + 13.0,
            slider_w,
            4.0,
            Color::new(track_r, track_g, track_b, 1.0),
        );

        let fill_w = slider_w * 0.7;
        let (fill_r, fill_g, fill_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            slider_x,
            slider_y + 13.0,
            fill_w,
            4.0,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        let thumb_x = slider_x + fill_w - 8.0;
        draw_circle(
            thumb_x,
            slider_y + 15.0,
            8.0,
            Color::new(fill_r, fill_g, fill_b, 1.0),
        );

        // Preview panel
        let preview_x = x + left_width + spacing::LG;
        let preview_y = y + 70.0;
        let preview_w = right_width;
        let preview_h = section_height - 90.0;

        let (preview_r, preview_g, preview_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(
            preview_x,
            preview_y,
            preview_w,
            preview_h,
            Color::new(preview_r, preview_g, preview_b, 1.0),
        );

        let (preview_label_r, preview_label_g, preview_label_b) =
            colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "Live Preview",
            preview_x + spacing::MD,
            preview_y + 20.0,
            12.0,
            Color::new(preview_label_r, preview_label_g, preview_label_b, 1.0),
        );

        // Color swatches preview
        let swatch_size = 24.0;
        let swatch_y = preview_y + 40.0;

        let (p_r, p_g, p_b) = colors::to_normalized(self.primary_picker.current_color);
        draw_rectangle(
            preview_x + spacing::MD,
            swatch_y,
            swatch_size,
            swatch_size,
            Color::new(p_r, p_g, p_b, 1.0),
        );

        let (s_r, s_g, s_b) = colors::to_normalized(self.secondary_picker.current_color);
        draw_rectangle(
            preview_x + spacing::MD + swatch_size + spacing::SM,
            swatch_y,
            swatch_size,
            swatch_size,
            Color::new(s_r, s_g, s_b, 1.0),
        );

        let (t_r, t_g, t_b) = colors::to_normalized(self.tertiary_picker.current_color);
        draw_rectangle(
            preview_x + spacing::MD + (swatch_size + spacing::SM) * 2.0,
            swatch_y,
            swatch_size,
            swatch_size,
            Color::new(t_r, t_g, t_b, 1.0),
        );

        // Mini keyboard preview
        let mini_kb_y = preview_y + 80.0;
        let mini_kb_w = preview_w - spacing::LG * 2.0;
        let mini_kb_h = 60.0;
        let mini_kb_x = preview_x + spacing::LG;

        let (kb_r, kb_g, kb_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            mini_kb_x,
            mini_kb_y,
            mini_kb_w,
            mini_kb_h,
            Color::new(kb_r, kb_g, kb_b, 1.0),
        );

        let white_key_count = 14;
        let white_key_w = mini_kb_w / white_key_count as f32;
        for i in 0..white_key_count {
            let kx = mini_kb_x + i as f32 * white_key_w;
            draw_rectangle(
                kx + 0.5,
                mini_kb_y + 2.0,
                white_key_w - 1.0,
                mini_kb_h - 4.0,
                Color::new(p_r, p_g, p_b, 0.6),
            );
        }

        // Full screen preview button
        let btn_x = preview_x + spacing::LG;
        let btn_y = preview_y + preview_h - 50.0;
        let btn_w = preview_w - spacing::LG * 2.0;
        let btn_h = 36.0;

        let is_btn_hovered =
            mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (btn_r, btn_g, btn_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(btn_r, btn_g, btn_b, if is_btn_hovered { 0.3 } else { 0.15 }),
        );
        draw_rectangle_lines(
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            1.0,
            Color::new(btn_r, btn_g, btn_b, 0.5),
        );

        let btn_text_w = measure_text("Full Screen Preview", None, 13, 1.0).width;
        draw_text(
            "Full Screen Preview",
            btn_x + (btn_w - btn_text_w) / 2.0,
            btn_y + 24.0,
            13.0,
            Color::new(btn_r, btn_g, btn_b, 1.0),
        );

        (y + section_height + spacing::LG, SettingsInteraction::None)
    }

    fn render_actions(
        &self,
        x: f32,
        y: f32,
        width: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> (f32, SettingsInteraction) {
        let panel = GlassPanel::new(x, y, width, 80.0);
        panel.render();

        let btn_w = 140.0;
        let btn_h = 40.0;
        let btn_y = y + 20.0;

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
        let reset_text_w = measure_text("Reset to Default", None, 14, 1.0).width;
        draw_text(
            "Reset to Default",
            reset_x + (btn_w - reset_text_w) / 2.0,
            btn_y + 26.0,
            14.0,
            Color::new(reset_r, reset_g, reset_b, 1.0),
        );

        // Save button
        let save_x = x + width - spacing::XL - btn_w;
        let is_save_hovered =
            mx >= save_x && mx <= save_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
        let (save_r, save_g, save_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            save_x,
            btn_y,
            btn_w,
            btn_h,
            Color::new(
                save_r,
                save_g,
                save_b,
                if is_save_hovered { 1.0 } else { 0.8 },
            ),
        );
        let save_text_w = measure_text("Save Theme", None, 14, 1.0).width;
        draw_text(
            "Save Theme",
            save_x + (btn_w - save_text_w) / 2.0,
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

        (y + 80.0 + spacing::LG, interaction)
    }
}

impl SettingsPage for ThemesPage {
    fn title(&self) -> &str {
        "Themes"
    }

    fn description(&self) -> &str {
        "Customize the look and feel of your piano keyboard"
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

        let (next_y, interaction) = self.render_presets_section(
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
            self.render_creator_section(content_x, current_y, content_width, mx, my, mouse_pressed);
        if !matches!(interaction, SettingsInteraction::None) {
            return interaction;
        }
        current_y = next_y;

        let (_, interaction) =
            self.render_actions(content_x, current_y, content_width, mx, my, mouse_pressed);
        interaction
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset - delta * 20.0).max(0.0);
    }
}
