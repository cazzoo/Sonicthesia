use crate::{context_macroquad::MacroquadContext, settings::SettingsPage, NeothesiaEvent};
use std::time::Duration;

use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use super::PlyScene;
use crate::ui::components::{Header, NavItem, Sidebar, SidebarSection};
use crate::virtual_resolution::{vh, vmouse, vw};

pub struct PlySettingsScene {
    pending_nav_event: Option<NeothesiaEvent>,
    header: Header,
    sidebar: Sidebar,
    settings_nav: crate::settings::SettingsNav,
    general_page: crate::settings::pages::GeneralPage,
    midi_page: crate::settings::pages::MidiPage,
    audio_page: crate::settings::pages::AudioPage,
    themes_page: crate::settings::pages::ThemesPage,
    folders_page: crate::settings::pages::FoldersPage,
}

impl PlySettingsScene {
    pub fn new() -> Self {
        let mut header = Header::new();
        header.set_active_nav(NavItem::Settings);

        let mut sidebar = Sidebar::new();
        sidebar.set_active_section(SidebarSection::MidiLibrary);

        Self {
            pending_nav_event: None,
            header,
            sidebar,
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
                    } else {
                        ctx.config
                            .synth_config
                            .set_soundfont_path(Some(path.clone()));
                        match ctx.output_manager.switch_soundfont(&path) {
                            Ok(()) => log::info!("Successfully switched soundfont"),
                            Err(e) => log::error!("Failed to switch soundfont: {}", e),
                        }
                    }
                }
                ctx.config.save();
            }
            SettingsInteraction::InputDeviceSelected(device) => {
                log::info!("[SETTINGS] Input device selected: '{}'", device);
                ctx.config.set_input(Some(&device));
                ctx.config.save();
                ctx.midi_input.connect_input(&device);
            }
            SettingsInteraction::OutputDeviceSelected(device) => {
                log::info!("Output device selected: {}", device);
                let outputs = ctx.output_manager.outputs();
                if let Some(desc) = outputs.into_iter().find(|o| o.to_string() == device) {
                    ctx.output_manager.connect(desc);
                }
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
            SettingsInteraction::VelocityEnabledToggled(enabled) => {
                ctx.config.set_velocity_enabled(enabled);
                ctx.config.save();
            }
            SettingsInteraction::VelocityMinChanged(val) => {
                ctx.config.set_velocity_min(val);
                ctx.config.save();
            }
            SettingsInteraction::VelocityMaxChanged(val) => {
                ctx.config.set_velocity_max(val);
                ctx.config.save();
            }
            SettingsInteraction::PressureSensitivityChanged(val) => {
                ctx.config.set_pressure_sensitivity(val);
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
                    self.cycle_input_device(ctx);
                }
                "output" => {
                    self.cycle_output_device(ctx);
                }
                "goto_midi" => {
                    self.settings_nav.current_tab = crate::settings::SettingsTab::Midi;
                }
                "goto_audio" => {
                    self.settings_nav.current_tab = crate::settings::SettingsTab::Audio;
                }
                "goto_themes" => {
                    self.settings_nav.current_tab = crate::settings::SettingsTab::Themes;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn pick_song_directory(&mut self, ctx: &mut MacroquadContext) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            log::info!("Selected song directory: {:?}", folder);
            ctx.config.add_song_directory(folder);
            ctx.config.save();
        }
    }

    fn pick_soundfont_folder(&mut self, ctx: &mut MacroquadContext) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            log::info!("Selected soundfont folder: {:?}", folder);
            ctx.config.synth_config.add_soundfont_folder(folder);
            ctx.config.save();
        }
    }

    fn cycle_input_device(&mut self, ctx: &mut MacroquadContext) {
        let ports = ctx.midi_input.inputs();
        if ports.is_empty() {
            log::info!("No MIDI input devices available");
            return;
        }

        let current = ctx.config.input().unwrap_or("");
        let next = if let Some(idx) = ports.iter().position(|n| n == current) {
            ports[(idx + 1) % ports.len()].clone()
        } else {
            ports[0].clone()
        };

        log::info!("Switching input device to: {}", next);
        ctx.config.set_input(Some(&next));
        ctx.config.save();
        ctx.midi_input.connect_input(&next);
    }

    fn cycle_output_device(&mut self, ctx: &mut MacroquadContext) {
        let outputs = ctx.output_manager.outputs();
        if outputs.is_empty() {
            log::info!("No MIDI output devices available");
            return;
        }

        let current = ctx.config.output().unwrap_or("");
        let output_names: Vec<String> = outputs.iter().map(|o| o.to_string()).collect();

        let next = if let Some(idx) = output_names.iter().position(|n| n == current) {
            output_names[(idx + 1) % output_names.len()].clone()
        } else {
            output_names[0].clone()
        };

        log::info!("Switching output device to: {}", next);

        let descriptor = outputs.into_iter().find(|o| o.to_string() == next).unwrap();

        ctx.output_manager.connect(descriptor);
        ctx.config.set_output(Some(next));
        ctx.config.save();
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
        clear_background(Color::new(0.055, 0.055, 0.075, 1.0));

        let screen_w = vw();
        let screen_h = vh();

        let (mouse_x, mouse_y) = vmouse();
        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        let content_x = self.sidebar.width;
        let content_y = self.header.height;
        let content_w = screen_w - self.sidebar.width;
        let content_h = screen_h - self.header.height;

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
            crate::settings::SettingsTab::Midi => {
                self.midi_page.input_devices = ctx.midi_input.inputs();
                self.midi_page.output_devices = ctx
                    .output_manager
                    .outputs()
                    .iter()
                    .map(|o| o.to_string())
                    .collect();
                self.midi_page.pressure_history = ctx.midi_input.pressure_history().to_vec();
                self.midi_page.active_pressure = ctx.midi_input.active_note_pressure();
                self.midi_page.render(
                    content_x,
                    content_y,
                    content_w,
                    content_h,
                    &ctx.config,
                    mouse_x,
                    mouse_y,
                    mouse_pressed,
                    mouse_down,
                )
            }
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
        if mouse_wheel.1 != 0.0
            && !self.sidebar.contains_point(mouse_x, mouse_y)
            && !self.header.contains_point(mouse_x, mouse_y)
        {
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

        let header_nav = self.header.render(mouse_x, mouse_y, mouse_pressed);
        if let Some(nav) = header_nav {
            match nav {
                NavItem::Back => {}
                NavItem::Library => {
                    self.pending_nav_event = Some(NeothesiaEvent::ShowSongLibrary(None));
                }
                NavItem::FreePlay => {
                    self.pending_nav_event = Some(NeothesiaEvent::FreePlay(None));
                }
                NavItem::Practice => {
                    self.pending_nav_event = Some(NeothesiaEvent::ResumeFromSettings);
                }
                NavItem::Settings => {}
            }
        }

        self.render_settings_sidebar(mouse_x, mouse_y, mouse_pressed);
    }
}

impl PlySettingsScene {
    fn render_settings_sidebar(&mut self, mx: f32, my: f32, mouse_pressed: bool) {
        let screen_h = vh();
        let sidebar_w = self.sidebar.width;
        let top_offset = self.header.height;

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(
            0.0,
            top_offset,
            sidebar_w,
            screen_h - top_offset,
            Color::new(bg_r, bg_g, bg_b, 1.0),
        );

        let (border_r, border_g, border_b) = colors::to_normalized(colors::OUTLINE_VARIANT);
        draw_rectangle(
            sidebar_w - 1.0,
            top_offset,
            1.0,
            screen_h - top_offset,
            Color::new(border_r, border_g, border_b, 0.1),
        );

        use crate::scene::ply_fonts;

        let content_x = spacing::LG;
        let mut current_y = top_offset + spacing::XL + 20.0;

        let (title_r, title_g, title_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Settings",
            content_x,
            current_y,
            18.0,
            Color::new(title_r, title_g, title_b, 1.0),
        );

        current_y += 24.0;
        let (sub_r, sub_g, sub_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        ply_fonts::draw_body(
            "Configure Neothesia",
            content_x,
            current_y,
            12.0,
            Color::new(sub_r, sub_g, sub_b, 1.0),
        );

        current_y += 36.0;

        use crate::settings::SettingsTab;
        for tab in SettingsTab::all() {
            let is_active = self.settings_nav.current_tab == tab;
            let is_hovered =
                mx >= 0.0 && mx <= sidebar_w && my >= current_y && my <= current_y + 44.0;

            if is_active {
                let (active_bg_r, active_bg_g, active_bg_b) =
                    colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
                draw_rectangle(
                    spacing::SM,
                    current_y,
                    sidebar_w - spacing::LG,
                    44.0,
                    Color::new(active_bg_r, active_bg_g, active_bg_b, 1.0),
                );

                let (accent_r, accent_g, accent_b) = colors::to_normalized(colors::TERTIARY);
                draw_rectangle(
                    0.0,
                    current_y,
                    3.0,
                    44.0,
                    Color::new(accent_r, accent_g, accent_b, 1.0),
                );
            } else if is_hovered {
                let (hover_r, hover_g, hover_b) = colors::to_normalized(colors::SURFACE_CONTAINER);
                draw_rectangle(
                    spacing::SM,
                    current_y,
                    sidebar_w - spacing::LG,
                    44.0,
                    Color::new(hover_r, hover_g, hover_b, 0.5),
                );
            }

            let text_color = if is_active {
                colors::PRIMARY
            } else if is_hovered {
                colors::ON_SURFACE
            } else {
                colors::ON_SURFACE_VARIANT
            };
            let (text_r, text_g, text_b) = colors::to_normalized(text_color);

            let icon_x = content_x + spacing::SM;
            let icon_y = current_y + 28.0;
            ply_fonts::draw_body(
                tab.icon(),
                icon_x,
                icon_y,
                16.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            let label_x = icon_x + 28.0;
            ply_fonts::draw_body(
                tab.label(),
                label_x,
                icon_y,
                14.0,
                Color::new(text_r, text_g, text_b, 1.0),
            );

            if is_hovered && mouse_pressed {
                self.settings_nav.current_tab = tab;
            }

            current_y += 48.0;
        }
    }
}
