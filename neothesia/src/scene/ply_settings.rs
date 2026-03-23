//! PLY Settings Scene - Interactive settings menu

use crate::{context_macroquad::MacroquadContext, settings::SettingsPage, NeothesiaEvent};
use std::time::Duration;

use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use super::PlyScene;

/// PLY Settings Scene - Interactive settings menu with all controls
pub struct PlySettingsScene {
    pending_nav_event: Option<NeothesiaEvent>,
    settings_nav: crate::settings::SettingsNav,
    general_page: crate::settings::pages::GeneralPage,
    midi_page: crate::settings::pages::MidiPage,
    audio_page: crate::settings::pages::AudioPage,
    themes_page: crate::settings::pages::ThemesPage,
    folders_page: crate::settings::pages::FoldersPage,
}

impl PlySettingsScene {
    pub fn new() -> Self {
        Self {
            pending_nav_event: None,
            settings_nav: crate::settings::SettingsNav::new(),
            general_page: crate::settings::pages::GeneralPage::new(),
            midi_page: crate::settings::pages::MidiPage::new(),
            audio_page: crate::settings::pages::AudioPage::new(),
            themes_page: crate::settings::pages::ThemesPage::new(),
            folders_page: crate::settings::pages::FoldersPage::new(),
        }
    }

    pub fn initialize(&mut self, ctx: &mut MacroquadContext) {
        let folders = ctx.config.synth_config.soundfont_folders().to_vec();
        log::info!(
            "Initializing settings with {} soundfont folders",
            folders.len()
        );
    }

    fn render_top_nav(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
    ) -> Option<NeothesiaEvent> {
        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
        draw_rectangle(x, y, width, height, Color::new(bg_r, bg_g, bg_b, 0.95));

        let logo_x = x + spacing::XL;
        let logo_y = y + height / 2.0 + 8.0;
        let (logo_r, logo_g, logo_b) = colors::to_normalized(colors::PRIMARY);
        draw_text(
            "Sonicthesia",
            logo_x,
            logo_y,
            24.0,
            Color::new(logo_r, logo_g, logo_b, 1.0),
        );

        let nav_items = ["Library", "Practice", "Settings"];
        let mut nav_x = width - 400.0;
        let mut clicked_event = None;

        for item in nav_items.iter() {
            let is_hovered = mx >= nav_x && mx <= nav_x + 80.0 && my >= y && my <= y + height;
            let is_active = *item == "Settings";

            let text_color = if is_active {
                colors::PRIMARY
            } else if is_hovered {
                colors::ON_SURFACE
            } else {
                colors::ON_SURFACE_VARIANT
            };

            let (text_r, text_g, text_b) = colors::to_normalized(text_color);
            draw_text(
                item,
                nav_x,
                logo_y,
                16.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            if is_active {
                draw_rectangle(
                    nav_x,
                    y + height - 3.0,
                    60.0,
                    3.0,
                    Color::new(text_r, text_g, text_b, 1.0),
                );
            }

            if is_hovered && mouse_pressed {
                clicked_event = match *item {
                    "Library" => Some(NeothesiaEvent::ShowSongLibrary(None)),
                    "Practice" => Some(NeothesiaEvent::ResumeFromSettings),
                    _ => None,
                };
            }
            nav_x += 100.0;
        }
        clicked_event
    }

    fn handle_settings_interaction(
        &mut self,
        ctx: &mut MacroquadContext,
        interaction: crate::settings::SettingsInteraction,
    ) {
        use crate::settings::SettingsInteraction;

        match interaction {
            SettingsInteraction::None => {}
            SettingsInteraction::ThemeSelected(theme_id) => {
                ctx.config.set_piano_theme_name(theme_id);
                ctx.config.save();
            }
            SettingsInteraction::AddSongDirectory => {
                self.pick_song_directory(ctx);
            }
            SettingsInteraction::RemoveSongDirectory(idx) => {
                ctx.config.remove_song_directory(idx);
                ctx.config.save();
            }
            SettingsInteraction::AddSoundFontFolder => {
                self.pick_soundfont_folder(ctx);
                self.audio_page.mark_needs_refresh();
            }
            SettingsInteraction::RemoveSoundFontFolder(idx) => {
                ctx.config.synth_config.remove_soundfont_folder(idx);
                ctx.config.save();
                self.audio_page.mark_needs_refresh();
            }
            SettingsInteraction::SoundFontSelected(idx) => {
                log::info!("SoundFontSelected: index={}", idx);
                ctx.config.synth_config.set_soundfont_index(Some(idx));
                let sf_path = self.audio_page.get_soundfont_path(idx);
                if let Some(path) = sf_path {
                    log::info!("Switching to soundfont: {:?}", path);
                    if !path.exists() {
                        log::error!("SoundFont file does not exist: {:?}", path);
                    } else if !ctx.output_manager.is_synth_output() {
                        log::warn!("Cannot switch soundfont: not connected to synth output");
                        log::info!("Please select a synth output first (e.g., 'Buildin Synth')");
                    } else {
                        ctx.config
                            .synth_config
                            .set_soundfont_path(Some(path.clone()));
                        match ctx.output_manager.switch_soundfont(&path) {
                            Ok(()) => log::info!("Successfully switched soundfont"),
                            Err(e) => log::error!("Failed to switch soundfont: {}", e),
                        }
                    }
                } else {
                    log::warn!("No soundfont path found for index {}", idx);
                }
                ctx.config.save();
            }
            SettingsInteraction::InputDeviceSelected(device) => {
                ctx.config.set_input(Some(device));
                ctx.config.save();
            }
            SettingsInteraction::OutputDeviceSelected(device) => {
                ctx.config.set_output(Some(device));
                ctx.config.save();
            }
            SettingsInteraction::AudioGainChanged(gain) => {
                ctx.config.set_audio_gain(gain);
                ctx.config.save();
            }
            SettingsInteraction::PlaybackGainChanged(gain) => {
                ctx.config.set_playback_gain(gain);
                ctx.config.save();
            }
            SettingsInteraction::PianoRangeStartChanged(start) => {
                ctx.config.set_piano_range_start(start);
                ctx.config.save();
            }
            SettingsInteraction::PianoRangeEndChanged(end) => {
                ctx.config.set_piano_range_end(end);
                ctx.config.save();
            }
            SettingsInteraction::VerticalGuidelinesToggled(enabled) => {
                ctx.config.set_vertical_guidelines(enabled);
                ctx.config.save();
            }
            SettingsInteraction::HorizontalGuidelinesToggled(enabled) => {
                ctx.config.set_horizontal_guidelines(enabled);
                ctx.config.save();
            }
            SettingsInteraction::GlowToggled(enabled) => {
                ctx.config.set_glow(enabled);
                ctx.config.save();
            }
            SettingsInteraction::NoteLabelsToggled(enabled) => {
                ctx.config.set_note_labels(enabled);
                ctx.config.save();
            }
            SettingsInteraction::SeparateChannelsToggled(enabled) => {
                ctx.config.set_separate_channels(enabled);
                ctx.config.save();
            }
            SettingsInteraction::PlayNote(key, vel) => {
                use midi_file::midly::{num::u7, MidiMessage};
                let message = MidiMessage::NoteOn {
                    key: u7::new(key),
                    vel: u7::new(vel),
                };
                ctx.output_manager
                    .connection()
                    .midi_event(0u8.into(), message);
            }
            SettingsInteraction::StopNote(key) => {
                use midi_file::midly::{num::u7, MidiMessage};
                let message = MidiMessage::NoteOff {
                    key: u7::new(key),
                    vel: u7::new(0),
                };
                ctx.output_manager
                    .connection()
                    .midi_event(0u8.into(), message);
            }
            SettingsInteraction::SaveChanges => {
                ctx.config.save();
            }
            SettingsInteraction::OpenPopup(popup_type) => match popup_type.as_str() {
                "input" => {
                    log::info!("Opening input selector popup");
                }
                "output" => {
                    log::info!("Opening output selector popup");
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn pick_song_directory(&mut self, ctx: &mut MacroquadContext) {
        log::info!("pick_song_directory called - file picker integration needed");
    }

    fn pick_soundfont_folder(&mut self, ctx: &mut MacroquadContext) {
        log::info!("pick_soundfont_folder called - file picker integration needed");
    }
}

impl PlyScene for PlySettingsScene {
    fn update(&mut self, ctx: &mut MacroquadContext, _delta: Duration) -> Option<NeothesiaEvent> {
        if let Some(nav_event) = self.pending_nav_event.take() {
            return Some(nav_event);
        }

        if is_key_pressed(KeyCode::Escape) {
            return Some(NeothesiaEvent::ResumeFromSettings);
        }

        None
    }

    fn render(&mut self, ctx: &mut MacroquadContext) {
        clear_background(Color::from_rgba(14, 14, 19, 255));

        let screen_w = screen_width();
        let screen_h = screen_height();

        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        if let Some(nav_event) =
            self.render_top_nav(0.0, 0.0, screen_w, 64.0, mouse_x, mouse_y, mouse_pressed)
        {
            self.pending_nav_event = Some(nav_event);
        }

        let sidebar_w = 256.0;
        let sidebar_h = screen_h - 64.0;
        self.settings_nav
            .render(0.0, 64.0, sidebar_w, mouse_x, mouse_y, mouse_pressed);

        let content_x = sidebar_w;
        let content_y = 64.0;
        let content_w = screen_w - sidebar_w;
        let content_h = sidebar_h;

        let interaction = match self.settings_nav.current_tab {
            crate::settings::SettingsTab::General => self.general_page.render(
                content_x,
                content_y,
                content_w,
                content_h,
                &ctx.config,
                mouse_x,
                mouse_y,
                mouse_pressed,
                mouse_down,
            ),
            crate::settings::SettingsTab::Midi => self.midi_page.render(
                content_x,
                content_y,
                content_w,
                content_h,
                &ctx.config,
                mouse_x,
                mouse_y,
                mouse_pressed,
                mouse_down,
            ),
            crate::settings::SettingsTab::Audio => self.audio_page.render(
                content_x,
                content_y,
                content_w,
                content_h,
                &ctx.config,
                mouse_x,
                mouse_y,
                mouse_pressed,
                mouse_down,
            ),
            crate::settings::SettingsTab::Themes => self.themes_page.render(
                content_x,
                content_y,
                content_w,
                content_h,
                &ctx.config,
                mouse_x,
                mouse_y,
                mouse_pressed,
                mouse_down,
            ),
            crate::settings::SettingsTab::Folders => self.folders_page.render(
                content_x,
                content_y,
                content_w,
                content_h,
                &ctx.config,
                mouse_x,
                mouse_y,
                mouse_pressed,
                mouse_down,
            ),
        };

        self.handle_settings_interaction(ctx, interaction);

        let mouse_wheel = mouse_wheel();
        if mouse_wheel.1 != 0.0 {
            match self.settings_nav.current_tab {
                crate::settings::SettingsTab::General => {
                    self.general_page.handle_scroll(mouse_wheel.1)
                }
                crate::settings::SettingsTab::Midi => self.midi_page.handle_scroll(mouse_wheel.1),
                crate::settings::SettingsTab::Audio => self.audio_page.handle_scroll(mouse_wheel.1),
                crate::settings::SettingsTab::Themes => {
                    self.themes_page.handle_scroll(mouse_wheel.1)
                }
                crate::settings::SettingsTab::Folders => {
                    self.folders_page.handle_scroll(mouse_wheel.1)
                }
            }
        }
    }
}
