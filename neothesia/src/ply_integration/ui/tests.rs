#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{PlyUi, RenderCommand, TextAlignment, WidgetState, center_x, center_y, color_to_rgba};

    // Tests for PlyUi basic functionality
    #[test]
    fn test_ply_ui_creation() {
        let ui = PlyUi::new();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_default() {
        let ui = PlyUi::default();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_begin_end_frame() {
        let mut ui = PlyUi::new();
        
        ui.begin_frame(1280.0, 720.0);
        let commands = ui.end_frame();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_ply_ui_multiple_frames() {
        let mut ui = PlyUi::new();
        
        for i in 0..10 {
            ui.begin_frame(1280.0, 720.0);
            ui.mouse_move(i as f32 * 10.0, i as f32 * 10.0);
            let _commands = ui.end_frame();
        }
        
        // Should not panic
    }

    // Tests for mouse input handling
    #[test]
    fn test_ply_ui_mouse_movement() {
        let mut ui = PlyUi::new();
        
        ui.mouse_move(100.0, 200.0);
        ui.mouse_down();
        ui.mouse_up();
        
        // Should not panic
    }

    #[test]
    fn test_ply_ui_mouse_click_sequence() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Simulate click
        ui.mouse_move(100.0, 100.0);
        ui.mouse_down();
        ui.mouse_up();
        
        let _commands = ui.end_frame();
    }

    #[test]
    fn test_ply_ui_mouse_drag() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Simulate drag
        ui.mouse_move(100.0, 100.0);
        ui.mouse_down();
        ui.mouse_move(150.0, 150.0);
        ui.mouse_move(200.0, 200.0);
        ui.mouse_up();
        
        let _commands = ui.end_frame();
    }

    // Tests for layout management
    #[test]
    fn test_ply_ui_layer_stack() {
        let mut ui = PlyUi::new();
        
        ui.push_layer();
        ui.translate(10.0, 20.0);
        assert_eq!(ui.current_offset(), (10.0, 20.0));
        
        ui.push_layer();
        ui.translate(5.0, 10.0);
        assert_eq!(ui.current_offset(), (15.0, 30.0));
        
        ui.pop_layer();
        assert_eq!(ui.current_offset(), (10.0, 20.0));
        
        ui.pop_layer();
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_layer_stack_underflow() {
        let mut ui = PlyUi::new();
        
        // Should not panic when popping more layers than pushed
        ui.pop_layer();
        ui.pop_layer();
        ui.pop_layer();
        
        assert_eq!(ui.current_offset(), (0.0, 0.0));
    }

    #[test]
    fn test_ply_ui_nested_layers() {
        let mut ui = PlyUi::new();
        
        ui.push_layer();
        ui.translate(10.0, 10.0);
        
        ui.push_layer();
        ui.translate(20.0, 20.0);
        
        ui.push_layer();
        ui.translate(30.0, 30.0);
        
        assert_eq!(ui.current_offset(), (60.0, 60.0));
        
        ui.pop_layer();
        assert_eq!(ui.current_offset(), (30.0, 30.0));
        
        ui.pop_layer();
        assert_eq!(ui.current_offset(), (10.0, 10.0));
    }

    // Tests for scissor rect
    #[test]
    fn test_ply_ui_scissor_rect() {
        let mut ui = PlyUi::new();
        
        ui.set_scissor_rect(10.0, 20.0, 100.0, 200.0);
        assert!(ui.in_scissor_rect(50.0, 100.0));
        assert!(!ui.in_scissor_rect(5.0, 100.0)); // Outside x
        assert!(!ui.in_scissor_rect(50.0, 10.0)); // Outside y
        assert!(!ui.in_scissor_rect(150.0, 100.0)); // Outside width
        assert!(!ui.in_scissor_rect(50.0, 300.0)); // Outside height
    }

    #[test]
    fn test_ply_ui_scissor_rect_with_layers() {
        let mut ui = PlyUi::new();
        
        ui.push_layer();
        ui.set_scissor_rect(10.0, 10.0, 100.0, 100.0);
        
        ui.push_layer();
        ui.translate(50.0, 50.0);
        
        // Scissor rect should be from the current layer
        assert!(ui.in_scissor_rect(60.0, 60.0));
        assert!(!ui.in_scissor_rect(5.0, 60.0));
    }

    #[test]
    fn test_ply_ui_scissor_rect_no_clip() {
        let mut ui = PlyUi::new();
        
        // Without scissor rect, all points should be valid
        assert!(ui.in_scissor_rect(-1000.0, -1000.0));
        assert!(ui.in_scissor_rect(10000.0, 10000.0));
    }

    // Tests for widget state management
    #[test]
    fn test_ply_ui_widget_state_hover() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Create a widget at (100, 100) with size (200, 50)
        let state = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        
        // Initially not hovered (mouse at -1, -1)
        assert!(!state.hovered);
        assert!(!state.pressed);
        assert!(!state.clicked);
    }

    #[test]
    fn test_ply_ui_widget_state_hover_with_mouse() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Move mouse over widget
        ui.mouse_move(150.0, 125.0);
        
        let state = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        
        assert!(state.hovered);
        assert!(!state.pressed);
        assert!(!state.clicked);
    }

    #[test]
    fn test_ply_ui_widget_state_click() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Move mouse over widget
        ui.mouse_move(150.0, 125.0);
        
        // Press and release
        ui.mouse_down();
        let state1 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        assert!(state1.pressed);
        assert!(!state1.clicked);
        
        ui.mouse_up();
        let state2 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        assert!(!state2.pressed);
        // Click detection happens on mouse up
    }

    #[test]
    fn test_ply_ui_widget_state_click_outside() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Move mouse outside widget
        ui.mouse_move(50.0, 50.0);
        
        // Try to click
        ui.mouse_down();
        let _state1 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        
        ui.mouse_up();
        let state2 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        
        // Should not be clicked
        assert!(!state2.clicked);
    }

    #[test]
    fn test_ply_ui_multiple_widgets() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        ui.mouse_move(150.0, 125.0);
        
        let state1 = ui.update_widget_state(1, (100.0, 100.0, 200.0, 50.0));
        let state2 = ui.update_widget_state(2, (100.0, 200.0, 200.0, 50.0));
        
        assert!(state1.hovered);
        assert!(!state2.hovered);
    }

    #[test]
    fn test_ply_ui_widget_state_persistence() {
        let mut ui = PlyUi::new();
        
        ui.begin_frame(1280.0, 720.0);
        ui.mouse_move(150.0, 125.0);
        let state1 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        let _commands1 = ui.end_frame();
        
        // Next frame
        ui.begin_frame(1280.0, 720.0);
        ui.mouse_move(150.0, 125.0);
        let state2 = ui.update_widget_state(123, (100.0, 100.0, 200.0, 50.0));
        let _commands2 = ui.end_frame();
        
        // State should be consistent
        assert_eq!(state1.hovered, state2.hovered);
    }

    // Tests for render commands
    #[test]
    fn test_ply_ui_render_command_quad() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        let cmd = RenderCommand::Quad {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            color: [255, 0, 0, 255],
            border_radius: [5.0, 5.0, 5.0, 5.0],
        };
        
        ui.add_command(cmd);
        let commands = ui.end_frame();
        
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            RenderCommand::Quad { x, y, width, height, color, border_radius } => {
                assert_eq!(*x, 10.0);
                assert_eq!(*y, 20.0);
                assert_eq!(*width, 100.0);
                assert_eq!(*height, 50.0);
                assert_eq!(*color, [255, 0, 0, 255]);
                assert_eq!(*border_radius, [5.0, 5.0, 5.0, 5.0]);
            }
            _ => panic!("Expected Quad command"),
        }
    }

    #[test]
    fn test_ply_ui_render_command_text() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        let cmd = RenderCommand::Text {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            text: "Hello, World!".to_string(),
            size: 16.0,
            color: [255, 255, 255, 255],
            font: "Roboto-Regular".to_string(),
            alignment: TextAlignment::Center,
        };
        
        ui.add_command(cmd);
        let commands = ui.end_frame();
        
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            RenderCommand::Text { text, size, alignment, .. } => {
                assert_eq!(text, "Hello, World!");
                assert_eq!(*size, 16.0);
                assert_eq!(*alignment, TextAlignment::Center);
            }
            _ => panic!("Expected Text command"),
        }
    }

    #[test]
    fn test_ply_ui_render_command_icon() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        let cmd = RenderCommand::Icon {
            x: 10.0,
            y: 20.0,
            size: 24.0,
            icon: "play".to_string(),
            color: [255, 255, 255, 255],
        };
        
        ui.add_command(cmd);
        let commands = ui.end_frame();
        
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            RenderCommand::Icon { icon, size, .. } => {
                assert_eq!(icon, "play");
                assert_eq!(*size, 24.0);
            }
            _ => panic!("Expected Icon command"),
        }
    }

    #[test]
    fn test_ply_ui_multiple_commands() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        ui.add_command(RenderCommand::Quad {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            color: [255, 0, 0, 255],
            border_radius: [0.0, 0.0, 0.0, 0.0],
        });
        
        ui.add_command(RenderCommand::Text {
            x: 10.0,
            y: 10.0,
            width: 80.0,
            height: 80.0,
            text: "Test".to_string(),
            size: 16.0,
            color: [255, 255, 255, 255],
            font: "Roboto-Regular".to_string(),
            alignment: TextAlignment::Left,
        });
        
        let commands = ui.end_frame();
        assert_eq!(commands.len(), 2);
    }

    // Tests for text alignment
    #[test]
    fn test_text_alignment_variants() {
        let _ = TextAlignment::Left;
        let _ = TextAlignment::Center;
        let _ = TextAlignment::Right;
        
        // Test equality
        assert_eq!(TextAlignment::Left, TextAlignment::Left);
        assert_ne!(TextAlignment::Left, TextAlignment::Center);
    }

    // Tests for helper functions
    #[test]
    fn test_center_x() {
        assert_eq!(center_x(100.0, 20.0), 40.0);
        assert_eq!(center_x(200.0, 50.0), 75.0);
        assert_eq!(center_x(1000.0, 200.0), 400.0);
    }

    #[test]
    fn test_center_y() {
        assert_eq!(center_y(100.0, 20.0), 40.0);
        assert_eq!(center_y(200.0, 50.0), 75.0);
        assert_eq!(center_y(1000.0, 200.0), 400.0);
    }

    #[test]
    fn test_color_to_rgba() {
        assert_eq!(color_to_rgba([255, 0, 0], 128), [255, 0, 0, 128]);
        assert_eq!(color_to_rgba([0, 255, 0], 255), [0, 255, 0, 255]);
        assert_eq!(color_to_rgba([0, 0, 255], 0), [0, 0, 255, 0]);
    }

    // Performance tests
    #[test]
    fn test_ply_ui_frame_performance() {
        use std::time::Instant;
        
        let mut ui = PlyUi::new();
        
        let start = Instant::now();
        for i in 0..1000 {
            ui.begin_frame(1280.0, 720.0);
            ui.mouse_move(i as f32 % 1280.0, i as f32 % 720.0);
            
            // Add some widgets
            for j in 0..10 {
                ui.update_widget_state(j, (j as f32 * 100.0, 0.0, 50.0, 50.0));
            }
            
            let _commands = ui.end_frame();
        }
        let duration = start.elapsed();
        
        // Should complete in reasonable time (< 100ms for 1000 frames)
        assert!(duration.as_millis() < 100, "UI frame processing too slow: {:?}", duration);
    }

    #[test]
    fn test_ply_ui_command_addition_performance() {
        use std::time::Instant;
        
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        let start = Instant::now();
        for i in 0..10000 {
            ui.add_command(RenderCommand::Quad {
                x: i as f32 % 1280.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                color: [255, 255, 255, 255],
                border_radius: [0.0, 0.0, 0.0, 0.0],
            });
        }
        let duration = start.elapsed();
        
        // Should complete in reasonable time (< 50ms for 10k commands)
        assert!(duration.as_millis() < 50, "Command addition too slow: {:?}", duration);
    }

    // Memory tests
    #[test]
    fn test_ply_ui_memory_footprint() {
        use std::mem;
        
        let ui = PlyUi::new();
        
        // UI should have reasonable memory footprint
        let size = mem::size_of_val(&ui);
        assert!(size < 1024 * 1024, "UI too large: {} bytes", size);
    }

    // Edge case tests
    #[test]
    fn test_ply_ui_zero_dimensions() {
        let mut ui = PlyUi::new();
        
        // Should handle zero dimensions
        ui.begin_frame(0.0, 0.0);
        let _commands = ui.end_frame();
    }

    #[test]
    fn test_ply_ui_negative_coordinates() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Should handle negative coordinates
        ui.mouse_move(-100.0, -100.0);
        ui.translate(-50.0, -50.0);
        
        let _commands = ui.end_frame();
    }

    #[test]
    fn test_ply_ui_very_large_coordinates() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Should handle very large coordinates
        ui.mouse_move(100000.0, 100000.0);
        ui.translate(50000.0, 50000.0);
        
        let _commands = ui.end_frame();
    }

    #[test]
    fn test_ply_ui_empty_text() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        let cmd = RenderCommand::Text {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            text: "".to_string(),
            size: 16.0,
            color: [255, 255, 255, 255],
            font: "Roboto-Regular".to_string(),
            alignment: TextAlignment::Left,
        };
        
        ui.add_command(cmd);
        let commands = ui.end_frame();
        
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_ply_ui_zero_size_widget() {
        let mut ui = PlyUi::new();
        ui.begin_frame(1280.0, 720.0);
        
        // Should handle zero-size widgets
        let state = ui.update_widget_state(123, (100.0, 100.0, 0.0, 0.0));
        
        assert!(!state.hovered);
    }
}
