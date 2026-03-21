# Waterfall Rendering Techniques

This document describes the various rendering techniques for the waterfall visualization system.

## Implemented Techniques

### 2D Top-to-Bottom (Default)

Notes fall from the top of the screen toward the keyboard at the bottom.

**Coordinate Transformation:**
```
y = keyboard_top - (time_until_start * pixels_per_second)
height = note_duration * pixels_per_second
x = key.x()
width = key.width() - 1.0
```

**Visual Properties:**
- Notes are rectangles with consistent width per key
- Height corresponds to note duration
- Color based on track
- Notes appear when `current_time >= note.start - travel_time`
- Notes disappear when `y + height > keyboard_top`

## Planned Techniques

### 2.5D Perspective

Notes fall with perspective distortion, appearing smaller at the top and larger near the keyboard.

**Coordinate Transformation:**
```
depth = (keyboard_top - y) / keyboard_top
scale = 1.0 - depth * 0.7  # Notes shrink as they recede
x = center_x + (key.x() - center_x) * scale
width = key.width() * scale
height = note_duration * pixels_per_second * scale
```

**Visual Properties:**
- Perspective-correct rendering
- Depth fog effect (notes fade into distance)
- 3D rotation of note rectangles

### 3D Depth-to-Close

Notes come from the distance toward the viewer, like a tunnel.

**Coordinate Transformation:**
```
z = note.start - current_time
if z > 0:
    scale = 1.0 / (z + 1.0)
    x = screen_center_x + (key.x() - center_x) * scale
    y = screen_center_y + (keyboard_y - center_y) * scale
    width = key.width() * scale
    height = note_duration * scale
```

**Visual Properties:**
- Vanishing point perspective
- Notes grow as they approach
- Fog effect for depth cue

### 3D Spacecraft/Milky Way

A sci-fi themed visualization where the player rides through a musical tunnel.

**Coordinate Transformation:**
```
# Tunnel coordinates
angle = note.pitch * (2 * PI / 88)
radius = base_radius + (note.start - current_time) * speed
x = center_x + cos(angle) * radius
y = center_y + sin(angle) * radius
z = (note.start - current_time) * depth_scale
```

**Visual Properties:**
- Circular or spiral arrangement
- Particle effects
- Glow and bloom effects
- Dynamic camera movement

### 2D Horizontal (Partition View)

Traditional sheet music style horizontal scrolling.

**Coordinate Transformation:**
```
x = (note.start - current_time) * pixels_per_second + keyboard_x
y = pitch_to_staff_y(note.pitch)
width = note_duration * pixels_per_second
height = staff_line_spacing
```

**Visual Properties:**
- Horizontal scrolling
- Staff lines
- Note heads and stems
- Traditional music notation elements

### 2D Game Horizontal (Runner Style)

Side-scrolling platformer style where notes are obstacles or collectibles.

**Coordinate Transformation:**
```
x = (note.start - current_time) * scroll_speed + player_x
y = ground_level - note_height * (note.pitch % octave_count)
width = note_duration * scroll_speed
height = base_height + note.velocity * height_scale
```

**Visual Properties:**
- Character that runs/jumps
- Notes as platforms or obstacles
- Parallax background
- Game-like visual effects

## Implementation Guidelines

### Adding a New Technique

1. Create a new struct implementing `WaterfallRenderer` trait
2. Implement the required methods:
   - `update(time, layout)` - Update internal state
   - `render()` - Draw the visualization
   - `get_active_notes()` - Return currently playing notes
   - `should_be_pressed(note)` - Check if note should be pressed
   - `get_pressed_keys()` - Get list of keys to press

3. Register the technique in `WaterfallConfig`
4. Add selection logic in settings

### Performance Considerations

- Use viewport culling to only render visible notes
- Batch similar draw calls
- Use instanced rendering for many similar notes
- Cache computed positions when possible

### Coordinate System Notes

- Screen coordinates: (0,0) at top-left
- X increases right, Y increases down
- Keyboard at bottom of screen
- Time increases forward (notes with higher start time are future notes)
