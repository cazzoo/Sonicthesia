# Neothesia Settings UI Implementation Plan - Technical Edition

## Executive Summary
This comprehensive plan outlines the implementation of Neothesia's settings UI based on the "Sonic Obsidian" design system. The implementation will enhance the existing `PlySettingsScene` with new UI components while maintaining backward compatibility with the current configuration system.

### User Decisions & Requirements (Updated 2026-03-22)
**Critical Decisions Made:**
1. **Application Name**: Change from "Neothesia" to "Sonicthesia" throughout UI
2. **Implementation Priority**: Start with App Settings (General) page first
3. **Glassmorphism**: Implement full glassmorphism with solid background fallback
4. **Mobile Support**: Full responsive design (desktop-first, PLY/Macroquad focus)
5. **Design System**: Follow "Sonic Obsidian" concept with consistent branding

## 1. Current System Analysis

### 1.1 Existing Infrastructure (Validated)
**UI Framework**: PLY Engine - Custom immediate-mode UI framework built on Macroquad
- **Location**: `/ply-engine/src/lib.rs`
- **Features**: Flexbox-like layout, text input, shaders, accessibility, cross-platform

**Settings System**: Already implemented and functional
- **Config Location**: `/neothesia-core/src/config/mod.rs`
- **Storage**: RON format serialization
- **Structure**: Comprehensive categories (playback, appearance, devices, synth, waterfall, song library, piano theme)

**Settings Scene**: Existing `PlySettingsScene` in `/neothesia/src/scene/ply_scene.rs` (line 3237+)
- **Features**: Interactive settings with keyboard navigation, sliders, toggles, spinners, popups
- **Components**: ButtonArea, ToggleArea, SpinArea, SliderArea, StepperArea
- **Integration**: UnifiedInputManager for focus management

### 1.2 Technical Architecture
```rust
// Current Settings Structure (neothesia-core/src/config/mod.rs)
pub struct Config {
    playback: PlaybackConfigV1,      // Speed, wait mode, LUMI settings
    waterfall: WaterfallConfigV1,    // Animation speed, offset, note labels
    appearance: AppearanceConfigV1,  // Background color, color schemas, guidelines, glow
    devices: DevicesConfigV1,        // MIDI input/output, separate channels
    synth_config: SynthConfig,       // Soundfont paths, audio/playback/keyboard gains
    history: HistoryV1,              // Last opened song
    keyboard_layout: LayoutConfigV1, // Piano range
    song_library: SongLibraryConfig, // Directories, sorting, filters
    piano_theme: PianoThemeConfig,   // Theme name management
}
```

## 2. Design Inconsistencies Analysis

### 2.1 Branding Inconsistencies (Critical)
| Design File | Brand Name | User Profile | Navigation Structure |
|-------------|------------|--------------|---------------------|
| App Settings | "Sonic Obsidian Piano" | Generic user | Top nav + sidebar |
| Audio Settings | "Sonic Obsidian Piano" | "Pro User - Obsidian Tier" | Top nav + sidebar |
| Folder Settings | "Piano Studio" | "Pro Performer" | Sidebar only |
| Theme Settings | "Sonic Obsidian" | None shown | Top nav + sidebar |

**Resolution Required**: Consistent application name and user profile representation.

### 2.2 Layout Inconsistencies
1. **Top Navigation Bar**: Present in 3/4 designs, different styling
2. **Sidebar Width**: Varies between 256px and varies
3. **Settings Tab Placement**: Vertical tabs vs integrated sidebar navigation
4. **Content Padding**: Inconsistent margin/padding values

### 2.3 Component Inconsistencies
1. **Card Styles**: Glass panels vs solid backgrounds
2. **Button Styles**: Gradient vs solid vs ghost variants
3. **Color Pickers**: Different implementations (simpler vs advanced)
4. **Slider Designs**: Different track/thumb styling

## 3. UI Component Inventory

### 3.1 Components Already Implemented in PlySettingsScene
✅ **Interactive Elements**:
- `ButtonArea` - Click detection for buttons
- `ToggleArea` - Toggle switches with state
- `SpinArea` - Increment/decrement buttons
- `SliderArea` - Draggable sliders with min/max/step
- `StepperArea` - Multi-option selection
- `InteractiveSetting` - Keyboard navigation support

✅ **Popups/Selectors**:
- `SettingsPopup::OutputSelector` - Audio output device selection
- `SettingsPopup::InputSelector` - MIDI input device selection
- `SettingsPopup::ThemeSelector` - Theme selection popup

✅ **Input Management**:
- `UnifiedInputManager` - Focus, priority, cursor management
- Keyboard navigation (arrow keys, Enter/Space activation)
- Mouse interaction (hover, click, drag)

### 3.2 Components Missing from Design Specifications

#### High Priority (Core Settings UI)
1. **Advanced Color Picker**:
   - Color swatch with glow effect
   - Hex value display
   - Color presets grid
   - Gradient background preview

2. **Theme Preset Gallery**:
   - 4:3 aspect ratio cards
   - Image preview with gradient overlay
   - Active/hover states
   - Label positioning

3. **Glass Panel Container**:
   - Backdrop blur effect
   - Semi-transparent background
   - Border with low opacity

4. **Status Chips**:
   - "Live", "Default", "External" indicators
   - Tertiary color background
   - Full rounded corners

5. **Storage Usage Indicator**:
   - Progress bar with gradient
   - Text display (X GB / Y GB)
   - Color-coded based on usage

#### Medium Priority (Enhanced Functionality)
6. **Search Input with Suggestions**:
   - Auto-complete dropdown
   - Clear button
   - Search icon

7. **Notification Badge**:
   - Small dot indicator
   - Positioned on icons
   - Animated pulse effect

8. **Spectrum Visualizer**:
   - Audio frequency bars
   - Gradient coloring
   - Real-time animation

9. **Folder Path Display**:
   - Truncated path with ellipsis
   - File count metadata
   - Hover state expansion

10. **Tooltip System**:
    - Inverse surface background
    - Positioned near element
    - Delayed appearance

#### Low Priority (Polish & Extras)
11. **Loading Skeleton**:
    - Placeholder for content loading
    - Shimmer animation effect

12. **Breadcrumb Navigation**:
    - "Settings / Audio / Soundfonts" style
    - Clickable segments

13. **Tab Bar Component**:
    - For mobile/bottom navigation
    - Icon + label combinations

## 4. Implementation Strategy

### Phase 1: Foundation & Design System Integration (Week 1)
**Objective**: Establish design system in codebase and create foundational components

#### Tasks:
1. **Create Design Token Module**
   - Location: `/neothesia-core/src/design/mod.rs`
   - Content: Color constants, typography, spacing, shadow values
   - Integration: Export from neothesia-core
   - **IMPORTANT**: Use existing color constants from `ply_scene.rs` for consistency

2. **Implement Application-Wide Theme System**
   - Create `ThemeManager` in neothesia-core
   - Extend `PianoThemeConfig` to include `AppThemeConfig`
   - Define `AppTheme`, `ThemeColors`, `ThemeTypography` structures
   - Implement theme loading/saving via RON/JSON
   - Create system themes: Sonic Obsidian (default), Classic Light, Dark Pro, High Contrast

3. **Implement Basic Component Library**
   - Extend PLY Engine with custom renderers
   - Create: `glass_panel.rs`, `status_chip.rs`, `color_picker.rs`
   - Follow existing patterns in `ply_scene.rs`
   - **All components must use theme values, not hardcoded colors**

4. **Establish Consistent Branding**
   - Update all references to "Sonicthesia" (not "Neothesia", "Sonic Obsidian Piano", or "Piano Studio")
   - Create user profile placeholder component
   - Standardize navigation structure
   - Update top navigation and sidebar with consistent branding

5. **Create Component Testing Framework**
   - Visual regression testing
   - Interaction testing
   - Performance benchmarking
   - Theme consistency testing

### Phase 2: Core Settings Components (Week 2-3)
**Objective**: Implement missing UI components from design specifications

#### High Priority Components:
1. **Advanced Color Picker**
   - 48px × 48px color swatch with glow
   - Hex value display in monospace
   - Preset color grid (8-12 colors)
   - Gradient background preview

2. **Theme Preset Gallery**
   - Grid layout (4 columns on desktop)
   - 4:3 aspect ratio cards
   - Image preview area showing **application-wide theme** (not just piano keyboard)
   - Active state with 2px primary border
   - Hover scale animation
   - **Shows full UI preview**: sidebar, buttons, cards, etc.

3. **Glass Panel Container**
   - Semi-transparent background (60% opacity)
   - 20px backdrop blur
   - Border with 10% opacity
   - Consistent padding (24px)

4. **Enhanced Slider Component**
   - Track: surface-container-highest, 4px height
   - Thumb: 16px circle with primary glow
   - Value display: monospace, primary color
   - Hover/focus states

#### Integration Tasks:
5. **Update PlySettingsScene Rendering**
   - Replace custom rendering with component library
   - Maintain existing interactive functionality
   - Add missing sections from design mockups

6. **Implement Settings Sections**
   - MIDI Setup section (device selection, latency, pedals)
   - Audio/Soundfont section (engine config, mixer)
   - Theme section (application-wide themes, color customization, typography, effects)
   - Folder section (directories, soundfont folders)

### Phase 3: Advanced Features & Polish (Week 4)
**Objective**: Add advanced components and polish user experience

#### Tasks:
1. **Search Functionality**
   - Search input component
   - Auto-complete suggestions
   - Integration with settings filtering

2. **Notification System**
   - Toast notification component
   - Badge system for alerts
   - Animation effects

3. **Storage Visualization**
   - Progress bar with gradient
   - Usage statistics display
   - Color-coded warnings

4. **Audio Spectrum Visualizer**
   - Real-time frequency bars
   - Gradient coloring
   - Integration with audio engine

5. **Responsive Design**
   - Mobile adaptation of settings
   - Bottom navigation for mobile
   - Collapsible sections

### Phase 4: Testing & Optimization (Week 5)
**Objective**: Ensure quality, performance, and accessibility

#### Tasks:
1. **Visual Testing**
   - Compare with design mockups pixel-by-pixel
   - Color accuracy verification
   - Typography consistency check

2. **Performance Testing**
   - Settings UI impact on MIDI playback
   - Memory usage optimization
   - Frame rate consistency

3. **Accessibility Testing**
   - Keyboard navigation completeness
   - Screen reader compatibility
   - Color contrast verification

4. **Cross-platform Testing**
   - Windows, macOS, Linux
   - Different screen resolutions
   - High DPI scaling

## 5. Technical Implementation Details

### 5.1 Component Architecture
```rust
// Example: Theme Preset Card Component
pub struct ThemePresetCard {
    pub id: String,
    pub name: String,
    pub preview_image: Option<Texture2D>,
    pub is_active: bool,
    pub colors: ThemeColors,
}

impl ThemePresetCard {
    pub fn render(&self, ui: &mut Ui, position: Vec2, size: Vec2) {
        // Glass panel background
        ui.element()
            .width(layout::Sizing::Fixed(size.x))
            .height(layout::Sizing::Fixed(size.y))
            .background_color(if self.is_active {
                COLOR_PRIMARY.with_alpha(0.2)
            } else {
                COLOR_SURFACE_CONTAINER
            })
            .corner_radius(8.0)
            // Active border
            .border(if self.is_active { 2.0 } else { 0.0 })
            .border_color(COLOR_PRIMARY)
            // ... render content
    }
}
```

### 5.2 Integration with Existing Config
```rust
// Extend PlySettingsScene to use new components
impl PlySettingsScene {
    pub fn render_theme_section(&mut self, ctx: &mut MacroquadContext) {
        // Use new ThemePresetCard component
        for theme in THEMES {
            let card = ThemePresetCard::new(theme);
            card.render(ui, position, size);
            
            if card.is_clicked() {
                ctx.config.set_piano_theme_name(theme.name.to_string());
                ctx.config.save();
            }
        }
    }
}
```

### 5.3 Performance Considerations
1. **Caching**: Cache rendered textures for theme previews
2. **Lazy Loading**: Load high-res previews only when visible
3. **Batch Rendering**: Group similar elements for efficient drawing
4. **Memory Management**: Proper texture cleanup

## 5.5 Visual Diagrams & Layout

### 5.5.1 Settings Page Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ Top Navigation Bar (64px height, glassmorphism)             │
│ ┌─────────────┐  ┌──────────────────────────────────────┐  │
│ │ Sonicthesia │  │ Library  Practice  Settings  [User]  │  │
│ └─────────────┘  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Main Content Area (flex layout)                             │
│ ┌─────────────┐ ┌────────────────────────────────────────┐ │
│ │   Sidebar   │ │           Settings Content             │ │
│ │   256px     │ │                                        │ │
│ │             │ │  ┌──────────────────────────────────┐  │ │
│ │ ○ General   │ │  │ MIDI Setup Section               │  │ │
│ │   MIDI      │ │  │ ┌──────────┐ ┌──────────────┐   │  │ │
│ │   Audio     │ │  │ │ Device   │ │ Latency      │   │  │ │
│ │   Themes    │ │  │ │ Select   │ │ Slider       │   │  │ │
│ │   Folders   │ │  │ └──────────┘ └──────────────┘   │  │ │
│ │             │ │  └──────────────────────────────────┘  │ │
│ │             │ │                                        │ │
│ │             │ │  ┌──────────────────────────────────┐  │ │
│ │             │ │  │ Themes & Customization           │  │ │
│ │             │ │  │ ┌────┐ ┌────┐ ┌────┐ ┌────┐    │  │ │
│ │             │ │  │ │ T1 │ │ T2 │ │ T3 │ │ T4 │    │  │ │
│ │             │ │  │ └────┘ └────┘ └────┘ └────┘    │  │ │
│ │             │ │  └──────────────────────────────────┘  │ │
│ │             │ │                                        │ │
│ └─────────────┘ └────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 5.5.2 Component Hierarchy
```
PlySettingsScene
├── TopNavigationBar (glassmorphism)
│   ├── LogoText ("Sonicthesia")
│   ├── NavigationLinks[]
│   └── UserProfileButton
├── SidebarNavigation (256px width)
│   ├── SectionTitle ("Settings")
│   ├── SettingsTabs[]
│   │   ├── GeneralTab (active)
│   │   ├── MIDITab
│   │   ├── AudioTab
│   │   ├── ThemesTab
│   │   └── FoldersTab
│   └── StorageUsageIndicator
└── MainContentArea (scrollable)
    ├── SectionHeader ("General Settings")
    ├── GlassPanelContainer
    │   ├── DeviceSelectionCard
    │   ├── LatencySlider
    │   └── PedalSettingsToggle
    ├── ThemePresetGallery
    │   ├── ThemePresetCard[4]
    │   └── ThemeCustomizationForm
    └── DirectoryManagementSection
        ├── FolderList
        └── AddFolderButton
```

### 5.5.3 Implementation Flow
```
Phase 1: Foundation
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Design Tokens   │───>│ Component Base  │───>│ PlySettingsScene│
│ (Colors, Fonts) │    │ (Glass, Chip)   │    │ Integration     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
Phase 2: Core Components
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ ColorPicker     │───>│ ThemeGallery    │───>│ AppSettingsPage │
│ (Advanced)      │    │ (4 presets)     │    │ (General Tab)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
Phase 3: Advanced Features
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ SearchBar       │───>│ SpectrumVisual  │───>│ ResponsiveDesign│
│ (Auto-complete) │    │ (Real-time)     │    │ (Mobile adapt)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 5.5.4 Settings Tab Layout
```
┌─────────────────────────────────────────────────────────────┐
│ App Settings (General) - Current Implementation Plan        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ MIDI Setup Section                                      │ │
│ │                                                         │ │
│ │ Input Device: [Obsidian Pro MKII (Connected) ▼]         │ │
│ │                                                         │ │
│ │ Latency Compensation: ──●───────────── (12ms)           │ │
│ │                         0ms        100ms                │ │
│ │                                                         │ │
│ │ Pedal Response:                                         │ │
│ │   [●] Invert Sustain (CC64)                             │ │
│ │   [○] Continuous Expression                             │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Application Theme & Customization                        │ │
│ │                                                         │ │
│ │ Theme Presets (Application-Wide):                       │ │
│ │ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐           │ │
│ │ │████████│ │░░░░░░░░│ │░░░░░░░░│ │░░░░░░░░│           │ │
│ │ │████████│ │░░░░░░░░│ │░░░░░░░░│ │░░░░░░░░│           │ │
│ │ │Sonic   │ │Classic │ │Dark    │ │High    │           │ │
│ │ │Obsidian│ │Light   │ │Pro     │ │Contrast│           │ │
│ │ └────────┘ └────────┘ └────────┘ └────────┘           │ │
│ │                                                         │ │
│ │ Custom Theme Editor:                                    │ │
│ │ Primary Color:  ●●●●● (5 preset colors)               │ │
│ │ Background:     ████ (color picker)                    │ │
│ │ Typography:     [Space Grotesk ▼] [Inter ▼]            │ │
│ │ Effects:        Glow [━━●━━] Blur [━━●━━]              │ │
│ │                                                         │ │
│ │ Live Preview:                                           │ │
│ │ [Sidebar preview] [Button preview] [Card preview]       │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Directory Management                                    │ │
│ │                                                         │ │
│ │ MIDI Libraries:                                         │ │
│ │ ┌─────────────────────────────────────────────────────┐ │ │
│ │ │ 📁 C:/Users/Producer/Music/MIDI_Library            [×]│ │ │
│ │ │    Default source for recordings and imports           │ │ │
│ │ └─────────────────────────────────────────────────────┘ │ │
│ │ ┌─────────────────────────────────────────────────────┐ │ │
│ │ │ 📁 D:/Samples/Piano_Soundfonts                   [×]│ │ │
│ │ │    Global Soundfont directory (.sf2)                 │ │ │
│ │ └─────────────────────────────────────────────────────┘ │ │
│ │                                                         │ │
│ │ [＋] Add New Directory                                  │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Final Actions                                           │ │
│ │                                                         │ │
│ │ [↺ Reset to Default]        [Cancel] [Save Changes]     │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 5.5.5 Responsive Design Breakpoints
```
Desktop (>1200px):
┌─────────────────────────────────────────────────────────────┐
│ Sidebar │                    Content Area                   │
│  256px  │                    (flex: 1)                      │
└─────────────────────────────────────────────────────────────┘

Tablet (768px - 1200px):
┌─────────────────────────────────────────────────────────────┐
│                Collapsible Sidebar                          │
├─────────────────────────────────────────────────────────────┤
│                    Content Area                             │
│                    (full width)                             │
└─────────────────────────────────────────────────────────────┘

Mobile (<768px):
┌─────────────────────────────────────────────────────────────┐
│                    Top Navigation                           │
├─────────────────────────────────────────────────────────────┤
│                    Content Area                             │
│                    (full width)                             │
│                    (scrollable)                             │
├─────────────────────────────────────────────────────────────┤
│ Bottom Navigation: [Settings] [Library] [Practice] [Profile]│
└─────────────────────────────────────────────────────────────┘
```

### 5.5.6 Component States
```
Button States:
Normal:    ┌─────────────┐  Hover:     ┌─────────────┐
           │ Save Changes │            │ Save Changes │
           └─────────────┘            └─────────────┘
                                     (scale 1.02, glow+)

Active:    ┌─────────────┐  Disabled:  ┌─────────────┐
           │ Save Changes │            │ Save Changes │
           └─────────────┘            └─────────────┘
           (scale 0.95)               (opacity 50%)

Toggle States:
Off:       ┌───○───────┐  On:        ┌───────●───┐
           └───────────┘            └───────────┘
           (surface container)      (primary bg, glow)

Card States:
Normal:    ┌─────────────┐  Hover:     ┌─────────────┐
           │   Theme 1   │            │   Theme 1   │
           └─────────────┘            └─────────────┘
           (solid bg)                 (bg shift, border)

Active:    ┌─────────────┐
           │   Theme 1   │
           └─────────────┘
           (2px primary border, glow)
```

---

## 5.6 Application-Wide Theme System

### 5.6.1 Theme Architecture Overview
The theming system must support **application-wide theming**, not just piano keyboard themes. This includes:
- **UI Colors**: All color tokens (primary, secondary, tertiary, surfaces, text)
- **Typography**: Font families, sizes, weights
- **Spacing**: Layout spacing scales
- **Effects**: Glow intensities, blur amounts, animation speeds
- **Component Styles**: Button styles, card styles, input styles

### 5.6.2 Integration with Existing Config System
Extend the existing `PianoThemeConfig` to include full application themes:
```rust
// Extend existing config in neothesia-core/src/config/model.rs
pub struct AppConfigV2 {
    // Existing fields...
    pub piano_theme: PianoThemeConfig,
    
    // New: Application theme
    pub app_theme: AppThemeConfig,
}

pub struct AppThemeConfig {
    /// Selected theme ID
    pub theme_id: String,
    /// Custom theme overrides (optional)
    pub custom_colors: Option<ThemeColors>,
}
```

### 5.6.3 Theme Management Flow
```
User selects theme in UI
    ↓
ThemeManager.apply_theme(theme_id)
    ↓
Config saves theme preference
    ↓
All UI components re-render with new theme
    ↓
Piano keyboard updates with theme colors
    ↓
Settings UI updates with theme colors
```

### 5.6.4 Predefined Themes
1. **Sonic Obsidian** (default): Purple/blue/pink neon dark theme
2. **Classic Light**: Light theme for daytime use
3. **Dark Pro**: Professional dark theme with less saturation
4. **High Contrast**: Accessibility-focused theme

### 5.6.5 Theme Creation UI in Settings
The Theme Settings page should include:
1. **Theme Gallery**: Visual preview of all themes
2. **Theme Editor**: Color pickers, typography selectors
3. **Live Preview**: Real-time theme application
4. **Import/Export**: Share themes as JSON files
5. **Reset to Default**: Restore system themes

### 5.6.6 Component Theming Implementation
All UI components must use theme values, not hardcoded colors:
```rust
// Example: Button uses theme colors
ui.element()
    .background_color(theme.colors().primary)  // Not hardcoded COLOR_PRIMARY
    .corner_radius(8.0)
    // ...
```

---

## 6. Risk Assessment & Mitigation

### 6.1 Technical Risks
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| PLY Engine limitations | High | Medium | Extend with custom renderers, fallback to simpler designs |
| Performance degradation | High | Low | Profile early, implement caching, lazy loading |
| Cross-platform issues | Medium | Medium | Test early on all platforms, use conditional compilation |
| Config format changes | Medium | Low | Maintain backward compatibility, version migration |

### 6.2 Design Risks
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Glassmorphism performance | Medium | High | Fallback to solid backgrounds, reduce blur radius |
| Complex animations | Medium | Medium | Respect prefers-reduced-motion, provide alternatives |
| Color contrast issues | High | Low | Use design system tokens, test with contrast checkers |

### 6.3 Implementation Risks
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Scope creep | High | High | Stick to phased approach, clear prioritization |
| Integration complexity | Medium | Medium | Incremental integration, thorough testing |
| Backward compatibility | Medium | Low | Version config, migration scripts |

## 7. Success Metrics & Validation

### 7.1 Visual Fidelity
- **Target**: 95% match with design mockups
- **Measurement**: Visual regression testing, pixel comparison
- **Acceptance**: No critical visual discrepancies

### 7.2 Performance Impact
- **Target**: < 5% FPS drop in settings UI vs main application
- **Measurement**: Frame rate monitoring, memory profiling
- **Acceptance**: No noticeable performance degradation

### 7.3 User Experience
- **Target**: Complete keyboard navigation, accessible to screen readers
- **Measurement**: Accessibility audits, user testing
- **Acceptance**: WCAG 2.1 AA compliance

### 7.4 Code Quality
- **Target**: Maintainable, extensible component library
- **Measurement**: Code review, documentation coverage
- **Acceptance**: 90%+ test coverage for new components

## 8. Resource Requirements

### 8.1 Development Time
- **Phase 1**: 1 week (40 hours)
- **Phase 2**: 2 weeks (80 hours)
- **Phase 3**: 1 week (40 hours)
- **Phase 4**: 1 week (40 hours)
- **Total**: 5 weeks (200 hours)

### 8.2 Dependencies
1. **PLY Engine**: Already included, may need extensions
2. **Macroquad**: Already included for rendering
3. **Design Assets**: Need actual theme preview images
4. **Audio Library**: For spectrum visualizer integration

### 8.3 Testing Requirements
1. **Platforms**: Windows 10/11, macOS 12+, Ubuntu 20.04+
2. **Hardware**: Various GPU vendors, integrated/dedicated
3. **Screen Sizes**: 1366×768 to 3840×2160
4. **Accessibility Tools**: Screen readers, keyboard-only navigation

## 9. Next Steps & Immediate Actions

### 9.1 Before Implementation Starts
1. ✅ **Design System Document**: Created (`/design/DESIGN_SYSTEM.md`)
2. ✅ **Technical Analysis**: Completed (existing system validated)
3. 🔄 **Uncertainty Resolution**: Need user input on branding/design choices
4. ⏳ **Component Prioritization**: Finalize which components to implement first

### 9.2 User Decisions Required
1. **Application Name**: "Neothesia" vs "Sonic Obsidian" vs "Piano Studio"
2. **Design Priority**: Which settings page to implement first?
3. **Feature Scope**: Advanced features (spectrum, search) vs core settings only
4. **Performance vs Fidelity**: Glassmorphism effects vs performance impact
5. **Mobile Support**: Full mobile adaptation or basic responsiveness

### 9.3 Immediate Next Actions
1. **Resolve uncertainties** with user
2. **Create component library** foundation
3. **Update PlySettingsScene** with first component (Color Picker)
4. **Implement Theme Preset Gallery** (most requested feature)
5. **Add Glass Panel** component for consistent styling

## 10. Conclusion

This implementation plan provides a structured approach to enhancing Neothesia's settings UI while leveraging existing infrastructure. The phased approach ensures manageable development cycles, early validation, and continuous improvement.

The existing `PlySettingsScene` provides a solid foundation with interactive elements and keyboard navigation. The plan extends this with missing components from the design specifications while maintaining performance and accessibility standards.

**Key Success Factors**:
1. **Leverage existing infrastructure** rather than building from scratch
2. **Follow the design system** strictly for consistency
3. **Implement incrementally** to validate approach early
4. **Test thoroughly** at each phase
5. **Maintain backward compatibility** with existing configs
6. **Support application-wide theming** (not just piano keyboard themes)

**Expected Outcome**: A modern, visually stunning settings interface for "Sonicthesia" that matches the "Sonic Obsidian" design vision while maintaining performance and cross-platform compatibility. The implementation includes an **application-wide theme system** that allows users to customize the entire UI (colors, typography, spacing) through the Theme Settings page, not just the piano keyboard appearance. The implementation will start with the App Settings (General) page and expand to other settings pages in subsequent phases.

**User Requirements Integration**:
1. ✅ **Application Name**: Change to "Sonicthesia" throughout UI
2. ✅ **Implementation Priority**: App Settings (General) page first
3. ✅ **Glassmorphism**: Full implementation with solid fallback
4. ✅ **Mobile Support**: Full responsive design (desktop-first approach)
5. ✅ **Design System**: "Sonic Obsidian" concept with consistent branding

---

**Status**: Technical Plan v1.1 - Ready for Implementation  
**Next Milestone**: Phase 1 kickoff (Foundation & Design System Integration)  
**Estimated Completion**: 5 weeks from implementation start  
**Document Location**: `/home/caz/Documents/Neothesia/.sisyphus/plans/settings-implementation-plan.md`  

**Implementation Ready**: All uncertainties resolved, design system documented, visual diagrams created. **Theme consistency ensured**: All colors, fonts, and design tokens match existing codebase constants.