# Piano Keyboard Implementation Plan - PLY Mode

## Overview
Implement a visual piano keyboard for Neothesia's PLY (macroquad) rendering mode that works in both freeplay and regular song play modes. The keyboard will display configurable note ranges, provide visual feedback on key presses, and sync with audio input from both keyboard and mouse.

---

## Current Architecture Analysis

### Rendering System
- **Framework**: Macroquad (2D graphics, non-WGPU)
- **Main Loop**: `main_macroquad.rs` with async `next_frame().await`
- **Existing Renderer**: `MacroquadKeyboardRenderer` in `render/ply/macroquad_renderer.rs`
- **Drawing Primitives**: `draw_rectangle()`, `draw_rectangle_lines()`, `draw_text()`, `draw_circle()`
- **UI Framework**: PLY UI in `ply_integration/ui/` with widgets, layout, input handling

### Input System
- **Handler**: `PlyInputHandler` in `ply_integration/input/mod.rs`
- **Keyboard Path**: Macroquad → KeyboardInput → KeyboardToMidiConverter → MIDI events
- **Mouse Path**: Macroquad → MouseInput → UI events
- **Piano Mapping**: `keyboard_to_midi.rs` maps keys (a,w,s,e,d,f,t,g,y,h,u,j,k,o,l,p,;,') to MIDI notes starting at C4

### Audio System
- **Backend**: `SynthBackend` using oxisynth/fluidsynth with soundfont loading
- **Integration**: `PlyAudioManager` wraps Neothesia backends
- **Flow**: Key press → NoteOn event → PlyAudioManager → SynthBackend → Audio output

### Configuration
- **Range Storage**: `(u8, u8)` tuple in `keyboard_layout.range` field
- **Access**: `config.piano_range()` returns `RangeInclusive<u8>`
- **Default**: (21, 108) for 88-key piano
- **Modification**: `set_piano_range_start()`, `set_piano_range_end()`

### Game Modes
- **Freeplay**: `FreeplayScene` in `scene/freeplay/mod.rs`
- **Regular Play**: `PlayingScene` in `scene/playing_scene/mod.rs`
- **Integration**: Both use `render_ply()` method with `PlyRendererCoordinator`

### Piano Data (piano-layout crate)
- **Note Constants**: C=0, C#=1, D=2, ..., B=11
- **Structs**: `Key`, `KeyKind`, `KeyboardLayout`, `Sizing`
- **Range**: `KeyboardRange` for note range handling

---

## Implementation Plan

### Phase 1: Visual Piano Keyboard Component

**File**: `src/render/ply/piano_keyboard.rs` (new)

**Responsibilities**:
1. Render piano keys based on configured note range
2. Handle key press/release visual effects
3. Calculate key positions and dimensions
4. Support both white and black key rendering

**Data Structures**:
```rust
pub struct PianoKeyboardRenderer {
    range: RangeInclusive<u8>,
    key_states: HashMap<u8, KeyState>,
    layout: piano_layout::KeyboardLayout,
    sizing: piano_layout::Sizing,
}

pub struct KeyState {
    is_pressed: bool,
    press_animation: f32,  // 0.0 to 1.0 for visual effect
    last_press_time: f64,
}
```

**Key Features**:
- Use `piano_layout::KeyboardLayout` for key positioning
- White keys: standard width, black keys: offset and narrower
- Visual feedback: color change, glow, or size change when pressed
- Smooth animation for press/release transitions

### Phase 2: Mouse Input Handling

**File**: `src/render/ply/piano_keyboard.rs` (extend)

**Responsibilities**:
1. Detect mouse clicks on piano keys
2. Map mouse coordinates to key bounds
3. Trigger note on/off events

**Implementation**:
```rust
impl PianoKeyboardRenderer {
    pub fn handle_mouse_input(&mut self, mouse_pos: Vec2, mouse_button: MouseButton) {
        // Convert mouse position to key
        if let Some(note) = self.get_key_at_position(mouse_pos) {
            match mouse_button {
                MouseButton::Left => {
                    self.key_states.entry(note).and_modify(|state| {
                        state.is_pressed = true;
                        state.press_animation = 1.0;
                    });
                    // Trigger NoteOn event
                }
                // ... handle release
            }
        }
    }

    fn get_key_at_position(&self, pos: Vec2) -> Option<u8> {
        // Reverse lookup using layout bounds
    }
}
```

### Phase 3: Keyboard Input Integration

**File**: `src/ply_integration/input/keyboard_to_midi.rs` (modify)

**Changes Needed**:
1. Emit additional events for visual feedback when keys are pressed
2. Share key state with piano keyboard renderer
3. Sync visual state with MIDI note on/off

**Implementation**:
- Add callback or event system to notify renderer of key changes
- Pass key state updates through context or event bus

### Phase 4: Audio Synchronization

**File**: `src/render/ply/piano_keyboard.rs` (integrate)

**Integration**:
- Use existing `PlyAudioManager` for note playback
- Ensure visual feedback syncs with audio timing
- Handle note duration and release properly

**Flow**:
```
User Input (Keyboard/Mouse)
    ↓
PianoKeyboardRenderer.handle_input()
    ↓
Update visual state (animation)
    ↓
Trigger MIDI event (NoteOn/NoteOff)
    ↓
PlyAudioManager.process_event()
    ↓
SynthBackend plays audio
```

### Phase 5: Mode Integration

**Files**:
- `src/scene/freeplay/mod.rs`
- `src/scene/playing_scene/mod.rs`

**Changes Needed**:
1. Add `PianoKeyboardRenderer` to scene state
2. Call renderer's `render()` method in `render_ply()`
3. Forward input events to renderer
4. Sync with existing keyboard rendering if present

**Implementation**:
```rust
// In FreeplayScene and PlayingScene
pub struct FreeplayScene {
    // ... existing fields
    piano_keyboard: PianoKeyboardRenderer,
}

impl Scene for FreeplayScene {
    fn render_ply(&mut self, ctx: &mut PlyContext) {
        // ... existing rendering
        self.piano_keyboard.render(ctx);
    }

    fn handle_event(&mut self, event: &NeothesiaEvent) {
        // Forward input to piano keyboard
        match event {
            NeothesiaEvent::MidiNoteOn(note, _) => {
                self.piano_keyboard.set_key_pressed(note, true);
            }
            NeothesiaEvent::MidiNoteOff(note) => {
                self.piano_keyboard.set_key_pressed(note, false);
            }
            // ... handle mouse events
        }
    }
}
```

### Phase 6: Configuration Integration

**Implementation**:
- Read note range from `config.piano_range()`
- Recreate layout when range changes
- Handle runtime configuration updates

---

## File Structure

### New Files
```
src/render/ply/
├── piano_keyboard.rs       # Main piano keyboard renderer
└── mod.rs                  # Export PianoKeyboardRenderer
```

### Modified Files
```
src/scene/freeplay/mod.rs           # Add piano keyboard to freeplay
src/scene/playing_scene/mod.rs      # Add piano keyboard to regular play
src/ply_integration/input/keyboard_to_midi.rs  # Emit visual feedback events
src/render/ply/mod.rs               # Export new renderer
src/context.rs                      # Add piano keyboard to context if needed
```

---

## Technical Specifications

### Key Rendering
- **White Keys**: Full height, standard width, light color
- **Black Keys**: Half height, narrower, dark color, offset between white keys
- **Pressed State**: Highlight color with glow effect
- **Animation**: Smooth transition (100-200ms) using press_animation field

### Layout Calculation
```rust
use piano_layout::{KeyboardLayout, KeyboardRange, Sizing};

let range = KeyboardRange::new(config.piano_range());
let sizing = Sizing {
    white_key_width: 40.0,
    black_key_width: 24.0,
    key_height: 120.0,
    // ... other dimensions
};
let layout = KeyboardLayout::new(range, sizing);
```

### Mouse Hit Testing
- Iterate through keys in layout
- Check if mouse position within key bounds
- Prioritize black keys (rendered on top)
- Return MIDI note number

### Input Synchronization
- Keyboard: Use existing `KeyboardToMidiConverter`
- Mouse: Add new mouse input handler in piano keyboard
- Both: Update key state and trigger MIDI events
- Visual feedback: Immediate update via `key_states` map

### Audio Feedback
- NoteOn: Trigger via `PlyAudioManager.note_on(note, velocity)`
- NoteOff: Trigger via `PlyAudioManager.note_off(note)`
- Ensure timing matches visual feedback

---

## Dependencies

### Existing Crates
- `macroquad` — 2D rendering primitives
- `piano-layout` — Keyboard layout calculation
- `midi-file` — MIDI note definitions

### No New Dependencies Required
All functionality can be implemented with existing crates.

---

## Testing Checklist

- [ ] Piano keyboard renders with correct note range from config
- [ ] White keys and black keys display correctly
- [ ] Keyboard input triggers visual feedback and audio
- [ ] Mouse clicks trigger visual feedback and audio
- [ ] Pressed keys animate smoothly
- [ ] Released keys return to normal state
- [ ] Works in freeplay mode
- [ ] Works in regular song play mode
- [ ] Configuration changes update keyboard range
- [ ] Audio timing matches visual feedback
- [ ] No performance degradation in render loop

---

## Open Questions

1. **Keyboard Position**: Should the piano keyboard be fixed at bottom of screen or configurable position?
2. **Size**: Fixed size or responsive to window size?
3. **Interaction**: Should mouse drag work (slide across keys like real piano)?
4. **Visual Style**: Specific colors for white/black keys, pressed state, effects?
5. **Keyboard Overlap**: How to integrate with existing keyboard renderer (keep both or replace)?

---

## Implementation Order

1. **Phase 1**: Create basic piano keyboard renderer (visual only)
2. **Phase 2**: Add mouse input handling
3. **Phase 3**: Integrate keyboard input for visual feedback
4. **Phase 4**: Ensure audio synchronization
5. **Phase 5**: Integrate into both game modes
6. **Phase 6**: Add configuration integration

---

## Risk Mitigation

- **Performance**: Cache layout calculations, only redraw when state changes
- **Complexity**: Start with simple rectangle rendering, add effects later
- **Input Conflicts**: Ensure piano input doesn't conflict with existing bindings
- **State Sync**: Use single source of truth for key state (renderer owns state)

---

## Success Criteria

✓ Piano keyboard displays correct note range from config
✓ Visual feedback on key press (keyboard and mouse)
✓ Audio playback synchronized with visual feedback
✓ Works in both freeplay and regular play modes
✓ Smooth animations and responsive input
✓ Clean integration with existing code architecture
