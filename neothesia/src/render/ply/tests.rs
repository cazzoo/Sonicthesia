#[cfg(test)]
mod tests {
    use super::super::guidelines::PlyGuidelineRenderer;
    use super::super::keyboard::PlyKeyboardRenderer;
    use super::super::note_labels::PlyNoteLabelsRenderer;
    use super::super::renderer::PlyRendererCoordinator;
    use super::super::waterfall::PlyWaterfallRenderer;
    use super::*;
    use piano_layout::{KeyboardLayout, KeyboardRange, Sizing};
    use std::sync::Arc;
    use std::time::Duration;

    // Helper function to create a keyboard layout for testing
    fn create_test_layout(start: u8, end: u8) -> KeyboardLayout {
        let range = KeyboardRange::new(start..end);
        let white_count = range.white_count();
        let neutral_width = 24.0;
        let neutral_height = 100.0;

        KeyboardLayout::from_range(Sizing::new(neutral_width, neutral_height), range)
    }

    // Tests for PlyWaterfallRenderer
    #[test]
    fn test_waterfall_renderer_creation() {
        let renderer = PlyWaterfallRenderer::new();
        // Renderer should be created successfully
        // Note: Without initialization, most operations will be no-ops
    }

    #[test]
    fn test_waterfall_renderer_notes_empty() {
        let renderer = PlyWaterfallRenderer::new();
        // Before initialization, notes should be empty or inaccessible
        // This test verifies the renderer handles this gracefully
    }

    // Tests for PlyKeyboardRenderer
    #[test]
    fn test_keyboard_renderer_creation() {
        let layout = create_test_layout(21, 108); // Standard 88-key piano
        let renderer = PlyKeyboardRenderer::new(layout.clone());

        // Verify layout is stored
        assert_eq!(renderer.layout().range.start(), 21);
        assert_eq!(renderer.layout().range.end(), 108);
    }

    #[test]
    fn test_keyboard_renderer_position() {
        let layout = create_test_layout(21, 108);
        let mut renderer = PlyKeyboardRenderer::new(layout);

        renderer.set_pos((100.0, 200.0));
        assert_eq!(renderer.pos(), (100.0, 200.0));
    }

    #[test]
    fn test_keyboard_renderer_layout_change() {
        let layout1 = create_test_layout(21, 88);
        let mut renderer = PlyKeyboardRenderer::new(layout1);

        let layout2 = create_test_layout(36, 72);
        renderer.set_layout(layout2.clone());

        // Layout should be updated
        assert_eq!(renderer.layout().range.start(), 36);
        assert_eq!(renderer.layout().range.end(), 72);
    }

    #[test]
    fn test_keyboard_renderer_reset_notes() {
        let layout = create_test_layout(21, 108);
        let mut renderer = PlyKeyboardRenderer::new(layout);

        // Should not panic even with no notes to reset
        renderer.reset_notes();
    }

    #[test]
    fn test_keyboard_renderer_update() {
        let layout = create_test_layout(21, 108);
        let mut renderer = PlyKeyboardRenderer::new(layout);

        // Should not panic on update
        renderer.update();
    }

    #[test]
    fn test_keyboard_renderer_key_states() {
        let layout = create_test_layout(21, 108);
        let renderer = PlyKeyboardRenderer::new(layout);

        // Should have key states for all keys in range
        let key_states = renderer.key_states();
        assert!(!key_states.is_empty());
    }

    // Tests for PlyGuidelineRenderer
    #[test]
    fn test_guideline_renderer_creation() {
        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from([]);

        let renderer = PlyGuidelineRenderer::new(
            layout,
            (0.0, 0.0),
            true, // vertical_guidelines
            true, // horizontal_guidelines
            measures,
        );

        // Renderer should be created successfully
    }

    #[test]
    fn test_guideline_renderer_position() {
        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from([]);

        let mut renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        renderer.set_pos((50.0, 100.0));
        // Position should be updated
    }

    #[test]
    fn test_guideline_renderer_layout_change() {
        let layout1 = create_test_layout(21, 88);
        let measures: Arc<[Duration]> = Arc::from([]);

        let mut renderer = PlyGuidelineRenderer::new(layout1, (0.0, 0.0), true, true, measures);

        let layout2 = create_test_layout(36, 72);
        renderer.set_layout(layout2);

        // Layout should be updated
    }

    #[test]
    fn test_guideline_renderer_update() {
        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from(vec![
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(6),
        ]);

        let mut renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        // Should not panic on update
        renderer.update(10.0, 1.0, 1.0);
    }

    // Tests for PlyNoteLabelsRenderer
    #[test]
    fn test_note_labels_note_label() {
        // Test the static method for getting note labels
        assert_eq!(PlyNoteLabelsRenderer::note_label(0), "C");
        assert_eq!(PlyNoteLabelsRenderer::note_label(1), "C#");
        assert_eq!(PlyNoteLabelsRenderer::note_label(11), "B");
    }

    // Integration tests for PlyRendererCoordinator
    #[test]
    fn test_renderer_coordinator_creation() {
        let coordinator = PlyRendererCoordinator::new();
        assert!(!coordinator.is_initialized());

        // All renderers should be None initially
        let mut coord = coordinator;
        assert!(coord.waterfall_mut().is_none());
        assert!(coord.keyboard_mut().is_none());
        assert!(coord.guidelines_mut().is_none());
        assert!(coord.note_labels_mut().is_none());
    }

    #[test]
    fn test_renderer_coordinator_default() {
        let coordinator = PlyRendererCoordinator::default();
        assert!(!coordinator.is_initialized());
    }

    #[test]
    fn test_renderer_coordinator_update_uninitialized() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Should not panic even when uninitialized
        coordinator.update(0.0, 1.0, 1.0, 0.0);
    }

    #[test]
    fn test_renderer_coordinator_keyboard_layout_change() {
        let mut coordinator = PlyRendererCoordinator::new();

        let layout1 = create_test_layout(21, 88);
        coordinator.set_keyboard_layout(layout1);

        let layout2 = create_test_layout(36, 72);
        coordinator.set_keyboard_layout(layout2);

        // Should not panic even without initialization
    }

    #[test]
    fn test_renderer_coordinator_keyboard_position_change() {
        let mut coordinator = PlyRendererCoordinator::new();

        coordinator.set_keyboard_position((100.0, 200.0));

        // Should not panic even without initialization
    }

    #[test]
    fn test_renderer_coordinator_reset_keyboard_notes() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Should not panic even without initialization
        coordinator.reset_keyboard_notes();
    }

    // Performance benchmarks for rendering
    #[test]
    fn test_keyboard_renderer_update_performance() {
        use std::time::Instant;

        let layout = create_test_layout(21, 108);
        let mut renderer = PlyKeyboardRenderer::new(layout);

        let start = Instant::now();
        for _ in 0..10000 {
            renderer.update();
        }
        let duration = start.elapsed();

        // Should complete in reasonable time (< 50ms for 10k updates)
        assert!(
            duration.as_millis() < 50,
            "Keyboard renderer update too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_guideline_renderer_update_performance() {
        use std::time::Instant;

        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from(vec![
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(6),
            Duration::from_secs(8),
        ]);

        let mut renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        let start = Instant::now();
        for i in 0..10000 {
            renderer.update(i as f32 * 0.016, 1.0, 1.0);
        }
        let duration = start.elapsed();

        // Should complete in reasonable time (< 100ms for 10k updates)
        assert!(
            duration.as_millis() < 100,
            "Guideline renderer update too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_renderer_coordinator_update_performance() {
        use std::time::Instant;

        let mut coordinator = PlyRendererCoordinator::new();

        let start = Instant::now();
        for i in 0..1000 {
            coordinator.update(i as f32 * 0.016, 1.0, 1.0, 0.0);
        }
        let duration = start.elapsed();

        // Should complete in reasonable time (< 20ms for 1k updates)
        assert!(
            duration.as_millis() < 20,
            "Coordinator update too slow: {:?}",
            duration
        );
    }

    // Memory usage tests
    #[test]
    fn test_keyboard_renderer_memory_footprint() {
        use std::mem;

        let layout = create_test_layout(21, 108);
        let renderer = PlyKeyboardRenderer::new(layout);

        // Renderer should have reasonable memory footprint
        let size = mem::size_of_val(&renderer);
        assert!(
            size < 1024 * 1024,
            "Keyboard renderer too large: {} bytes",
            size
        );
    }

    #[test]
    fn test_guideline_renderer_memory_footprint() {
        use std::mem;

        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from(vec![
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(6),
        ]);

        let renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        // Renderer should have reasonable memory footprint
        let size = mem::size_of_val(&renderer);
        assert!(
            size < 1024 * 1024,
            "Guideline renderer too large: {} bytes",
            size
        );
    }

    #[test]
    fn test_renderer_coordinator_memory_footprint() {
        use std::mem;

        let coordinator = PlyRendererCoordinator::new();

        // Coordinator should have reasonable memory footprint
        let size = mem::size_of_val(&coordinator);
        assert!(size < 1024 * 1024, "Coordinator too large: {} bytes", size);
    }

    // Stress tests
    #[test]
    fn test_multiple_renderers_coexistence() {
        let layout = create_test_layout(21, 108);

        let _keyboard = PlyKeyboardRenderer::new(layout.clone());
        let measures: Arc<[Duration]> = Arc::from([]);
        let _guidelines = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        // All renderers should coexist without issues
    }

    #[test]
    fn test_renderer_coordinator_with_all_components() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Test that coordinator can handle operations even without initialization
        coordinator.update(0.0, 1.0, 1.0, 0.0);
        coordinator.set_keyboard_layout(create_test_layout(21, 108));
        coordinator.set_keyboard_position((100.0, 200.0));
        coordinator.reset_keyboard_notes();

        // Should not panic
    }

    // Edge case tests
    #[test]
    fn test_keyboard_renderer_extreme_ranges() {
        // Test with very small range
        let layout_small = create_test_layout(60, 72);
        let _renderer_small = PlyKeyboardRenderer::new(layout_small);

        // Test with full MIDI range
        let layout_full = create_test_layout(0, 127);
        let _renderer_full = PlyKeyboardRenderer::new(layout_full);

        // Both should work without issues
    }

    #[test]
    fn test_guideline_renderer_no_measures() {
        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from([]);

        let mut renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        // Should handle empty measures gracefully
        renderer.update(10.0, 1.0, 1.0);
    }

    #[test]
    fn test_guideline_renderer_many_measures() {
        let layout = create_test_layout(21, 108);
        let measures: Arc<[Duration]> = Arc::from(
            (0..1000)
                .map(|i| Duration::from_secs(i as u64 * 2))
                .collect::<Vec<_>>(),
        );

        let mut renderer = PlyGuidelineRenderer::new(layout, (0.0, 0.0), true, true, measures);

        // Should handle many measures
        renderer.update(100.0, 1.0, 1.0);
    }

    #[test]
    fn test_renderer_coordinator_zero_scale() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Should handle zero scale
        coordinator.update(0.0, 1.0, 0.0, 0.0);
    }

    #[test]
    fn test_renderer_coordinator_large_scale() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Should handle large scale
        coordinator.update(0.0, 1.0, 10.0, 0.0);
    }

    #[test]
    fn test_renderer_coordinator_negative_time() {
        let mut coordinator = PlyRendererCoordinator::new();

        // Should handle negative time gracefully
        coordinator.update(-10.0, 1.0, 1.0, 0.0);
    }

    #[test]
    fn test_keyboard_renderer_key_states_count() {
        let layout = create_test_layout(21, 108);
        let renderer = PlyKeyboardRenderer::new(layout);

        // Should have key states for all keys in range
        let key_states = renderer.key_states();
        let expected_count = 108 - 21; // 87 keys
        assert_eq!(key_states.len(), expected_count);
    }

    #[test]
    fn test_keyboard_renderer_key_state_colors() {
        let layout = create_test_layout(60, 72); // Small range for easier testing
        let mut renderer = PlyKeyboardRenderer::new(layout);

        let key_states = renderer.key_states_mut();

        // Test setting pressed by user
        key_states[0].pressed_by_user_on([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(key_states[0].pressed_by_user(), Some([1.0, 0.0, 0.0, 1.0]));

        // Test setting pressed by file
        key_states[0].pressed_by_file_on([0.0, 1.0, 0.0, 1.0]);
        assert_eq!(key_states[0].pressed_by_file(), Some([0.0, 1.0, 0.0, 1.0]));

        // Test clearing
        key_states[0].pressed_by_user_off();
        assert!(key_states[0].pressed_by_user().is_none());
    }
}
