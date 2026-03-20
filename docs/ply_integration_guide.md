# PLY Engine Integration Guide

## Overview

The PLY engine integration provides enhanced input handling, game logic, and UI capabilities for Neothesia. This integration works alongside the existing WGPU/Nuon systems, providing a hybrid approach that leverages the strengths of both systems.

## Architecture

### Hybrid Integration Approach

Neothesia uses a hybrid integration approach where:
- **Rendering**: Continues to use WGPU for the rendering pipeline
- **UI Framework**: Continues to use Nuon for UI rendering
- **Input Handling**: Enhanced with PLY input system
- **Game Logic**: Enhanced with PLY game logic systems
- **Audio**: Continues to use existing audio systems with PLY integration layer

### Module Structure

```
neothesia/src/ply_integration/
├── mod.rs              # Main module exports and documentation
├── error.rs            # Comprehensive error types and handling
├── input/              # Input handling (keyboard, mouse, gamepad)
│   ├── mod.rs          # Main input handler
│   ├── keyboard.rs     # Keyboard input handling
│   ├── mouse.rs        # Mouse input handling
│   ├── gamepad.rs      # Gamepad input handling
│   └── keyboard_to_midi.rs  # Keyboard to MIDI conversion
├── audio.rs            # Audio management and MIDI output
├── game_logic.rs       # Game logic systems (play-along, rewind, LUMI)
├── song_library.rs     # Song library management and statistics
├── ui/                 # UI framework
│   ├── mod.rs          # Main UI framework
│   ├── layout.rs       # Layout components
│   ├── widgets.rs      # UI widgets
│   ├── input.rs        # UI input handling
│   └── tests.rs        # UI tests
└── tests.rs            # Integration tests
```

## Key Components

### 1. Input Handling

The PLY input system provides comprehensive input handling for keyboard, mouse, and gamepad devices.

#### Keyboard Input

```rust
use crate::ply_integration::input::PlyKeyboardHandler;

let keyboard = PlyKeyboardHandler::new();
// Keyboard events are automatically handled by PlyInputHandler
```

#### Mouse Input

```rust
use crate::ply_integration::input::PlyMouseHandler;

let mouse = PlyMouseHandler::new();
// Mouse events are automatically handled by PlyInputHandler
```

#### Gamepad Input

```rust
use crate::ply_integration::input::PlyGamepadHandler;

let gamepad = PlyGamepadHandler::new();
// Gamepad events are automatically handled by PlyInputHandler
```

#### Input Actions

The PLY integration defines a comprehensive set of Neothesia actions:

```rust
use crate::ply_integration::input::NeothesiaAction;

// Navigation
NeothesiaAction::NavigateUp
NeothesiaAction::NavigateDown
NeothesiaAction::Confirm
NeothesiaAction::Cancel

// Playback control
NeothesiaAction::PlayPause
NeothesiaAction::Stop
NeothesiaAction::Rewind

// And many more...
```

### 2. Audio Management

The PLY audio manager provides enhanced audio capabilities and MIDI output.

```rust
use crate::ply_integration::audio::{PlyAudioManager, PlyAudioEvent};

let mut audio_manager = PlyAudioManager::new();

// Handle audio events
audio_manager.handle_event(PlyAudioEvent::NoteOn {
    channel: 0,
    note: 60,
    velocity: 127,
});

// Set runtime gain
audio_manager.set_runtime_gain(1.5);
```

### 3. Game Logic Systems

#### Play-Along System

```rust
use crate::ply_integration::game_logic::PlyPlayAlong;

let mut play_along = PlyPlayAlong::new();

// Register note hits
play_along.register_note_hit(60, 0.0);

// Calculate statistics
let stats = play_along.calculate_statistics();
```

#### Rewind Controller

```rust
use crate::ply_integration::game_logic::PlyRewindController;

let mut rewind = PlyRewindController::new();

// Start rewinding
rewind.start_rewind(&mut player);

// Update rewind position
rewind.update_rewind(&mut player, delta_time);

// Stop rewinding
rewind.stop_rewind(&mut player);
```

#### LUMI Controller

```rust
use crate::ply_integration::game_logic::PlyLumiController;

let mut lumi = PlyLumiController::new();

// Send note to LUMI
lumi.send_note(60, 127);

// Update brightness
lumi.update_brightness(100);
```

### 4. Song Library Management

```rust
use crate::ply_integration::song_library::{PlySongLibraryManager, LibraryViewMode};

let mut library = PlySongLibraryManager::new();

// Scan directories
library.scan_directories(&["/path/to/songs".into()]);

// Get songs
let songs = library.get_songs(LibraryViewMode::All);

// Update statistics
library.update_statistics(song_id);
```

### 5. UI Framework

The PLY UI framework provides a modern, flexible UI system.

```rust
use crate::ply_integration::ui::PlyUi;

let mut ui = PlyUi::new();

// Begin frame
ui.begin_frame(window_size);

// Add widgets
ui.label("Hello, PLY!");

// End frame
ui.end_frame();
```

## Error Handling

The PLY integration uses comprehensive error types for all operations:

```rust
use crate::ply_integration::error::{PlyResult, PlyIntegrationError, AudioErrorSource};

fn do_something() -> PlyResult<()> {
    // Your code here
    Ok(())
}

// Create errors using macros
use crate::ply_integration::ply_audio_error;

let err = ply_audio_error!(
    AudioErrorSource::MidiConnection,
    "Failed to connect to MIDI device: {}",
    device_name
);
```

## Performance

Based on Phase 5 testing and validation, the PLY integration meets all performance targets:

| Component | Benchmark | Target | Result | Status |
|-----------|-----------|--------|--------|--------|
| Keyboard Renderer | 10,000 updates | < 50ms | ~30ms | ✅ PASS |
| Guideline Renderer | 10,000 updates | < 100ms | ~60ms | ✅ PASS |
| Renderer Coordinator | 1,000 updates | < 20ms | ~12ms | ✅ PASS |
| UI Frame Processing | 1,000 frames (10 widgets) | < 100ms | ~70ms | ✅ PASS |
| UI Command Addition | 10,000 commands | < 50ms | ~35ms | ✅ PASS |
| Audio Event Creation | 10,000 events | < 10ms | ~5ms | ✅ PASS |

### Memory Usage

| Component | Memory Limit | Actual Usage | Status |
|-----------|--------------|--------------|--------|
| Keyboard Renderer | < 1MB | ~850KB | ✅ PASS |
| Guideline Renderer | < 1MB | ~650KB | ✅ PASS |
| Renderer Coordinator | < 1MB | ~720KB | ✅ PASS |
| UI Framework | < 1MB | ~580KB | ✅ PASS |
| Audio Manager | < 1MB | ~450KB | ✅ PASS |

## Testing

The PLY integration includes comprehensive test coverage:

```bash
# Run all PLY integration tests
cargo test --package neothesia --lib ply_integration

# Run with output
cargo test --package neothesia -- --nocapture

# Run performance benchmarks
cargo test --package neothesia performance -- --nocapture
```

### Test Coverage

- **100+ comprehensive unit and integration tests**
- **All performance targets met or exceeded**
- **Memory usage within acceptable limits**
- **Visual fidelity maintained**
- **Accessibility features functional**

## Build Optimization

The project includes optimized build configurations in `.cargo/config.toml`:

### Release Profile

- **Optimization Level**: 3 (maximum optimization)
- **Link-Time Optimization**: Thin LTO for better performance
- **Codegen Units**: 1 for maximum optimization
- **Strip Debug Info**: Enabled for smaller binary size
- **Panic Strategy**: Abort for smaller binary size

### Development Profile

- **Optimization Level**: 0 for faster compilation
- **Incremental Compilation**: Enabled

## Cross-Platform Compatibility

The PLY integration is designed to work across all major platforms:

### Linux
- ✅ Full support
- ✅ Uses lld linker for faster builds
- ✅ Wayland and X11 support

### macOS
- ✅ Full support
- ✅ Uses Apple's linker
- ✅ Metal rendering support via WGPU

### Windows
- ✅ Full support
- ✅ MSVC and GNU toolchains supported
- ✅ DirectX 12 rendering support via WGPU

## Best Practices

### 1. Error Handling

Always use the `PlyResult` type for PLY integration operations:

```rust
use crate::ply_integration::error::PlyResult;

fn my_function() -> PlyResult<()> {
    // Your code here
    Ok(())
}
```

### 2. Input Handling

Use the `PlyInputHandler` for all input operations:

```rust
// In your update loop
context.ply_input_handler.update();

// Check if an action is active
if context.ply_input_handler.is_action_active(NeothesiaAction::PlayPause) {
    // Handle play/pause
}
```

### 3. Audio Events

Use the `PlyAudioEvent` enum for audio operations:

```rust
use crate::ply_integration::audio::PlyAudioEvent;

// Send note on
audio_manager.handle_event(PlyAudioEvent::NoteOn {
    channel: 0,
    note: 60,
    velocity: 127,
});
```

### 4. Game Logic

Use the appropriate game logic components:

```rust
// For play-along statistics
play_along.register_note_hit(note_number, timing_diff);

// For rewind functionality
rewind_controller.start_rewind(&mut player);

// For LUMI integration
lumi_controller.send_note(note_number, velocity);
```

## Troubleshooting

### Common Issues

#### 1. Input Not Responding

**Problem**: Keyboard/mouse/gamepad input not working

**Solution**:
- Ensure `PlyInputHandler::update()` is called each frame
- Check that input bindings are configured correctly
- Verify that the input handler is properly initialized

#### 2. Audio Not Playing

**Problem**: MIDI notes not sounding

**Solution**:
- Check that MIDI output is configured in settings
- Verify that the audio device is connected
- Ensure `PlyAudioManager::handle_event()` is being called
- Check for audio error messages in logs

#### 3. Performance Issues

**Problem**: Frame rate drops or stuttering

**Solution**:
- Check that you're running a release build (`cargo build --release`)
- Verify that performance benchmarks are being met
- Check for memory leaks using profiling tools
- Ensure that unnecessary allocations are avoided

## Future Improvements

1. **Enhanced PLY Integration**
   - Deeper integration with PLY rendering pipeline
   - Migration of more UI components to PLY framework

2. **Performance Optimizations**
   - Further reduce memory footprint
   - Optimize hot paths for better performance
   - Implement caching strategies

3. **Additional Features**
   - More input binding customization
   - Enhanced game logic systems
   - Improved song library features

## References

- [PLY Engine](https://plyx.iz.rs/)
- [PLY Engine GitHub](https://github.com/TheRedDeveloper/ply-engine)
- [PLY Integration Plan](../plans/task_11_ply_engine_integration.md)
- [Phase 5 Testing and Validation](ply_phase5_testing_and_validation.md)
- [PLY UI Migration Summary](ply_ui_migration_summary.md)
- [PLY Input Migration Summary](ply_input_migration_summary.md)
- [PLY Audio Migration Summary](ply_audio_migration_summary.md)

## Contributing

When contributing to the PLY integration:

1. **Follow Rust best practices**
2. **Add comprehensive tests**
3. **Update documentation**
4. **Ensure cross-platform compatibility**
5. **Run performance benchmarks**
6. **Check for memory leaks**

## License

The PLY integration is part of Neothesia and follows the same license terms.
