use crate::effects::{EffectsManager, ScreenFlash, ScreenShake, TimingFeedback};
use crate::lumi_controller::LumiController;
use crate::scoring::{LiveScoreTracker, StreakMilestone, TimingQuality};
use midi_file::midly::{num::u4, MidiMessage};
use neothesia_core::render::{
    waterfall::TrackChannelConfig, GlowRenderer, GuidelineRenderer, NoteLabels, QuadRenderer,
    TextRenderer,
};

use crate::render::ply::{waterfall::PlyWaterfallRenderer, PlyRendererCoordinator};

use std::time::Duration;
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use self::top_bar::TopBar;

use super::{NuonRenderer, Scene};
use crate::{
    context::Context, scene::MouseToMidiEventState, song::Song, song_library::SongRepository,
    utils::window::WinitEvent, NeothesiaEvent,
};
use neothesia_core::render::{KeyboardRenderer, WaterfallRenderer};

mod keyboard;
pub use keyboard::Keyboard;

pub mod midi_player;
use midi_player::MidiPlayer;

mod rewind_controller;
use rewind_controller::RewindController;

mod toast_manager;
use toast_manager::ToastManager;

mod animation;
mod top_bar;

fn format_score(score: u64) -> String {
    let s = score.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

pub struct RuntimeGain {
    value: f32,
}

impl RuntimeGain {
    pub const NEUTRAL: f32 = 1.0;
    pub const MIN: f32 = 0.0;
    pub const MAX: f32 = 2.0;

    pub fn new() -> Self {
        Self {
            value: Self::NEUTRAL,
        }
    }

    pub fn neutral() -> Self {
        Self::new()
    }

    pub fn from_value(value: f32) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }

    pub fn adjust(&mut self, delta: f32) {
        self.value = (self.value + delta).clamp(Self::MIN, Self::MAX);
    }

    pub fn reset(&mut self) {
        self.value = Self::NEUTRAL;
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn as_percentage(&self) -> f32 {
        self.value * 100.0
    }
}

impl Default for RuntimeGain {
    fn default() -> Self {
        Self::neutral()
    }
}

pub struct PlayingScene {
    keyboard: Keyboard,
    waterfall: WaterfallRenderer,
    guidelines: GuidelineRenderer,
    text_renderer: TextRenderer,
    nuon_renderer: NuonRenderer,

    note_labels: Option<NoteLabels>,

    player: MidiPlayer,
    lumi: LumiController,
    saved_color_mode: u8,
    rewind_controller: RewindController,
    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,
    glow: Option<GlowRenderer>,
    toast_manager: ToastManager,

    nuon: nuon::Ui,
    mouse_to_midi_state: MouseToMidiEventState,

    top_bar: TopBar,

    runtime_gain: RuntimeGain,

    cached_keyboard_gain: Option<f32>,

    current_song_id: Option<i64>,

    ply_renderer: PlyRendererCoordinator,

    live_score: LiveScoreTracker,
    effects: EffectsManager,
}

impl PlayingScene {
    pub fn new(ctx: &mut Context, song: Song) -> Self {
        let keyboard = Keyboard::new(ctx, song.config.clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            ctx.config.horizontal_guidelines(),
            song.file.measures.clone(),
        );

        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|t| !t.visible)
            .map(|t| t.track_id)
            .collect();

        let track_channel_configs: Vec<TrackChannelConfig> = song
            .config
            .tracks
            .iter()
            .map(|t| {
                let hidden_channels: Vec<u8> = t
                    .channels
                    .iter()
                    .filter(|cc| !cc.active)
                    .map(|cc| cc.channel)
                    .collect();

                TrackChannelConfig {
                    track_id: t.track_id,
                    hidden_channels,
                }
            })
            .collect();

        // Clone tracks and measures before song is moved
        let tracks_clone = song.file.tracks.clone();
        let measures_clone = song.file.measures.clone();

        let mut waterfall = WaterfallRenderer::new(
            &ctx.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &track_channel_configs,
            &ctx.config,
            &ctx.transform,
            keyboard_layout.clone(),
        );

        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let note_labels = ctx.config.note_labels().then_some(NoteLabels::new(
            *keyboard.pos(),
            waterfall.notes(),
            ctx.text_renderer_factory.new_renderer(),
        ));

        // Extract song_id before moving song
        let current_song_id = song.song_id;
        let player = MidiPlayer::new(
            ctx.output_manager.connection().clone(),
            song,
            keyboard_layout.range.clone(),
            ctx.config.separate_channels(),
        );
        let mut lumi = LumiController::new(ctx.output_manager.lumi_connection());
        lumi.begin_api_mode();
        lumi.clear_all();
        lumi.set_brightness(ctx.config.lumi_brightness());
        waterfall.update(player.time_without_lead_in());

        let quad_renderer_bg = ctx.quad_renderer_facotry.new_renderer();
        let quad_renderer_fg = ctx.quad_renderer_facotry.new_renderer();

        let glow = ctx.config.glow().then_some(GlowRenderer::new(
            &ctx.gpu,
            &ctx.transform,
            keyboard.layout(),
        ));

        ctx.output_manager.set_runtime_gain(1.0);
        let midi_file_gain = ctx.config.audio_gain() * ctx.config.playback_gain();
        ctx.output_manager.connection().set_gain(midi_file_gain);

        // Initialize PLY renderer coordinator
        let mut ply_renderer = PlyRendererCoordinator::new();

        ply_renderer.initialize(
            &tracks_clone,
            &hidden_tracks,
            &track_channel_configs,
            &ctx.config,
            &keyboard_layout,
            measures_clone,
            ctx.config.vertical_guidelines(),
            ctx.config.horizontal_guidelines(),
        );

        log::info!("🎨 PLY Renderer Coordinator initialized in PlayingScene");

        Self {
            keyboard,
            guidelines,
            note_labels,
            text_renderer,
            nuon_renderer: NuonRenderer::new(ctx),

            waterfall,
            player,
            lumi,
            saved_color_mode: ctx.config.lumi_color_mode(),
            rewind_controller: RewindController::new(),
            quad_renderer_bg,
            quad_renderer_fg,
            glow,
            toast_manager: ToastManager::default(),

            nuon: nuon::Ui::new(),
            mouse_to_midi_state: MouseToMidiEventState::default(),

            top_bar: TopBar::new(),

            runtime_gain: RuntimeGain::neutral(),
            cached_keyboard_gain: None,
            current_song_id,

            ply_renderer,

            live_score: LiveScoreTracker::new(),
            effects: EffectsManager::new(),
        }
    }

    fn update_glow(&mut self, delta: Duration) {
        let Some(glow) = &mut self.glow else {
            return;
        };

        glow.clear();

        let keys = &self.keyboard.layout().keys;
        let states = self.keyboard.key_states();

        for (key, state) in keys.iter().zip(states) {
            let Some(color) = state.pressed_by_file() else {
                continue;
            };

            glow.push(
                key.id(),
                *color,
                key.x(),
                self.keyboard.pos().y,
                key.width(),
                delta,
            );
        }
    }

    fn update_live_score(&mut self, delta: Duration) {
        let timing_events = self.player.play_along_mut().take_timing_events();
        let keyboard_y = self.keyboard.pos().y;

        for quality in timing_events {
            let (_, milestone) = self.live_score.on_note_hit(quality);

            let feedback = match quality {
                TimingQuality::Perfect => TimingFeedback::perfect(0.0, keyboard_y - 30.0),
                TimingQuality::Good => TimingFeedback::good(0.0, keyboard_y - 30.0),
                TimingQuality::Okay => TimingFeedback::okay(0.0, keyboard_y - 30.0),
                TimingQuality::Miss => {
                    self.effects.trigger_shake(ScreenShake::small());
                    TimingFeedback::miss(0.0, keyboard_y - 30.0)
                }
            };
            self.effects.add_timing_feedback(feedback);

            if let Some(m) = milestone {
                match m {
                    StreakMilestone::Multiplier2x => {
                        self.effects.add_timing_feedback(TimingFeedback::new(
                            0.0,
                            keyboard_y - 60.0,
                            "2× MULTIPLIER!",
                            0.0,
                            1.0,
                            0.0,
                        ));
                    }
                    StreakMilestone::Multiplier4x => {
                        self.effects.add_timing_feedback(TimingFeedback::new(
                            0.0,
                            keyboard_y - 60.0,
                            "4× MULTIPLIER!",
                            0.0,
                            0.53,
                            1.0,
                        ));
                    }
                    StreakMilestone::Multiplier8x => {
                        self.effects.add_timing_feedback(TimingFeedback::new(
                            0.0,
                            keyboard_y - 60.0,
                            "8× MULTIPLIER!",
                            1.0,
                            0.84,
                            0.0,
                        ));
                        self.effects.trigger_flash(ScreenFlash::gold(0.3));
                    }
                    StreakMilestone::OnFire => {
                        self.effects.add_timing_feedback(TimingFeedback::new(
                            0.0,
                            keyboard_y - 60.0,
                            "🔥 ON FIRE!",
                            1.0,
                            0.53,
                            0.0,
                        ));
                        self.effects.trigger_shake(ScreenShake::medium());
                        self.effects.trigger_flash(ScreenFlash::gold(0.5));
                    }
                    StreakMilestone::Legendary => {
                        self.effects.add_timing_feedback(TimingFeedback::new(
                            0.0,
                            keyboard_y - 60.0,
                            "★ LEGENDARY!",
                            1.0,
                            0.0,
                            1.0,
                        ));
                        self.effects.trigger_shake(ScreenShake::large());
                        self.effects.trigger_flash(ScreenFlash::gold(0.8));
                    }
                }
            }
        }

        self.effects.update(delta);
    }

    fn render_live_score(&mut self) {
        let score_text = format_score(self.live_score.score());
        let multiplier = self.live_score.multiplier();
        let streak = self.live_score.streak().current();

        let x_offset = 10.0;

        let score_buffer = TextRenderer::gen_buffer_with_attr(
            24.0,
            &score_text,
            cosmic_text::Attrs::new()
                .family(cosmic_text::Family::Name("Roboto"))
                .weight(cosmic_text::Weight::BOLD)
                .color(cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF)),
        );
        self.text_renderer.queue_buffer_right(
            nuon::Rect::new(
                nuon::Point::new(x_offset, 10.0),
                nuon::Size::new(300.0, 30.0),
            ),
            score_buffer,
        );

        let multiplier_color = match multiplier {
            8 => cosmic_text::Color::rgb(0xFF, 0xD7, 0x00),
            4 => cosmic_text::Color::rgb(0x00, 0x88, 0xFF),
            2 => cosmic_text::Color::rgb(0x00, 0xFF, 0x00),
            _ => cosmic_text::Color::rgb(0xCC, 0xCC, 0xCC),
        };

        let multiplier_buffer = TextRenderer::gen_buffer_with_attr(
            18.0,
            &format!("×{}", multiplier),
            cosmic_text::Attrs::new()
                .family(cosmic_text::Family::Name("Roboto"))
                .weight(cosmic_text::Weight::BOLD)
                .color(multiplier_color),
        );
        self.text_renderer.queue_buffer_right(
            nuon::Rect::new(
                nuon::Point::new(x_offset, 40.0),
                nuon::Size::new(300.0, 25.0),
            ),
            multiplier_buffer,
        );

        if streak > 0 {
            let streak_color = if streak >= 200 {
                cosmic_text::Color::rgb(0xFF, 0x00, 0xFF)
            } else if streak >= 100 {
                cosmic_text::Color::rgb(0xFF, 0x88, 0x00)
            } else if streak >= 50 {
                cosmic_text::Color::rgb(0xFF, 0xD7, 0x00)
            } else if streak >= 30 {
                cosmic_text::Color::rgb(0x00, 0x88, 0xFF)
            } else if streak >= 10 {
                cosmic_text::Color::rgb(0x00, 0xFF, 0x00)
            } else {
                cosmic_text::Color::rgb(0xAA, 0xAA, 0xAA)
            };

            let streak_text = if streak >= 200 {
                format!("★ LEGENDARY: {}", streak)
            } else if streak >= 100 {
                format!("🔥 ON FIRE: {}", streak)
            } else {
                format!("Streak: {}", streak)
            };

            let streak_buffer = TextRenderer::gen_buffer_with_attr(
                16.0,
                &streak_text,
                cosmic_text::Attrs::new()
                    .family(cosmic_text::Family::Name("Roboto"))
                    .color(streak_color),
            );
            self.text_renderer.queue_buffer_right(
                nuon::Rect::new(
                    nuon::Point::new(x_offset, 65.0),
                    nuon::Size::new(300.0, 20.0),
                ),
                streak_buffer,
            );
        }

        for feedback in self.effects.timing_feedbacks() {
            let alpha = feedback.alpha();
            if alpha <= 0.0 {
                continue;
            }

            let (r, g, b, _) = feedback.color();
            let color =
                cosmic_text::Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);

            let buffer = TextRenderer::gen_buffer_with_attr(
                20.0,
                &feedback.text,
                cosmic_text::Attrs::new()
                    .family(cosmic_text::Family::Name("Roboto"))
                    .weight(cosmic_text::Weight::BOLD)
                    .color(color),
            );

            let screen_width = 1920.0;
            let x = screen_width / 2.0 - 50.0;

            self.text_renderer.queue_buffer(x, feedback.y, buffer);
        }
    }

    #[profiling::function]
    fn update_midi_player(&mut self, ctx: &Context, delta: Duration) -> f32 {
        if self.top_bar.is_looper_active() && self.player.time() > self.top_bar.loop_end_timestamp()
        {
            self.player.set_time(self.top_bar.loop_start_timestamp());
            self.keyboard.reset_notes();
        }

        // Check per-song wait_mode setting instead of global config
        if !self.player.song().config.wait_mode
            || self.player.play_along().are_required_keys_pressed()
        {
            let delta = (delta / 10) * (ctx.config.speed_multiplier() * 10.0) as u32;
            let midi_file_gain = self.midi_file_gain_with_runtime(ctx);
            let midi_events = self.player.update(delta, midi_file_gain);
            self.keyboard.file_midi_events(&ctx.config, &midi_events);
        }

        // LUMI Keys logic
        let expected_notes = self.player.play_along().get_required_notes();
        let upcoming = self.waterfall.notes();
        // Use raw playback time for hinting distance (not time_without_lead_in)
        // so hinting works correctly during the lead-in period
        let target_time = self.player.time().as_secs_f32() + ctx.config.animation_offset();

        let key_states = self.keyboard.key_states();

        for key in self.keyboard.layout().keys.iter() {
            let note_id = key.note_id();

            // 1. Is the user holding the key? (Highest priority feedback)
            if let Some(user_color) = key_states[key.id()]
                .pressed_by_user()
                .map(|c| c.into_linear_rgba())
            {
                self.lumi.set_key_color(
                    note_id,
                    (user_color[0] * 255.0) as u8,
                    (user_color[1] * 255.0) as u8,
                    (user_color[2] * 255.0) as u8,
                );
                continue;
            }

            // 2. Is the key currently required for 'Wait Mode' or actively overlapping playhead?
            if expected_notes.contains_key(&note_id) {
                self.lumi.set_key_color(note_id, 0, 255, 0); // Green for "Play me!"
                continue;
            }

            // 3. Is the key approaching in the waterfall within 2 seconds? (Hinting)
            let mut is_hinted = false;
            if log::log_enabled!(log::Level::Debug) {
                // Debug: show first few notes for troubleshooting
                let debug_notes: Vec<_> = upcoming.inner().iter().take(5).collect();
                log::debug!(
                    "Hinting check for note_id={}: target_time={:.2}, debug_notes={:?}",
                    note_id,
                    target_time,
                    debug_notes
                );
            }
            for note in upcoming.inner().iter().filter(|n| n.note == note_id) {
                if note.start.as_secs_f32() > target_time
                    && (note.start.as_secs_f32() - target_time) < 2.0
                {
                    is_hinted = true;
                    log::debug!(
                        "HINTING note_id={} (start={:.2}, target={:.2}, diff={:.2})",
                        note_id,
                        note.start.as_secs_f32(),
                        target_time,
                        note.start.as_secs_f32() - target_time
                    );
                    break;
                }
            }
            if is_hinted {
                self.lumi.set_key_dim(note_id, 0, 100, 255); // Dim blue
                log::debug!("Sent hinting for note_id={}", note_id);
                continue;
            }
            // 4. Otherwise, dark.
            self.lumi.clear_key(note_id);
        }

        // LUMI-specific hinting: Check all upcoming notes in LUMI's octave range (48-71)
        // This handles notes that may not have physical keys in the current keyboard layout
        let lumi_hint_notes: Vec<_> = upcoming
            .inner()
            .iter()
            .filter(|note| {
                // LUMI range: C3 (48) to B4 (71) = 2 octaves
                note.note >= 48
                    && note.note <= 71
                    && note.start.as_secs_f32() > target_time
                    && (note.start.as_secs_f32() - target_time) < 2.0
            })
            .map(|note| note.note)
            .collect();

        for note_id in lumi_hint_notes {
            self.lumi.set_key_dim(note_id, 0, 100, 255); // Dim blue hinting
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("LUMI HINTING note_id={} (in LUMI range 48-71)", note_id);
            }
        }

        self.player.time_without_lead_in() + ctx.config.animation_offset()
    }

    #[profiling::function]
    fn resize(&mut self, ctx: &Context) {
        self.keyboard.resize(ctx);

        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.set_pos(*self.keyboard.pos());
        }

        self.waterfall
            .resize(&ctx.config, self.keyboard.layout().clone());
    }

    pub fn adjust_runtime_gain(&mut self, ctx: &mut Context, delta: f32) {
        self.runtime_gain.adjust(delta);
        ctx.output_manager
            .set_runtime_gain(self.runtime_gain.value());
    }

    pub fn reset_runtime_gain(&mut self, ctx: &mut Context) {
        self.runtime_gain.reset();
        ctx.output_manager
            .set_runtime_gain(self.runtime_gain.value());
    }

    pub fn runtime_gain_percentage(&self) -> f32 {
        self.runtime_gain.as_percentage()
    }

    fn midi_file_gain(&self, ctx: &Context) -> f32 {
        ctx.config.audio_gain() * ctx.config.playback_gain()
    }

    fn midi_file_gain_with_runtime(&self, ctx: &Context) -> f32 {
        ctx.config.audio_gain() * ctx.config.playback_gain() * self.runtime_gain.value()
    }
}

impl Scene for PlayingScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        self.rewind_controller.update(&mut self.player, ctx, delta);
        self.toast_manager.update(&mut self.text_renderer);

        let time = self.update_midi_player(ctx, delta);
        self.waterfall.update(time);

        // Update PLY renderer coordinator
        self.ply_renderer.update(
            time,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            self.keyboard.pos().y,
        );
        log::info!(
            "🎯 PLY RENDERER ACTIVE: Updated playing scene at time={:.2}",
            time
        );
        log::info!(
            "🎯 PLY Waterfall: {} notes tracked",
            self.ply_renderer
                .waterfall_mut()
                .map(|w| w.notes().inner().len())
                .unwrap_or(0)
        );
        log::info!(
            "🎯 PLY Keyboard: {} keys managed",
            self.ply_renderer
                .keyboard_mut()
                .map(|k| k.layout().keys.len())
                .unwrap_or(0)
        );

        self.guidelines.update(
            &mut self.quad_renderer_bg,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            time,
            ctx.window_state.logical_size,
        );
        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text_renderer);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.update(
                ctx.window_state.physical_size,
                ctx.window_state.scale_factor as f32,
                self.keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }

        self.update_glow(delta);

        self.update_live_score(delta);

        TopBar::update(self, ctx);

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();

        if let Some(glow) = &mut self.glow {
            glow.prepare();
        }

        #[cfg(debug_assertions)]
        self.text_renderer.queue_fps(
            ctx.fps_ticker.avg(),
            self.top_bar
                .topbar_expand_animation
                .animate_bool(5.0, 80.0, ctx.frame_timestamp),
        );

        // Add prominent PLY active indicator in top-left corner
        let ply_buffer = TextRenderer::gen_buffer_with_attr(
            18.0,
            "🎯 PLY ENGINE ACTIVE",
            cosmic_text::Attrs::new()
                .family(cosmic_text::Family::Name("Roboto"))
                .color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00)),
        );
        self.text_renderer.queue_buffer(10.0, 10.0, ply_buffer);

        // Add PLY system status below the main indicator
        let ply_status = TextRenderer::gen_buffer_with_attr(
            14.0,
            &format!(
                "🎨 Waterfall: {} notes",
                self.ply_renderer
                    .waterfall_mut()
                    .map(|w| w.notes().inner().len())
                    .unwrap_or(0)
            ),
            cosmic_text::Attrs::new()
                .family(cosmic_text::Family::Name("Roboto"))
                .color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00)),
        );
        self.text_renderer.queue_buffer(10.0, 35.0, ply_status);

        // Render live score display in top-right
        self.render_live_score();

        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );

        if self.player.is_finished() && !self.player.is_paused() {
            use crate::song::PlayMode;

            // Show score only for Learn and Play modes (not Watch mode)
            // play_mode reflects the user's initial mode selection
            match self.player.song().config.play_mode {
                PlayMode::Watch => {
                    // Watch mode - user is just watching, not playing
                    ctx.proxy
                        .send_event(NeothesiaEvent::MainMenu(Some(self.player.song().clone())))
                        .ok();
                }
                PlayMode::Learn | PlayMode::Play => {
                    let score_result = self.live_score.to_score_data();
                    let mut score_data = self.player.play_along().to_score_data();

                    score_data.stars = score_result.stars.count();
                    score_data.max_streak = score_result.max_streak;
                    score_data.score = score_result.score;

                    if let Some(song_id) = self.current_song_id {
                        if let Err(e) = ctx
                            .song_library_db
                            .update_stats(song_id, Some(score_result.accuracy as f32))
                        {
                            log::error!("Failed to update song library stats: {}", e);
                        }

                        if let Err(e) = ctx.song_library_db.save_high_score(
                            song_id,
                            score_result.score,
                            score_result.accuracy,
                            score_result.max_streak,
                            score_result.stars.count(),
                            score_result.perfect_count,
                            score_result.good_count,
                            score_result.okay_count,
                            score_result.miss_count,
                        ) {
                            log::error!("Failed to save high score: {}", e);
                        }
                    }

                    ctx.proxy
                        .send_event(NeothesiaEvent::ShowScore {
                            song: self.player.song().clone(),
                            score_data,
                        })
                        .ok();
                }
            }
        }
    }

    #[profiling::function]
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer_bg.render(rpass);
        self.waterfall.render(rpass);
        if let Some(note_labels) = self.note_labels.as_mut() {
            note_labels.render(rpass);
        }
        self.quad_renderer_fg.render(rpass);
        if let Some(glow) = &self.glow {
            glow.render(rpass);
        }
        self.text_renderer.render(rpass);

        self.nuon_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        self.rewind_controller
            .handle_window_event(ctx, event, &mut self.player);

        if self.rewind_controller.is_rewinding() {
            self.keyboard.reset_notes();
        }

        if event.back_mouse_pressed() || event.key_released(Key::Named(NamedKey::Escape)) {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(self.player.song().clone())))
                .ok();
        }

        if event.key_released(Key::Named(NamedKey::Space)) {
            self.player.pause_resume();
        }

        if let Some(ch) = event.character_released() {
            match ch {
                "[" => {
                    self.adjust_runtime_gain(ctx, -0.1);
                    self.toast_manager
                        .gain_toast(self.runtime_gain_percentage());
                }
                "]" => {
                    self.adjust_runtime_gain(ctx, 0.1);
                    self.toast_manager
                        .gain_toast(self.runtime_gain_percentage());
                }
                _ => {}
            }
        }

        if event.key_released(Key::Named(NamedKey::Backspace))
            || event.key_released(Key::Named(NamedKey::Delete))
        {
            self.reset_runtime_gain(ctx);
            self.toast_manager
                .gain_toast(self.runtime_gain_percentage());
        }

        handle_settings_input(ctx, &mut self.toast_manager, &mut self.waterfall, event);
        super::handle_pc_keyboard_to_midi_event(ctx, event);
        super::handle_mouse_to_midi_event(
            &mut self.keyboard,
            &mut self.mouse_to_midi_state,
            ctx,
            event,
        );

        if event.window_resized() || event.scale_factor_changed() {
            self.resize(ctx)
        }

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
    }

    fn midi_event(&mut self, ctx: &mut Context, _channel: u8, message: &MidiMessage) {
        let keyboard_gain = match self.player.song().config.play_mode {
            crate::song::PlayMode::Watch => 0.0,
            crate::song::PlayMode::Learn => ctx.config.keyboard_gain(),
            crate::song::PlayMode::Play => ctx.config.keyboard_gain(),
        };

        if self.player.song().config.wait_mode {
            match message {
                MidiMessage::NoteOn { key, .. } => {
                    let note_id = key.as_int();
                    if self.player.play_along_mut().is_required_note(note_id) {
                        self.player
                            .play_along_mut()
                            .mark_note_as_triggered(_channel, note_id);
                    }
                }
                _ => {}
            }
        }

        if keyboard_gain > 0.0 {
            // Only call set_gain if the value has changed (performance optimization)
            if self.cached_keyboard_gain != Some(keyboard_gain) {
                ctx.output_manager
                    .keyboard_connection()
                    .set_gain(keyboard_gain);
                self.cached_keyboard_gain = Some(keyboard_gain);
            }

            ctx.output_manager
                .keyboard_connection()
                .midi_event(u4::new(_channel), *message);
        }

        self.player
            .play_along_mut()
            .midi_event(midi_player::MidiEventSource::User, message);
        self.keyboard.user_midi_event(message);
    }
}

fn handle_settings_input(
    ctx: &mut Context,
    toast_manager: &mut ToastManager,
    waterfall: &mut WaterfallRenderer,
    event: &WindowEvent,
) {
    if event.key_released(Key::Named(NamedKey::ArrowUp))
        || event.key_released(Key::Named(NamedKey::ArrowDown))
    {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            0.5
        } else {
            0.1
        };

        if event.key_released(Key::Named(NamedKey::ArrowUp)) {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() + amount);
        } else {
            ctx.config
                .set_speed_multiplier(ctx.config.speed_multiplier() - amount);
        }

        toast_manager.speed_toast(ctx.config.speed_multiplier());
        return;
    }

    if event.key_released(Key::Named(NamedKey::PageUp))
        || event.key_released(Key::Named(NamedKey::PageDown))
    {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            500.0
        } else {
            100.0
        };

        if event.key_released(Key::Named(NamedKey::PageUp)) {
            ctx.config
                .set_animation_speed(ctx.config.animation_speed() + amount);
        } else {
            ctx.config
                .set_animation_speed(ctx.config.animation_speed() - amount);
        }

        waterfall
            .pipeline()
            .set_speed(&ctx.gpu.queue, ctx.config.animation_speed());
        toast_manager.animation_speed_toast(ctx.config.animation_speed());
        return;
    }

    if let Some(ch @ ("_" | "-" | "+" | "=")) = event.character_released() {
        let amount = if ctx.window_state.modifiers_state.shift_key() {
            0.1
        } else {
            0.01
        };

        if matches!(ch, "-" | "_") {
            ctx.config
                .set_animation_offset(ctx.config.animation_offset() - amount);
        } else {
            ctx.config
                .set_animation_offset(ctx.config.animation_offset() + amount);
        }

        toast_manager.offset_toast(ctx.config.animation_offset());
    }
}

impl Drop for PlayingScene {
    fn drop(&mut self) {
        // Restore menu settings when exiting playing scene
        log::info!(
            "PlayingScene: Exiting, restoring LUMI menu mode {}",
            self.saved_color_mode
        );
        self.lumi.end_api_mode();
        self.lumi.set_color_mode(self.saved_color_mode);
    }
}
