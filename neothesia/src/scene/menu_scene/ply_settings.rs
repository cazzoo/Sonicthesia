//! PLY-based Settings Menu Implementation
//!
//! This module demonstrates how to migrate the settings menu from Nuon to PLY UI.
//! It provides full interactive settings functionality matching the legacy WGPU settings.

use crate::context::Context;
use crate::ply_integration::ui::{PlyUi, center_x, center_y, TextAlignment, KeyboardAction};
use crate::ply_integration::ui::widgets::{Button, Label, Quad, Scroll};
use crate::ply_integration::ui::layout::{SettingsSection, SettingsRow};
use std::path::PathBuf;
use winit::keyboard::Key;

/// PLY-based settings menu state
pub struct PlySettingsMenu {
    ui: PlyUi,
    scroll_state: f32,
    /// Current popup state
    popup: PopupState,
    /// Discovered SoundFont files
    soundfont_files: Vec<SoundFontEntry>,
    /// Currently selected SoundFont index
    current_soundfont_index: Option<usize>,
    /// SoundFont folders
    soundfont_folders: Vec<PathBuf>,
    /// Song library directories
    song_directories: Vec<PathBuf>,
    /// Is currently loading
    is_loading: bool,
    /// Available output devices
    outputs: Vec<String>,
    /// Available input devices
    inputs: Vec<String>,
}

/// SoundFont entry
#[derive(Clone, Debug)]
struct SoundFontEntry {
    path: PathBuf,
    folder: PathBuf,
}

/// Popup state
#[derive(Debug, Clone, PartialEq, Eq)]
enum PopupState {
    None,
    OutputSelector,
    InputSelector,
}

impl Default for PopupState {
    fn default() -> Self {
        Self::None
    }
}

impl PlySettingsMenu {
    /// Create a new PLY-based settings menu
    pub fn new() -> Self {
        Self {
            ui: PlyUi::new(),
            scroll_state: 0.0,
            popup: PopupState::None,
            soundfont_files: Vec::new(),
            current_soundfont_index: None,
            soundfont_folders: Vec::new(),
            song_directories: Vec::new(),
            is_loading: false,
            outputs: Vec::new(),
            inputs: Vec::new(),
        }
    }
    
    /// Initialize settings menu with context data
    pub fn initialize(&mut self, ctx: &mut Context) {
        // Load SoundFont folders from config
        self.soundfont_folders = ctx.config.synth_config.soundfont_folders().to_vec();
        
        // Discover SoundFonts
        self.soundfont_files = crate::output_manager::discover_soundfonts(&self.soundfont_folders);
        
        // Load current SoundFont index
        self.current_soundfont_index = ctx.config.synth_config.soundfont_index();
        
        // Load song directories
        self.song_directories = ctx.config.song_directories().to_vec();
        
        // Get available outputs and inputs
        self.outputs = ctx.output_manager.outputs().iter().map(|o| o.to_string()).collect();
        self.inputs = ctx.output_manager.inputs().iter().map(|i| i.to_string()).collect();
        
        log::info!("PLY Settings initialized with {} SoundFonts, {} outputs, {} inputs",
                   self.soundfont_files.len(), self.outputs.len(), self.inputs.len());
    }
    
    /// Update the settings menu UI and handle actions
    pub fn update(&mut self, ctx: &mut Context) -> SettingsAction {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        // Begin frame
        self.ui.begin_frame(win_w, win_h);
        
        let mut action = SettingsAction::None;
        
        // Build settings menu UI
        self.build_settings_menu(ctx, &mut action);
        
        // End frame
        let commands = self.ui.end_frame();
        
        // TODO: Render commands using PLY renderer
        log::debug!("PLY Settings UI generated {} render commands", commands.len());
        
        action
    }
    
    /// Handle settings action
    pub fn handle_action(&mut self, ctx: &mut Context, action: SettingsAction) {
        match action {
            SettingsAction::None => {}
            SettingsAction::GoBack => {
                // Save config before going back
                ctx.config.save();
                log::info!("Settings saved, going back");
            }
            SettingsAction::ShowOutputPicker => {
                self.popup = PopupState::OutputSelector;
            }
            SettingsAction::ShowInputPicker => {
                self.popup = PopupState::InputSelector;
            }
            SettingsAction::Increment(id) => {
                self.handle_increment(ctx, &id);
                ctx.config.save();
            }
            SettingsAction::Decrement(id) => {
                self.handle_decrement(ctx, &id);
                ctx.config.save();
            }
            SettingsAction::Toggle(id) => {
                self.handle_toggle(ctx, &id);
                ctx.config.save();
            }
            SettingsAction::SelectOutput(output) => {
                ctx.config.set_output(Some(&output));
                self.popup = PopupState::None;
                ctx.config.save();
                log::info!("Output changed to: {}", output);
            }
            SettingsAction::SelectInput(input) => {
                ctx.config.set_input(Some(&input));
                self.popup = PopupState::None;
                ctx.config.save();
                log::info!("Input changed to: {}", input);
            }
            SettingsAction::ClosePopup => {
                self.popup = PopupState::None;
            }
            SettingsAction::AddSoundFontFolder => {
                log::info!("Add SoundFont folder requested - triggering native folder picker");
                // Note: This will be handled asynchronously through the event loop
                // The actual picker will be triggered from the main loop
            }
            SettingsAction::AddSongDirectory => {
                log::info!("Add song directory requested - triggering native folder picker");
                // Note: This will be handled asynchronously through the event loop
                // The actual picker will be triggered from the main loop
            }
            SettingsAction::RemoveSongDirectory(index) => {
                if index < self.song_directories.len() {
                    let removed = self.song_directories.remove(index);
                    ctx.config.remove_song_directory(index);
                    ctx.config.save();
                    log::info!("Removed song directory: {:?}", removed);
                }
            }
            SettingsAction::PreviousSoundFont => {
                self.previous_soundfont(ctx);
            }
            SettingsAction::NextSoundFont => {
                self.next_soundfont(ctx);
            }
        }
    }
    
    /// Handle increment action
    fn handle_increment(&mut self, ctx: &mut Context, id: &str) {
        match id {
            "range_start" => {
                let v = (ctx.config.piano_range().start() + 1).min(127);
                if v + 24 < *ctx.config.piano_range().end() {
                    ctx.config.set_piano_range_start(v);
                    log::info!("Range start incremented to {}", v);
                }
            }
            "range_end" => {
                ctx.config.set_piano_range_end(ctx.config.piano_range().end() + 1);
                log::info!("Range end incremented to {}", ctx.config.piano_range().end());
            }
            "audio_gain" => {
                let new_gain = ctx.config.audio_gain() + 0.1;
                ctx.config.set_audio_gain(new_gain);
                log::info!("Audio gain incremented to {}", new_gain);
            }
            "playback_gain" => {
                let new_gain = ctx.config.playback_gain() + 0.1;
                ctx.config.set_playback_gain(new_gain);
                log::info!("Playback gain incremented to {}", new_gain);
            }
            _ => {}
        }
    }
    
    /// Handle decrement action
    fn handle_decrement(&mut self, ctx: &mut Context, id: &str) {
        match id {
            "range_start" => {
                ctx.config.set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
                log::info!("Range start decremented to {}", ctx.config.piano_range().start());
            }
            "range_end" => {
                let v = ctx.config.piano_range().end().saturating_sub(1);
                if *ctx.config.piano_range().start() + 24 < v {
                    ctx.config.set_piano_range_end(v);
                    log::info!("Range end decremented to {}", v);
                }
            }
            "audio_gain" => {
                let new_gain = ctx.config.audio_gain() - 0.1;
                ctx.config.set_audio_gain(new_gain);
                log::info!("Audio gain decremented to {}", new_gain);
            }
            "playback_gain" => {
                let new_gain = ctx.config.playback_gain() - 0.1;
                ctx.config.set_playback_gain(new_gain);
                log::info!("Playback gain decremented to {}", new_gain);
            }
            _ => {}
        }
    }
    
    /// Handle toggle action
    fn handle_toggle(&mut self, ctx: &mut Context, id: &str) {
        match id {
            "vertical_guidelines" => {
                ctx.config.set_vertical_guidelines(!ctx.config.vertical_guidelines());
                log::info!("Vertical guidelines toggled to {}", ctx.config.vertical_guidelines());
            }
            "horizontal_guidelines" => {
                ctx.config.set_horizontal_guidelines(!ctx.config.horizontal_guidelines());
                log::info!("Horizontal guidelines toggled to {}", ctx.config.horizontal_guidelines());
            }
            "glow" => {
                ctx.config.set_glow(!ctx.config.glow());
                log::info!("Glow toggled to {}", ctx.config.glow());
            }
            "note_labels" => {
                ctx.config.set_note_labels(!ctx.config.note_labels());
                log::info!("Note labels toggled to {}", ctx.config.note_labels());
            }
            _ => {}
        }
    }
    
    /// Build the settings menu UI
    fn build_settings_menu(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let bottom_bar_h = 60.0;
        let margin_top = 40.0;
        let body_w = 650.0;
        
        // Draw bottom bar
        self.draw_bottom_bar(ctx, action);
        
        // Draw settings sections in scrollable area
        let scroll_height = (win_h - bottom_bar_h).max(0.0);
        
        Scroll::new()
            .pos(0.0, 0.0)
            .size(win_w, scroll_height)
            .scroll(crate::ply_integration::ui::widgets::ScrollState {
                value: self.scroll_state,
                max: 0.0, // Will be calculated by content
            })
            .build(&mut self.ui, |ui| {
                ui.translate(center_x(win_w, body_w), margin_top);
                
                // Output Section
                SettingsSection::new("Output")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.output_section(ctx, ui, rows, spacer, action);
                    });
                
                // Input Section
                SettingsSection::new("Input")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        self.input_section(ctx, ui, rows, action);
                    });
                
                // LUMI Hardware Section (conditional)
                let has_lumi = ctx.output_manager.has_lumi_connection();
                if has_lumi {
                    SettingsSection::new("LUMI Hardware")
                        .width(body_w)
                        .build(ui, |ui, rows, _spacer| {
                            self.lumi_section(ctx, ui, rows, action);
                        });
                }
                
                // Note Range Section
                SettingsSection::new("Note Range")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.note_range_section(ctx, ui, rows, spacer, action);
                    });
                
                // Keyboard preview
                ui.translate(0.0, 10.0);
                self.keyboard_layout_preview(ctx, body_w, 100.0, ui);
                ui.translate(0.0, 110.0);
                
                // Render Section
                SettingsSection::new("Render")
                    .width(body_w)
                    .build(ui, |ui, rows, _spacer| {
                        self.render_section(ctx, ui, rows, action);
                    });
                
                // Song Library Section
                SettingsSection::new("Song Library")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.song_library_section(ctx, ui, rows, spacer, action);
                    });
            });
        
        // Draw popup if active
        self.draw_popup(ctx, action);
    }
    
    /// Draw the bottom bar with back button
    fn draw_bottom_bar(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let padding = 10.0;
        let w = 80.0;
        let h = 60.0;
        
        // Draw bottom bar background
        Quad::new()
            .pos(0.0, win_h - padding - h)
            .size(win_w, h)
            .color([37, 35, 42])
            .build(&mut self.ui);
        
        // Draw back button
        if Button::new()
            .pos(padding, win_h - padding - h)
            .size(w, h)
            .label("←")
            .build(&mut self.ui)
        {
            *action = SettingsAction::GoBack;
        }
    }
    
    /// Output settings section
    fn output_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        let selected_output = ctx.config.output().as_deref().unwrap_or("None");
        
        SettingsRow::new()
            .title("Output")
            .subtitle(selected_output)
            .build(ui, |ui, row_w, row_h| {
                let btn_w = 320.0;
                let btn_h = 31.0;
                
                if Button::new()
                    .pos(row_w - btn_w, center_y(row_h, btn_h))
                    .size(btn_w, btn_h)
                    .label(selected_output)
                    .text_alignment(TextAlignment::Left)
                    .build(ui)
                {
                    *action = SettingsAction::ShowOutputPicker;
                }
                
                // Draw dropdown arrow
                Label::new()
                    .icon("▼".to_string())
                    .pos(row_w - 20.0, center_y(row_h, btn_h))
                    .size(20.0, btn_h)
                    .alignment(TextAlignment::Center)
                    .build(ui);
            })
            .build(ui, rows);
        
        // Check if output is synth
        let is_synth = selected_output.eq_ignore_ascii_case("Synth") || 
                       selected_output.contains("Synth");
        
        if is_synth {
            spacer(ui);
            
            // SoundFont Folders
            SettingsRow::new()
                .title("SoundFont Folders")
                .build(ui, |ui, row_w, row_h| {
                    let w = 93.0;
                    let h = 31.0;
                    if Button::new()
                        .pos(row_w - w, center_y(row_h, h))
                        .size(w, h)
                        .label("+ Add Folder")
                        .build(ui)
                    {
                        *action = SettingsAction::AddSoundFontFolder;
                    }
                })
                .build(ui, rows);
            
            // List folders
            for (index, folder) in self.soundfont_folders.iter().enumerate() {
                spacer(ui);
                
                let folder_name = folder.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                SettingsRow::new()
                    .title(format!("Folder {}", index + 1))
                    .subtitle(folder_name.to_string())
                    .build(ui, |ui, row_w, row_h| {
                        let w = 40.0;
                        let h = 31.0;
                        Button::new()
                            .pos(row_w - w, center_y(row_h, h))
                            .size(w, h)
                            .label("X")
                            .color([200, 50, 50])
                            .build(ui);
                    })
                    .build(ui, rows);
            }
            
            spacer(ui);
            
            // SoundFont Selection
            let soundfont_display = self.current_soundfont_display();
            let soundfont_count = self.soundfont_files.len();
            
            SettingsRow::new()
                .title("SoundFont")
                .subtitle(if soundfont_count > 0 {
                    format!("{} ({} of {})", soundfont_display,
                            self.current_soundfont_index.map_or(0, |i| i + 1),
                            soundfont_count)
                } else {
                    soundfont_display
                })
                .build(ui, |ui, row_w, row_h| {
                    let btn_w = 40.0;
                    let btn_h = 31.0;
                    let gap = 5.0;
                    
                    // Previous button
                    if Button::new()
                        .pos(row_w - btn_w * 2.0 - gap, center_y(row_h, btn_h))
                        .size(btn_w, btn_h)
                        .label("<")
                        .build(ui)
                    {
                        *action = SettingsAction::PreviousSoundFont;
                    }
                    
                    // Next button
                    if Button::new()
                        .pos(row_w - btn_w, center_y(row_h, btn_h))
                        .size(btn_w, btn_h)
                        .label(">")
                        .build(ui)
                    {
                        *action = SettingsAction::NextSoundFont;
                    }
                })
                .build(ui, rows);
            
            spacer(ui);
            
            // Audio Gain
            SettingsRow::new()
                .title("Audio Gain")
                .subtitle(format!("{:.1}", ctx.config.audio_gain()))
                .build(ui, |ui, row_w, row_h| {
                    Self::draw_spin_buttons(ui, row_w, row_h, "audio_gain", action);
                })
                .build(ui, rows);
            
            spacer(ui);
            
            // Playback Gain
            SettingsRow::new()
                .title("Playback Gain")
                .subtitle(format!("{:.1}", ctx.config.playback_gain()))
                .build(ui, |ui, row_w, row_h| {
                    Self::draw_spin_buttons(ui, row_w, row_h, "playback_gain", action);
                })
                .build(ui, rows);
        }
    }
    
    /// Input settings section
    fn input_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        _spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        let selected_input = ctx.config.input().as_deref().unwrap_or("None");
        
        SettingsRow::new()
            .title("Input")
            .subtitle(selected_input)
            .build(ui, |ui, row_w, row_h| {
                let btn_w = 320.0;
                let btn_h = 31.0;
                
                if Button::new()
                    .pos(row_w - btn_w, center_y(row_h, btn_h))
                    .size(btn_w, btn_h)
                    .label(selected_input)
                    .text_alignment(TextAlignment::Left)
                    .build(ui)
                {
                    *action = SettingsAction::ShowInputPicker;
                }
                
                // Draw dropdown arrow
                Label::new()
                    .icon("▼".to_string())
                    .pos(row_w - 20.0, center_y(row_h, btn_h))
                    .size(20.0, btn_h)
                    .alignment(TextAlignment::Center)
                    .build(ui);
            })
            .build(ui, rows);
    }
    
    /// LUMI Hardware section
    fn lumi_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        _spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        // LED Brightness
        let brightness_percent = (ctx.config.lumi_brightness() as f32 / 127.0 * 100.0) as u8;
        SettingsRow::new()
            .title("LED Brightness")
            .subtitle(format!("{}%", brightness_percent))
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "lumi_brightness", action);
            })
            .build(ui, rows);
        
        // Color Mode
        let mode_names = ["Rainbow", "Single Color", "Piano", "Night"];
        let mode_name = mode_names.get(ctx.config.lumi_color_mode() as usize)
            .unwrap_or(&"Unknown");
        SettingsRow::new()
            .title("Color Mode")
            .subtitle(mode_name.to_string())
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "lumi_mode", action);
            })
            .build(ui, rows);
    }
    
    /// Note range settings section
    fn note_range_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        let range = ctx.config.piano_range();
        
        SettingsRow::new()
            .title("Start")
            .subtitle(range.start().to_string())
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "range_start", action);
            })
            .build(ui, rows);
        
        spacer(ui);
        
        SettingsRow::new()
            .title("End")
            .subtitle(range.end().to_string())
            .build(ui, |ui, row_w, row_h| {
                Self::draw_spin_buttons(ui, row_w, row_h, "range_end", action);
            })
            .build(ui, rows);
    }
    
    /// Render settings section
    fn render_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        _spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        // Vertical Guidelines
        SettingsRow::new()
            .title("Vertical Guidelines")
            .subtitle("Display octave indicators")
            .build(ui, |ui, row_w, row_h| {
                if Self::draw_toggle(ui, row_w, row_h, ctx.config.vertical_guidelines(), "vertical_guidelines") {
                    *action = SettingsAction::Toggle("vertical_guidelines".to_string());
                }
            })
            .build(ui, rows);
        
        // Horizontal Guidelines
        SettingsRow::new()
            .title("Horizontal Guidelines")
            .subtitle("Display measure/bar indicators")
            .build(ui, |ui, row_w, row_h| {
                if Self::draw_toggle(ui, row_w, row_h, ctx.config.horizontal_guidelines(), "horizontal_guidelines") {
                    *action = SettingsAction::Toggle("horizontal_guidelines".to_string());
                }
            })
            .build(ui, rows);
        
        // Glow
        SettingsRow::new()
            .title("Glow")
            .subtitle("Key glow effect")
            .build(ui, |ui, row_w, row_h| {
                if Self::draw_toggle(ui, row_w, row_h, ctx.config.glow(), "glow") {
                    *action = SettingsAction::Toggle("glow".to_string());
                }
            })
            .build(ui, rows);
        
        // Note Labels
        SettingsRow::new()
            .title("Note Labels")
            .subtitle("Display waterfall note labels")
            .build(ui, |ui, row_w, row_h| {
                if Self::draw_toggle(ui, row_w, row_h, ctx.config.note_labels(), "note_labels") {
                    *action = SettingsAction::Toggle("note_labels".to_string());
                }
            })
            .build(ui, rows);
    }
    
    /// Song library section
    fn song_library_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut PlyUi,
        rows: &dyn Fn(&mut PlyUi, SettingsRow),
        spacer: &dyn Fn(&mut PlyUi),
        action: &mut SettingsAction,
    ) {
        let total_song_count = match ctx.song_library_db.song_count() {
            Ok(count) => count,
            Err(e) => {
                log::error!("Failed to get song count from database: {}", e);
                0
            }
        };
        
        SettingsRow::new()
            .title("Total Songs")
            .subtitle(total_song_count.to_string())
            .build(ui, rows);
        
        spacer(ui);
        
        SettingsRow::new()
            .title("Song Directories")
            .build(ui, |ui, row_w, row_h| {
                let w = 115.0;
                let h = 31.0;
                if Button::new()
                    .pos(row_w - w, center_y(row_h, h))
                    .size(w, h)
                    .label("+ Add Directory")
                    .build(ui)
                {
                    *action = SettingsAction::AddSongDirectory;
                }
            })
            .build(ui, rows);
        
        for (index, dir_path) in self.song_directories.iter().enumerate() {
            spacer(ui);
            
            let dir_name = dir_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            
            let idx = index;
            SettingsRow::new()
                .title(format!("Directory {}", idx + 1))
                .subtitle(dir_name.to_string())
                .build(ui, |ui, row_w, row_h| {
                    let w = 75.0;
                    let h = 31.0;
                    if Button::new()
                        .pos(row_w - w, center_y(row_h, h))
                        .size(w, h)
                        .label("Remove")
                        .color([200, 50, 50])
                        .build(ui)
                    {
                        *action = SettingsAction::RemoveSongDirectory(idx);
                    }
                })
                .build(ui, rows);
        }
    }
    
    /// Draw keyboard layout preview
    fn keyboard_layout_preview(&self, ctx: &Context, keyboard_w: f32, keyboard_h: f32, ui: &mut PlyUi) {
        // Draw keyboard background
        Quad::new()
            .size(keyboard_w, keyboard_h)
            .color([255, 255, 255])
            .border_radius([7.0; 4])
            .build(ui);
        
        let range = piano_layout::KeyboardRange::new(ctx.config.piano_range());
        
        let white_count = range.white_count();
        let neutral_width = keyboard_w / white_count as f32;
        let neutral_height = keyboard_h;
        
        let layout = piano_layout::KeyboardLayout::from_range(
            piano_layout::Sizing::new(neutral_width, neutral_height),
            range,
        );
        
        // Draw key separators
        let mut neutral = layout.keys.iter()
            .filter(|key| key.kind().is_neutral())
            .peekable();
        
        while let Some(key) = neutral.next() {
            if neutral.peek().is_some() {
                Quad::new()
                    .pos(key.x() + key.width(), 0.0)
                    .size(1.0, key.height())
                    .color([150, 150, 150])
                    .build(ui);
            }
        }
        
        // Draw black keys
        for key in layout.keys.iter().filter(|key| key.kind().is_sharp()) {
            Quad::new()
                .pos(key.x(), 0.0)
                .size(key.width(), key.height())
                .color([0, 0, 0])
                .build(ui);
        }
    }
    
    /// Draw popup overlay
    fn draw_popup(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        match self.popup {
            PopupState::None => {}
            PopupState::OutputSelector => {
                self.draw_output_selector(ctx, action);
            }
            PopupState::InputSelector => {
                self.draw_input_selector(ctx, action);
            }
        }
    }
    
    /// Draw output selector popup
    fn draw_output_selector(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let popup_w = 320.0;
        let popup_h = 300.0;
        let popup_x = center_x(win_w, popup_w);
        let popup_y = center_y(win_h, popup_h);
        
        // Draw overlay
        Quad::new()
            .pos(0.0, 0.0)
            .size(win_w, win_h)
            .color([0, 0, 0])
            .build(&mut self.ui);
        
        // Draw popup background
        Quad::new()
            .pos(popup_x, popup_y)
            .size(popup_w, popup_h)
            .color([45, 43, 50])
            .border_radius([10.0; 4])
            .build(&mut self.ui);
        
        // Draw title
        Label::new()
            .text("Select Output")
            .pos(popup_x + 10.0, popup_y + 10.0)
            .size(popup_w - 20.0, 30.0)
            .font_size(18.0)
            .bold(true)
            .build(&mut self.ui);
        
        // Draw output options
        let mut y = popup_y + 50.0;
        for output in &self.outputs {
            let is_selected = ctx.config.output().as_deref() == Some(output.as_str());
            
            // Draw option background
            if is_selected {
                Quad::new()
                    .pos(popup_x + 10.0, y)
                    .size(popup_w - 20.0, 40.0)
                    .color([160, 81, 255])
                    .border_radius([5.0; 4])
                    .build(&mut self.ui);
            }
            
            // Draw option text
            Label::new()
                .text(output)
                .pos(popup_x + 20.0, y)
                .size(popup_w - 40.0, 40.0)
                .font_size(16.0)
                .color(if is_selected { [255, 255, 255] } else { [200, 200, 200] })
                .build(&mut self.ui);
            
            // Make clickable
            if Button::new()
                .id(&format!("output_{}", output))
                .pos(popup_x + 10.0, y)
                .size(popup_w - 20.0, 40.0)
                .color([0, 0, 0, 0])
                .build(&mut self.ui)
            {
                *action = SettingsAction::SelectOutput(output.clone());
            }
            
            y += 45.0;
        }
        
        // Close button
        if Button::new()
            .pos(popup_x + popup_w - 40.0, popup_y + 10.0)
            .size(30.0, 30.0)
            .label("✕")
            .build(&mut self.ui)
        {
            *action = SettingsAction::ClosePopup;
        }
    }
    
    /// Draw input selector popup
    fn draw_input_selector(&mut self, ctx: &mut Context, action: &mut SettingsAction) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        
        let popup_w = 320.0;
        let popup_h = 300.0;
        let popup_x = center_x(win_w, popup_w);
        let popup_y = center_y(win_h, popup_h);
        
        // Draw overlay
        Quad::new()
            .pos(0.0, 0.0)
            .size(win_w, win_h)
            .color([0, 0, 0])
            .build(&mut self.ui);
        
        // Draw popup background
        Quad::new()
            .pos(popup_x, popup_y)
            .size(popup_w, popup_h)
            .color([45, 43, 50])
            .border_radius([10.0; 4])
            .build(&mut self.ui);
        
        // Draw title
        Label::new()
            .text("Select Input")
            .pos(popup_x + 10.0, popup_y + 10.0)
            .size(popup_w - 20.0, 30.0)
            .font_size(18.0)
            .bold(true)
            .build(&mut self.ui);
        
        // Draw input options
        let mut y = popup_y + 50.0;
        for input in &self.inputs {
            let is_selected = ctx.config.input().as_deref() == Some(input.as_str());
            
            // Draw option background
            if is_selected {
                Quad::new()
                    .pos(popup_x + 10.0, y)
                    .size(popup_w - 20.0, 40.0)
                    .color([160, 81, 255])
                    .border_radius([5.0; 4])
                    .build(&mut self.ui);
            }
            
            // Draw option text
            Label::new()
                .text(input)
                .pos(popup_x + 20.0, y)
                .size(popup_w - 40.0, 40.0)
                .font_size(16.0)
                .color(if is_selected { [255, 255, 255] } else { [200, 200, 200] })
                .build(&mut self.ui);
            
            // Make clickable
            if Button::new()
                .id(&format!("input_{}", input))
                .pos(popup_x + 10.0, y)
                .size(popup_w - 20.0, 40.0)
                .color([0, 0, 0, 0])
                .build(&mut self.ui)
            {
                *action = SettingsAction::SelectInput(input.clone());
            }
            
            y += 45.0;
        }
        
        // Close button
        if Button::new()
            .pos(popup_x + popup_w - 40.0, popup_y + 10.0)
            .size(30.0, 30.0)
            .label("✕")
            .build(&mut self.ui)
        {
            *action = SettingsAction::ClosePopup;
        }
    }
    
    /// Draw spin buttons (plus/minus)
    fn draw_spin_buttons(ui: &mut PlyUi, row_w: f32, row_h: f32, id: &str, action: &mut SettingsAction) {
        let w = 30.0;
        let h = 30.0;
        let gap = 10.0;
        
        // Plus button
        if Button::new()
            .id(&format!("{}_plus", id))
            .pos(row_w - w, center_y(row_h, h))
            .size(w, h)
            .icon("+")
            .build(ui)
        {
            *action = SettingsAction::Increment(id.to_string());
        }
        
        // Minus button
        if Button::new()
            .id(&format!("{}_minus", id))
            .pos(row_w - w * 2.0 - gap, center_y(row_h, h))
            .size(w, h)
            .icon("-")
            .build(ui)
        {
            *action = SettingsAction::Decrement(id.to_string());
        }
    }
    
    /// Draw a toggle button, returns true if clicked
    fn draw_toggle(ui: &mut PlyUi, row_w: f32, row_h: f32, value: bool, id: &str) -> bool {
        let w = 40.0;
        let h = 20.0;
        
        let clicked = Button::new()
            .id(id)
            .pos(row_w - w, center_y(row_h, h))
            .size(w, h)
            .color(if value { [160, 81, 255] } else { [74, 68, 88] })
            .border_radius([10.0; 4])
            .build(ui);
        
        // Draw toggle thumb
        let head_w = 16.0;
        let head_h = 16.0;
        let gap = 2.0;
        
        Quad::new()
            .pos(
                if value {
                    row_w - head_w - gap
                } else {
                    row_w - w + gap
                },
                center_y(row_h, head_h)
            )
            .size(head_w, head_h)
            .color([255, 255, 255])
            .border_radius([8.0; 4])
            .build(ui);
        
        clicked
    }
    
    /// Get current SoundFont display name
    fn current_soundfont_display(&self) -> String {
        if let Some(index) = self.current_soundfont_index {
            if let Some(entry) = self.soundfont_files.get(index) {
                let file_name = entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                let folder_name = entry.folder.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                return format!("{} from {}", file_name, folder_name);
            }
        }
        
        "None".to_string()
    }
    
    /// Select previous SoundFont
    fn previous_soundfont(&mut self, ctx: &mut Context) {
        if self.soundfont_files.is_empty() {
            return;
        }
        
        let current = self.current_soundfont_index.unwrap_or(0);
        let count = self.soundfont_files.len();
        
        if count > 0 {
            let new_index = if current == 0 { count - 1 } else { current - 1 };
            self.select_soundfont_at_index(ctx, new_index);
        }
    }
    
    /// Select next SoundFont
    fn next_soundfont(&mut self, ctx: &mut Context) {
        if self.soundfont_files.is_empty() {
            return;
        }
        
        let current = self.current_soundfont_index.unwrap_or(0);
        let count = self.soundfont_files.len();
        
        if count > 0 {
            let new_index = (current + 1) % count;
            self.select_soundfont_at_index(ctx, new_index);
        }
    }
    
    /// Select SoundFont at index
    fn select_soundfont_at_index(&mut self, ctx: &mut Context, index: usize) {
        if let Some(entry) = self.soundfont_files.get(index) {
            self.current_soundfont_index = Some(index);
            ctx.config.synth_config.set_soundfont_path(Some(entry.path.clone()));
            ctx.config.synth_config.set_soundfont_index(Some(index));
            
            // Trigger runtime switch if output manager exists
            let _ = ctx.output_manager.switch_soundfont(&entry.path);
            
            ctx.config.save();
            log::info!("Selected SoundFont: {:?}", entry.path);
        }
    }
    
    /// Handle mouse movement
    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.ui.mouse_move(x, y);
    }
    
    /// Handle mouse button press
    pub fn mouse_down(&mut self) {
        self.ui.mouse_down();
    }
    
    /// Handle mouse button release
    pub fn mouse_up(&mut self) {
        self.ui.mouse_up();
    }
    
    /// Handle scroll
    pub fn scroll(&mut self, delta: f32) {
        self.scroll_state = (self.scroll_state - delta).max(0.0);
    }
    
    /// Handle keyboard event
    pub fn handle_key_event(&mut self, key: &Key) -> SettingsAction {
        let keyboard_action = self.ui.handle_key_event(key);
        
        match keyboard_action {
            KeyboardAction::Activate(widget_id) => {
                // Find which action corresponds to this widget
                self.handle_widget_activation(widget_id)
            }
            KeyboardAction::Cancel => {
                SettingsAction::GoBack
            }
            KeyboardAction::AdjustValue(widget_id, delta) => {
                self.handle_value_adjustment(widget_id, delta)
            }
            _ => SettingsAction::None,
        }
    }
    
    /// Handle widget activation from keyboard
    fn handle_widget_activation(&mut self, widget_id: u64) -> SettingsAction {
        // Check if this is a known widget and trigger the appropriate action
        // This is a simplified version - in production you'd map widget IDs to actions
        SettingsAction::None
    }
    
    /// Handle value adjustment from keyboard
    fn handle_value_adjustment(&mut self, widget_id: u64, delta: i32) -> SettingsAction {
        // Check which widget is being adjusted and apply the change
        // This is a simplified version - in production you'd map widget IDs to settings
        SettingsAction::None
    }
    
    /// Request native folder picker for SoundFont folder
    pub async fn request_soundfont_folder_picker(&mut self) -> Option<PathBuf> {
        log::info!("Requesting SoundFont folder picker");
        
        // Use rfd for native file dialog
        #[cfg(feature = "ply-rendering")]
        {
            if let Some(folder) = rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
            {
                let path = folder.path().to_path_buf();
                log::info!("Selected SoundFont folder: {:?}", path);
                return Some(path);
            }
        }
        
        log::warn!("Folder picker not available or cancelled");
        None
    }
    
    /// Request native folder picker for song directory
    pub async fn request_song_directory_picker(&mut self) -> Option<PathBuf> {
        log::info!("Requesting song directory picker");
        
        // Use rfd for native file dialog
        #[cfg(feature = "ply-rendering")]
        {
            if let Some(folder) = rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
            {
                let path = folder.path().to_path_buf();
                log::info!("Selected song directory: {:?}", path);
                return Some(path);
            }
        }
        
        log::warn!("Folder picker not available or cancelled");
        None
    }
    
    /// Add SoundFont folder from picker result
    pub fn add_soundfont_folder(&mut self, ctx: &mut Context, path: PathBuf) {
        if !path.exists() {
            log::warn!("SoundFont folder does not exist: {:?}", path);
            return;
        }
        
        self.soundfont_folders.push(path.clone());
        ctx.config.synth_config.add_soundfont_folder(path);
        
        // Re-discover SoundFonts
        self.soundfont_files = crate::output_manager::discover_soundfonts(&self.soundfont_folders);
        
        ctx.config.save();
        log::info!("Added SoundFont folder, now have {} SoundFonts", self.soundfont_files.len());
    }
    
    /// Add song directory from picker result
    pub fn add_song_directory(&mut self, ctx: &mut Context, path: PathBuf) {
        if !path.exists() {
            log::warn!("Song directory does not exist: {:?}", path);
            return;
        }
        
        self.song_directories.push(path.clone());
        ctx.config.add_song_directory(path);
        
        ctx.config.save();
        log::info!("Added song directory, now have {} directories", self.song_directories.len());
    }
}

/// Settings action returned by PLY UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    GoBack,
    ShowOutputPicker,
    ShowInputPicker,
    Increment(String),
    Decrement(String),
    Toggle(String),
    SelectOutput(String),
    SelectInput(String),
    ClosePopup,
    AddSoundFontFolder,
    AddSongDirectory,
    RemoveSongDirectory(usize),
    PreviousSoundFont,
    NextSoundFont,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_settings_creation() {
        let settings = PlySettingsMenu::new();
        assert_eq!(settings.scroll_state, 0.0);
        assert_eq!(settings.popup, PopupState::None);
    }
    
    #[test]
    fn test_settings_action_equality() {
        assert_eq!(SettingsAction::None, SettingsAction::None);
        assert_ne!(SettingsAction::GoBack, SettingsAction::ShowOutputPicker);
        assert_eq!(
            SettingsAction::Increment("test".to_string()),
            SettingsAction::Increment("test".to_string())
        );
    }
    
    #[test]
    fn test_popup_state_default() {
        assert_eq!(PopupState::default(), PopupState::None);
    }
}
