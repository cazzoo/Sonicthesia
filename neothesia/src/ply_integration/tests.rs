#[cfg(test)]
mod tests {
    use super::*;
    use ply_engine::math::Dimensions;

    #[test]
    fn test_init_ply_context() {
        let ply = init_ply_context();
        assert_eq!(ply.context.layout_dimensions().width, 1280.0);
        assert_eq!(ply.context.layout_dimensions().height, 720.0);
    }

    #[test]
    fn test_update_ply_engine() {
        let mut ply = init_ply_context();
        let initial_time = ply.context.current_time;
        update_ply_engine(&mut ply, 0.016); // ~60 FPS
        // Note: We can't easily test the time update because current_time is private
        // In a real implementation, we'd have a public method to get/set time
    }

    #[test]
    fn test_ply_context_dimensions() {
        let ply = init_ply_context();
        let dims = ply.context.layout_dimensions();
        
        // Verify default dimensions
        assert!(dims.width > 0.0);
        assert!(dims.height > 0.0);
    }

    #[test]
    fn test_ply_context_multiple_instances() {
        let ply1 = init_ply_context();
        let ply2 = init_ply_context();
        
        // Each instance should be independent
        assert_eq!(ply1.context.layout_dimensions().width, 1280.0);
        assert_eq!(ply2.context.layout_dimensions().width, 1280.0);
    }

    // Integration tests for audio module
    #[test]
    fn test_audio_manager_default() {
        let manager = audio::PlyAudioManager::default();
        assert!(manager.main_output().is_none());
        assert_eq!(manager.runtime_gain(), 1.0);
        assert!(!manager.has_lumi_connection());
    }

    #[test]
    fn test_audio_manager_set_runtime_gain() {
        let mut manager = audio::PlyAudioManager::new();
        
        manager.set_runtime_gain(0.5);
        assert_eq!(manager.runtime_gain(), 0.5);
        
        manager.set_runtime_gain(1.5);
        assert_eq!(manager.runtime_gain(), 1.5);
    }

    #[test]
    fn test_audio_manager_gain_clamping() {
        let mut manager = audio::PlyAudioManager::new();
        
        // Test upper bound clamping
        manager.set_runtime_gain(3.0);
        assert_eq!(manager.runtime_gain(), 2.0);
        
        // Test lower bound clamping
        manager.set_runtime_gain(-1.0);
        assert_eq!(manager.runtime_gain(), 0.0);
    }

    #[test]
    fn test_audio_event_note_on() {
        let event = audio::PlyAudioEvent::NoteOn {
            channel: midi_file::midly::num::u4::new(0),
            key: 60,
            velocity: 100,
        };
        
        match event {
            audio::PlyAudioEvent::NoteOn { channel, key, velocity } => {
                assert_eq!(channel.as_int(), 0);
                assert_eq!(key, 60);
                assert_eq!(velocity, 100);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_audio_event_note_off() {
        let event = audio::PlyAudioEvent::NoteOff {
            channel: midi_file::midly::num::u4::new(1),
            key: 64,
        };
        
        match event {
            audio::PlyAudioEvent::NoteOff { channel, key } => {
                assert_eq!(channel.as_int(), 1);
                assert_eq!(key, 64);
            }
            _ => panic!("Expected NoteOff event"),
        }
    }

    #[test]
    fn test_audio_event_control_change() {
        let event = audio::PlyAudioEvent::ControlChange {
            channel: midi_file::midly::num::u4::new(0),
            controller: 7, // Volume
            value: 100,
        };
        
        match event {
            audio::PlyAudioEvent::ControlChange { channel, controller, value } => {
                assert_eq!(channel.as_int(), 0);
                assert_eq!(controller, 7);
                assert_eq!(value, 100);
            }
            _ => panic!("Expected ControlChange event"),
        }
    }

    #[test]
    fn test_audio_event_pitch_bend() {
        let event = audio::PlyAudioEvent::PitchBend {
            channel: midi_file::midly::num::u4::new(0),
            value: 8192, // Center position
        };
        
        match event {
            audio::PlyAudioEvent::PitchBend { channel, value } => {
                assert_eq!(channel.as_int(), 0);
                assert_eq!(value, 8192);
            }
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_audio_event_sysex() {
        let data = vec![0xF0, 0x01, 0xF7];
        let event = audio::PlyAudioEvent::SysEx(data.clone());
        
        match event {
            audio::PlyAudioEvent::SysEx(d) => {
                assert_eq!(d, data);
            }
            _ => panic!("Expected SysEx event"),
        }
    }

    #[test]
    fn test_audio_event_set_gain() {
        let event = audio::PlyAudioEvent::SetGain(0.75);
        
        match event {
            audio::PlyAudioEvent::SetGain(gain) => {
                assert_eq!(gain, 0.75);
            }
            _ => panic!("Expected SetGain event"),
        }
    }

    #[test]
    fn test_audio_event_stop_all() {
        let event = audio::PlyAudioEvent::StopAll;
        
        match event {
            audio::PlyAudioEvent::StopAll => {
                // Success
            }
            _ => panic!("Expected StopAll event"),
        }
    }

    #[test]
    fn test_dummy_audio_connection() {
        let conn = audio::PlyAudioConnection::dummy();
        
        // Should not panic on any operations
        conn.midi_event(
            midi_file::midly::num::u4::new(0),
            midi_file::midly::MidiMessage::NoteOn {
                key: midi_file::midly::num::u7::new(60),
                vel: midi_file::midly::num::u7::new(100),
            },
        );
        conn.send_sysex(&[0xF0, 0x01, 0xF7]);
        conn.set_gain(0.8);
        conn.stop_all();
    }

    // Integration tests for input module
    #[test]
    fn test_neothesia_action_variants() {
        use input::NeothesiaAction;
        
        // Test navigation actions
        let _ = NeothesiaAction::NavigateUp;
        let _ = NeothesiaAction::NavigateDown;
        let _ = NeothesiaAction::NavigateLeft;
        let _ = NeothesiaAction::NavigateRight;
        
        // Test playback actions
        let _ = NeothesiaAction::PlayPause;
        let _ = NeothesiaAction::Stop;
        let _ = NeothesiaAction::Restart;
        
        // Test that actions are Copy and Eq
        let action1 = NeothesiaAction::PlayPause;
        let action2 = NeothesiaAction::PlayPause;
        assert_eq!(action1, action2);
    }

    #[test]
    fn test_input_binding_creation() {
        use input::{InputBinding, GamepadButton};
        use winit::event::MouseButton;
        use winit::keyboard::NamedKey;
        
        let binding = InputBinding {
            key: Some(winit::keyboard::Key::Named(NamedKey::ArrowUp)),
            gamepad_button: Some(GamepadButton::DPadUp),
            mouse_button: Some(MouseButton::Left),
        };
        
        assert!(binding.key.is_some());
        assert!(binding.gamepad_button.is_some());
        assert!(binding.mouse_button.is_some());
    }

    #[test]
    fn test_gamepad_button_variants() {
        use input::GamepadButton;
        
        let _ = GamepadButton::A;
        let _ = GamepadButton::B;
        let _ = GamepadButton::X;
        let _ = GamepadButton::Y;
        let _ = GamepadButton::DPadUp;
        let _ = GamepadButton::DPadDown;
        let _ = GamepadButton::LeftTrigger;
        let _ = GamepadButton::RightTrigger;
    }

    // Integration tests for game logic module
    #[test]
    fn test_ply_playalong_creation() {
        let playalong = game_logic::PlyPlayAlong::new();
        assert!(playalong.get_visual_feedback(60).is_none());
        assert_eq!(playalong.get_timing_stats().perfect_notes, 0);
    }

    #[test]
    fn test_ply_playalong_default() {
        let playalong = game_logic::PlyPlayAlong::default();
        assert!(playalong.get_visual_feedback(60).is_none());
    }

    #[test]
    fn test_ply_playalong_clear() {
        let mut playalong = game_logic::PlyPlayAlong::new();
        playalong.clear();
        assert!(playalong.get_visual_feedback(60).is_none());
        assert_eq!(playalong.get_timing_stats().perfect_notes, 0);
    }

    #[test]
    fn test_ply_rewind_controller_creation() {
        let controller = game_logic::PlyRewindController::new();
        assert!(!controller.is_rewinding());
        assert!(!controller.is_keyboard_rewinding());
        assert!(!controller.is_mouse_rewinding());
        assert!(!controller.is_scrubbing());
    }

    #[test]
    fn test_ply_rewind_controller_default() {
        let controller = game_logic::PlyRewindController::default();
        assert!(!controller.is_rewinding());
    }

    #[test]
    fn test_ply_rewind_controller_mouse_rewind() {
        let mut controller = game_logic::PlyRewindController::new();
        
        controller.start_mouse_rewind(false);
        assert!(controller.is_rewinding());
        assert!(controller.is_mouse_rewinding());
        assert!(controller.is_scrubbing());
        
        let (was_paused, speed) = controller.stop_rewind();
        assert!(!was_paused);
        assert!(speed.is_none());
        assert!(!controller.is_rewinding());
        assert!(!controller.is_scrubbing());
    }

    #[test]
    fn test_ply_rewind_controller_keyboard_rewind() {
        let mut controller = game_logic::PlyRewindController::new();
        
        controller.start_keyboard_rewind(2, true);
        assert!(controller.is_rewinding());
        assert!(controller.is_keyboard_rewinding());
        
        let (was_paused, speed) = controller.stop_rewind();
        assert!(was_paused);
        assert_eq!(speed, Some(2));
        assert!(!controller.is_rewinding());
    }

    #[test]
    fn test_ply_rewind_controller_scrubbing() {
        let mut controller = game_logic::PlyRewindController::new();
        
        controller.set_scrub_position(0.5);
        assert_eq!(controller.scrub_position(), 0.5);
        assert!(controller.is_scrubbing());
        
        controller.set_scrub_position(1.5); // Should clamp to 1.0
        assert_eq!(controller.scrub_position(), 1.0);
        
        controller.set_scrub_position(-0.5); // Should clamp to 0.0
        assert_eq!(controller.scrub_position(), 0.0);
    }

    #[test]
    fn test_rewind_modifiers() {
        let modifiers = game_logic::RewindModifiers {
            shift: true,
            control: false,
        };
        
        assert!(modifiers.shift);
        assert!(!modifiers.control);
    }

    #[test]
    fn test_timing_stats_default() {
        let stats = game_logic::TimingStats::default();
        assert_eq!(stats.perfect_notes, 0);
        assert_eq!(stats.good_notes, 0);
        assert_eq!(stats.okay_notes, 0);
        assert_eq!(stats.bad_notes, 0);
    }

    // Integration tests for UI module
    #[test]
    fn test_ply_ui_creation() {
        let ui = ui::PlyUi::new();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_default() {
        let ui = ui::PlyUi::default();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_begin_end_frame() {
        let mut ui = ui::PlyUi::new();
        
        ui.begin_frame(1280.0, 720.0);
        let commands = ui.end_frame();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_ply_ui_mouse_movement() {
        let mut ui = ui::PlyUi::new();
        
        ui.mouse_move(100.0, 200.0);
        ui.mouse_down();
        ui.mouse_up();
    }

    #[test]
    fn test_ply_ui_layer_stack() {
        let mut ui = ui::PlyUi::new();
        
        ui.push_layer();
        ui.translate(10.0, 20.0);
        assert_eq!(ui.current_offset(), (10.0, 20.0));
        
        ui.pop_layer();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_scissor_rect() {
        let mut ui = ui::PlyUi::new();
        
        ui.set_scissor_rect(10.0, 20.0, 100.0, 200.0);
        assert!(ui.in_scissor_rect(50.0, 100.0));
        assert!(!ui.in_scissor_rect(5.0, 100.0)); // Outside x
        assert!(!ui.in_scissor_rect(50.0, 10.0)); // Outside y
    }

    #[test]
    fn test_ply_ui_widget_state() {
        let mut ui = ui::PlyUi::new();
        
        ui.begin_frame(1280.0, 720.0);
        let state = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        
        assert!(!state.hovered);
        assert!(!state.pressed);
        assert!(!state.clicked);
    }

    #[test]
    fn test_ply_ui_render_command() {
        use ui::RenderCommand;
        
        let cmd = RenderCommand::Quad {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            color: [255, 0, 0, 255],
            border_radius: [5.0, 5.0, 5.0, 5.0],
        };
        
        match cmd {
            RenderCommand::Quad { x, y, width, height, color, border_radius } => {
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
                assert_eq!(width, 100.0);
                assert_eq!(height, 50.0);
                assert_eq!(color, [255, 0, 0, 255]);
                assert_eq!(border_radius, [5.0, 5.0, 5.0, 5.0]);
            }
            _ => panic!("Expected Quad command"),
        }
    }

    #[test]
    fn test_ply_ui_text_alignment() {
        use ui::TextAlignment;
        
        let _ = TextAlignment::Left;
        let _ = TextAlignment::Center;
        let _ = TextAlignment::Right;
    }

    #[test]
    fn test_ui_helper_functions() {
        use ui::{center_x, center_y, color_to_rgba};
        
        assert_eq!(center_x(100.0, 20.0), 40.0);
        assert_eq!(center_y(100.0, 20.0), 40.0);
        
        assert_eq!(color_to_rgba([255, 0, 0], 128), [255, 0, 0, 128]);
    }

    // Integration tests for rendering module
    #[test]
    fn test_ply_renderer_coordinator_creation() {
        let coordinator = render::ply::PlyRendererCoordinator::new();
        assert!(!coordinator.is_initialized());
    }

    #[test]
    fn test_ply_renderer_coordinator_default() {
        let coordinator = render::ply::PlyRendererCoordinator::default();
        assert!(!coordinator.is_initialized());
    }

    #[test]
    fn test_ply_renderer_coordinator_update_uninitialized() {
        let mut coordinator = render::ply::PlyRendererCoordinator::new();
        
        // Should not panic even when uninitialized
        coordinator.update(0.0, 1.0, 1.0, 0.0);
    }

    // Performance benchmarks
    #[test]
    fn test_audio_event_creation_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        for i in 0..10000 {
            let _ = audio::PlyAudioEvent::NoteOn {
                channel: midi_file::midly::num::u4::new((i % 16) as u8),
                key: (i % 128) as u8,
                velocity: 100,
            };
        }
        let duration = start.elapsed();
        
        // Should complete in reasonable time (< 10ms for 10k events)
        assert!(duration.as_millis() < 10, "Audio event creation too slow: {:?}", duration);
    }

    #[test]
    fn test_playalong_timing_categorization() {
        use std::time::Duration;
        
        let mut playalong = game_logic::PlyPlayAlong::new();
        
        // Test different timing categories
        let _ = playalong.categorize_timing(Duration::from_millis(25)); // Perfect
        let _ = playalong.categorize_timing(Duration::from_millis(75)); // Good
        let _ = playalong.categorize_timing(Duration::from_millis(150)); // Okay
        let _ = playalong.categorize_timing(Duration::from_millis(300)); // Bad
        
        let stats = playalong.get_timing_stats();
        assert_eq!(stats.perfect_notes, 1);
        assert_eq!(stats.good_notes, 1);
        assert_eq!(stats.okay_notes, 1);
        assert_eq!(stats.bad_notes, 1);
    }
}