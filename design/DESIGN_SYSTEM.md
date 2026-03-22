# Sonicthesia Design System: "The Sonic Obsidian"

## 1. Design Philosophy

### Creative North Star: "The Sonic Obsidian"
This design system moves beyond the "gamer aesthetic" into the realm of high-end, editorial performance software. We are not just building a tool; we are building an instrument. The "Sonic Obsidian" concept treats the screen as a dark, infinite stage where information does not sit *on* the surface but glows *within* it.

**Note**: This design system supports **application-wide theming**, not just piano keyboard themes. Users can customize colors, typography, spacing, and effects for the entire UI through the Theme Settings page.

### Core Principles
1. **Intentional Asymmetry**: Use spacing scale to create breathing room (e.g., massive `24` unit gutter) to evoke premium physical mixing console feel
2. **Depth Through Layering**: Overlapping elements (Glassmorphism panels partially obscuring visualizations) create 3D depth
3. **Neon Radiance**: Information glows from within the void, not painted on top
4. **Editorial Hierarchy**: Extreme contrast between display and functional text
5. **"No-Line" Rule**: Structural definition through background shifts, not explicit borders

---

## 2. Color System: "The Radiance of the Void"

### Base Palette
```css
/* The Vacuum */
--background: #0e0e13;           /* Deep black base */
--surface: #0e0e13;              /* Same as background */
--surface-dim: #0e0e13;          /* Dimmed surface */

/* Surface Hierarchy (Layers of Smoke) */
--surface-container-lowest: #000000;
--surface-container-low: #131318;
--surface-container: #19191f;
--surface-container-high: #1f1f26;
--surface-container-highest: #25252c;

/* Neon Accents */
--primary: #db90ff;              /* Neon purple */
--primary-dim: #d277ff;          /* Dimmed purple */
--primary-container: #d37bff;    /* Purple container */
--primary-fixed: #d37bff;        /* Fixed purple */
--primary-fixed-dim: #cb66fe;    /* Fixed dimmed purple */

--secondary: #5f9eff;            /* Neon blue */
--secondary-dim: #0073e0;        /* Dimmed blue */
--secondary-container: #005db8;  /* Blue container */

--tertiary: #ff6e80;             /* Neon pink */
--tertiary-dim: #e21d4d;         /* Dimmed pink */
--tertiary-container: #fc345d;   /* Pink container */

/* Text & Content */
--on-surface: #f8f5fd;           /* Near-white (NOT pure white) */
--on-surface-variant: #acaab1;   /* Muted text */
--on-background: #f8f5fd;        /* Text on background */

/* Functional Colors */
--error: #ff6e84;                 /* Neon red error */
--error-dim: #d73357;            /* Dimmed error */
--error-container: #a70138;      /* Error container */

/* Outline (Ghost Borders) */
--outline: #76747b;
--outline-variant: #48474d;      /* For ghost borders */
```

### The "Glass & Gradient" Rule
Standard flat buttons are forbidden for primary actions. Use:
- **Primary Actions**: Linear gradient from `primary` (#db90ff) to `primary_container` (#d37bff) at 135-degree angle
- **Floating Overlays**: `surface_variant` at 60% opacity with 20px backdrop blur for glassmorphism
- **Ghost Borders**: 1px stroke of `outline_variant` (#48474d) at 20% opacity - felt, not seen

---

## 3. Typography: "The Editorial Tech"

### Font Pairing
- **Display & Headlines**: Space Grotesk (geometric precision, high-impact)
- **Functional Data**: Inter (Swiss-style clarity, excellent for technical readouts)

### Type Scale
```css
/* Display (Space Grotesk) */
--display-lg: 48px / bold;       /* Poster-like track titles */
--display-md: 36px / bold;       
--display-sm: 24px / bold;

/* Headlines (Space Grotesk) */
--headline-lg: 32px / bold;
--headline-md: 24px / bold;      /* Section headers */
--headline-sm: 20px / bold;

/* Body (Inter) */
--body-lg: 18px / regular;
--body-md: 16px / regular;       /* Default body text */
--body-sm: 14px / regular;

/* Labels (Inter, often monospace for alignment) */
--label-lg: 16px / medium;
--label-md: 14px / medium;
--label-sm: 12px / medium;
--label-xs: 10px / bold;         /* Small technical readouts */
```

### Hierarchy Principle
Use extreme contrast:
- `display-lg` title paired with `label-sm` subtitle in `on_surface_variant`
- Creates authoritative, editorial hierarchy

---

## 4. Spacing & Layout: "The Expansive Premium"

### Spacing Scale (4px base unit)
```
xs: 4px
sm: 8px
md: 12px
lg: 16px
xl: 24px
2xl: 32px
3xl: 48px
4xl: 64px
```

### Layout Principles
- **Outer Margins**: Use 24px (6rem) or 20px (5rem) for expansive premium feel
- **Card Padding**: 24px standard for main containers
- **Section Spacing**: 32px between major sections
- **Component Gaps**: 8px-16px between related elements

### Roundedness
- **Small Elements** (chips, tags): `9999px` (full)
- **Standard Containers**: `0.25rem` (4px) to `0.5rem` (8px)
- **Large Cards**: `0.75rem` (12px)
- **NEVER use** `0px` or sharp corners

---

## 5. Elevation & Depth: "Tonal Layering"

### The Layering Principle
Depth achieved by stacking `surface-container` tiers:
1. **Base Layer**: `surface` (#0e0e13) - the void
2. **Primary Workspaces**: `surface_container` (#19191f) - raised slightly
3. **Interactive Modules**: `surface_container_highest` (#25252c) - closest to user

Nesting `surface_container_highest` card inside `surface_container_low` section creates natural "lift".

### Shadows & Glows
**NO DROP SHADOWS**. Use "Glows" to represent energy:
- **Ambient Glow**: Primary color (#db90ff) with 32px blur, 0px offset, 6% opacity
- **Active State Glow**: Primary 1px border with `primary_dim` outer glow (4px blur)
- **Hover Glow**: Increase glow opacity to 10-15%

### The "Ghost Border" Fallback
If tactile edge needed for accessibility:
- 1px stroke of `outline_variant` (#48474d) at 20% opacity
- Should be felt, not seen

---

## 6. Component Specifications

### 6.1 Buttons

#### Primary Button
- **Background**: Gradient (`primary` → `primary_container`)
- **Text**: `on-primary-fixed` (#000000)
- **Height**: 48px
- **Padding**: 24px horizontal, 12px vertical
- **Border Radius**: 8px
- **Shadow**: `primary` glow 20px blur, 20% opacity
- **Hover**: Scale 1.02, increase shadow
- **Active**: Scale 0.95

#### Secondary Button
- **Background**: Transparent
- **Border**: 1px `outline_variant` ghost border
- **Text**: `primary`
- **Hover**: Fill with `primary` at 10% opacity
- **Height**: 48px
- **Padding**: 24px horizontal

#### Tertiary Button
- **Background**: None
- **Text**: `primary_dim`
- **Height**: Auto
- **Padding**: 8px horizontal

#### Icon Button
- **Size**: 48px × 48px
- **Background**: Transparent
- **Hover**: Fill with `surface-container-highest`
- **Border Radius**: 50% (full)

### 6.2 Form Controls

#### Input Fields
- **Background**: `surface_container_highest`
- **Border**: None
- **Focus Ring**: 1px `secondary` (#5f9eff) ghost border + subtle glow
- **Text**: `on-surface`, `title-sm` for legibility
- **Height**: 48px
- **Padding**: 16px horizontal

#### Dropdowns
- **Same as Input Fields** with dropdown arrow
- **Expanded Background**: `surface-container` with 20px blur

#### Sliders
- **Track**: `surface-container-highest`, 4px height
- **Thumb**: 16px circle, `primary` with glow
- **Value Display**: Monospace font, `primary` color
- **Height**: 40px (including label space)

#### Toggles
- **Track**: 40px × 20px, `surface-container-highest`
- **Thumb**: 16px circle, `on-surface-variant`
- **Active Track**: `primary` at 20% opacity
- **Active Thumb**: `primary` with glow

#### Color Pickers
- **Color Swatch**: 40px × 40px, full rounded
- **Hex Display**: Monospace font
- **Container**: `surface-container-highest` with border

### 6.3 Cards & Containers

#### Glass Panel
- **Background**: `surface_variant` at 60% opacity
- **Backdrop Blur**: 20px
- **Border**: 1px `outline_variant` at 10% opacity
- **Padding**: 24px
- **Border Radius**: 12px

#### Surface Container
- **Background**: `surface-container`
- **Padding**: 24px
- **Border Radius**: 8px
- **Border Left Accent**: 2px `tertiary` for active/playing status

#### Performance Card
- **NO DIVIDERS** - use 6px vertical gap
- **Background**: `surface_container_highest`
- **Left Accent**: 2px `tertiary` for active status
- **Typography**: `headline-md` for title, `label-sm` for metadata

### 6.4 Navigation

#### Top Navigation Bar
- **Height**: 64px
- **Background**: `background` with 80% opacity + blur
- **Shadow**: 0 0 20px `primary` at 15% opacity
- **Logo**: `primary` color, Space Grotesk font
- **Links**: `on-surface-variant`, hover to `on-surface`
- **Active Link**: `primary` with underline

#### Sidebar Navigation
- **Width**: 256px
- **Background**: `surface-container-low`
- **Border Right**: `outline_variant` at 10% opacity
- **Item Height**: 48px
- **Icon**: `material-symbols-outlined`
- **Active Item**: `surface-container-highest` background, `primary` text, left border accent `tertiary`

#### Settings Tabs (Vertical)
- **Same as Sidebar Navigation** but with settings-specific icons
- **Active Tab**: `surface-container-highest`, `primary` text, left border `tertiary`

### 6.5 Lists & Tables

#### List Item
- **Height**: 64px
- **Padding**: 16px horizontal
- **Background**: `surface-container-high`
- **Hover**: `surface-container-highest`
- **Border Bottom**: 1px gap revealing `background`
- **Icon**: Left-aligned, 24px
- **Primary Text**: `on-surface`
- **Secondary Text**: `on-surface-variant`
- **Action Buttons**: Right-aligned, opacity 0 → 100% on hover

#### Folder Item
- **Special**: Icon color indicates type (folder, music, USB)
- **Secondary Info**: File count, last scanned time
- **Delete Button**: Appears on hover, `error` color

### 6.6 Feedback Components

#### Toast Notification
- **Background**: Glass panel (60% opacity, 20px blur)
- **Border**: 1px `primary` at 30% opacity
- **Shadow**: 32px `primary` glow at 10% opacity
- **Position**: Fixed bottom-right, 40px from edges
- **Height**: 48px
- **Icon**: Animated pulse indicator

#### Status Chip
- **Background**: `tertiary` at 10% opacity
- **Text**: `tertiary`, uppercase, bold
- **Border Radius**: 9999px (full)
- **Height**: 24px
- **Padding**: 8px horizontal

#### Progress Bar
- **Track**: `surface-container-highest`, 4px height
- **Fill**: `primary` gradient
- **Background Glow**: `primary` at 20% opacity

### 6.7 Advanced Components

#### Color Picker (Advanced)
- **Swatch**: 48px × 48px, full rounded, with glow
- **Hex Display**: Monospace, on-surface text
- **Container**: `surface-container-highest`
- **Border**: 1px `outline-variant` at 10% opacity

#### Theme Preset Card
- **Aspect Ratio**: 4:3
- **Background**: Image with gradient overlay
- **Active State**: 2px `primary` border
- **Hover State**: Scale 1.02, opacity transition
- **Label**: Bottom-aligned, Space Grotesk font

#### Spectrum Visualizer
- **Container**: `surface-container-low`, rounded
- **Bars**: `primary` gradient for high frequencies, `secondary` for low
- **Height**: 96px
- **Gap**: 4px between bars

---

## 7. Animation & Motion

### Transitions
- **Duration**: 200ms standard, 300ms for complex transitions
- **Easing**: `ease-out` for entrances, `ease-in` for exits
- **Transform**: `scale()` for interactive feedback

### Hover Effects
- **Buttons**: Scale 1.02, glow increase
- **Cards**: Scale 1.01, background shift
- **List Items**: Background transition

### Active/Press Effects
- **Buttons**: Scale 0.95
- **Toggles**: Scale 0.98

### Focus States
- **Visible**: 2px `primary` outline with glow
- **Duration**: 150ms

---

## 8. Accessibility Guidelines

### Color Contrast
- **NEVER use pure white** (#FFFFFF) - always use `on_surface` (#f8f5fd)
- **Minimum Contrast**: 4.5:1 for normal text, 3:1 for large text
- **Focus Indicators**: Clearly visible against all backgrounds

### Keyboard Navigation
- **All interactive elements** must be focusable
- **Tab Order**: Logical, follows visual flow
- **Focus Rings**: Visible, 2px with glow

### Screen Reader Support
- **ARIA Labels**: For all icons, buttons, and interactive elements
- **Semantic HTML**: Where applicable
- **Alt Text**: For all meaningful images

### Motion Sensitivity
- **Respect** `prefers-reduced-motion`
- **Provide** alternative feedback methods
- **Disable** animations when requested

---

## 9. Implementation Guidelines

### For PLY Engine Implementation
1. **Use** existing PlyScene trait and component system
2. **Extend** with custom rendering for glassmorphism effects
3. **Implement** theme system using existing config structure
4. **Follow** existing patterns for input handling and focus management

### For Settings Implementation
1. **Leverage** existing Config struct in `neothesia-core`
2. **Extend** PlySettingsScene with new UI components
3. **Maintain** backward compatibility with RON config format
4. **Implement** incremental UI improvements, not full rewrite

### Testing & Validation
1. **Visual Testing**: Compare with design mockups
2. **Interaction Testing**: All components respond correctly
3. **Performance Testing**: No degradation during MIDI playback
4. **Accessibility Testing**: Keyboard navigation, screen reader compatibility

---

## 10. Theming System: "Application-Wide Themeability"

### 10.1 Theme Architecture Overview
The theming system must support **application-wide theming**, not just piano keyboard themes. This includes:
- **UI Colors**: All color tokens (primary, secondary, tertiary, surfaces, text)
- **Typography**: Font families, sizes, weights
- **Spacing**: Layout spacing scales
- **Effects**: Glow intensities, blur amounts, animation speeds
- **Component Styles**: Button styles, card styles, input styles

### 10.2 Theme Data Structure
```rust
/// Complete application theme definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppTheme {
    /// Theme identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Theme description
    pub description: String,
    /// Color palette
    pub colors: ThemeColors,
    /// Typography settings
    pub typography: ThemeTypography,
    /// Spacing scale
    pub spacing: ThemeSpacing,
    /// Effect settings
    pub effects: ThemeEffects,
    /// Whether this is a system theme (cannot be deleted)
    pub is_system: bool,
}

/// Color palette for a theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: (u8, u8, u8),
    pub surface: (u8, u8, u8),
    pub surface_container_low: (u8, u8, u8),
    pub surface_container: (u8, u8, u8),
    pub surface_container_high: (u8, u8, u8),
    pub surface_container_highest: (u8, u8, u8),
    pub primary: (u8, u8, u8),
    pub primary_container: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
    pub secondary_container: (u8, u8, u8),
    pub tertiary: (u8, u8, u8),
    pub tertiary_container: (u8, u8, u8),
    pub on_surface: (u8, u8, u8),
    pub on_surface_variant: (u8, u8, u8),
    pub error: (u8, u8, u8),
    pub error_container: (u8, u8, u8),
}

/// Typography settings for a theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub display_font: String,
    pub body_font: String,
    pub label_font: String,
    pub monospace_font: String,
    // Font sizes could be scaled relative to base size
    pub font_scale: f32,
}

/// Spacing scale for a theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

/// Effect settings for a theme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeEffects {
    pub glow_intensity: f32,
    pub blur_radius: f32,
    pub animation_speed: f32,
    pub glassmorphism_enabled: bool,
}
```

### 10.3 Predefined Themes
```rust
/// Built-in system themes
pub enum SystemTheme {
    /// Default "Sonic Obsidian" theme (purple/blue/pink neon)
    SonicObsidian,
    /// Classic light theme
    ClassicLight,
    /// Dark professional theme
    DarkPro,
    /// High contrast theme for accessibility
    HighContrast,
    /// Custom user themes stored in config
    Custom(String),
}

impl SystemTheme {
    pub fn all() -> Vec<AppTheme> {
        vec![
            Self::sonic_obsidian(),
            Self::classic_light(),
            Self::dark_pro(),
            Self::high_contrast(),
        ]
    }
    
    fn sonic_obsidian() -> AppTheme {
        AppTheme {
            id: "sonic_obsidian".to_string(),
            name: "Sonic Obsidian".to_string(),
            description: "The original neon dark theme".to_string(),
            colors: ThemeColors {
                background: (14, 14, 19),
                surface: (14, 14, 19),
                surface_container_low: (19, 19, 24),
                surface_container: (25, 25, 31),
                surface_container_high: (31, 31, 38),
                surface_container_highest: (37, 37, 44),
                primary: (219, 144, 255),
                primary_container: (211, 123, 255),
                secondary: (95, 158, 255),
                secondary_container: (0, 93, 184),
                tertiary: (255, 110, 128),
                tertiary_container: (252, 52, 93),
                on_surface: (248, 245, 253),
                on_surface_variant: (172, 170, 177),
                error: (255, 110, 132),
                error_container: (167, 1, 56),
            },
            typography: ThemeTypography {
                display_font: "Space Grotesk".to_string(),
                body_font: "Inter".to_string(),
                label_font: "Inter".to_string(),
                monospace_font: "JetBrains Mono".to_string(),
                font_scale: 1.0,
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
                xxl: 32.0,
            },
            effects: ThemeEffects {
                glow_intensity: 1.0,
                blur_radius: 20.0,
                animation_speed: 1.0,
                glassmorphism_enabled: true,
            },
            is_system: true,
        }
    }
}
```

### 10.4 Theme Management System
```rust
/// Theme manager that handles loading, saving, and applying themes
pub struct ThemeManager {
    /// Currently active theme
    current_theme: AppTheme,
    /// All available themes (system + custom)
    available_themes: Vec<AppTheme>,
    /// Path to custom themes directory
    custom_themes_path: PathBuf,
}

impl ThemeManager {
    /// Load theme manager with system themes and custom themes
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: SystemTheme::sonic_obsidian(),
            available_themes: SystemTheme::all(),
            custom_themes_path: Self::custom_themes_directory(),
        };
        
        // Load custom themes from disk
        manager.load_custom_themes();
        
        // Load user's saved theme preference from config
        manager.load_saved_theme();
        
        manager
    }
    
    /// Switch to a different theme
    pub fn apply_theme(&mut self, theme_id: &str) -> Result<(), ThemeError> {
        if let Some(theme) = self.available_themes.iter().find(|t| t.id == theme_id) {
            self.current_theme = theme.clone();
            self.save_theme_preference();
            Ok(())
        } else {
            Err(ThemeError::ThemeNotFound(theme_id.to_string()))
        }
    }
    
    /// Get current theme colors for UI rendering
    pub fn colors(&self) -> &ThemeColors {
        &self.current_theme.colors
    }
    
    /// Get current theme typography
    pub fn typography(&self) -> &ThemeTypography {
        &self.current_theme.typography
    }
    
    /// Save a custom theme
    pub fn save_custom_theme(&mut self, theme: AppTheme) -> Result<(), ThemeError> {
        // Validate theme
        Self::validate_theme(&theme)?;
        
        // Save to disk
        let theme_path = self.custom_themes_path.join(format!("{}.json", theme.id));
        let json = serde_json::to_string_pretty(&theme)?;
        std::fs::write(theme_path, json)?;
        
        // Add to available themes
        self.available_themes.push(theme);
        
        Ok(())
    }
}
```

### 10.5 Theme Application in UI Components
```rust
/// UI components should use theme values, not hardcoded colors
impl Button {
    pub fn render_themed(&self, ui: &mut Ui, theme: &ThemeManager) {
        ui.element()
            .width(layout::Sizing::Fixed(200.0))
            .height(layout::Sizing::Fixed(48.0))
            // Use theme colors
            .background_color(theme.colors().primary)
            .corner_radius(8.0)
            // Use theme typography
            .layout(|l| {
                l.padding(theme.spacing().lg)
                 .direction(TopToBottom)
                 .justify_center()
            })
            .children(|ui| {
                ui.text(&self.label, |t| 
                    t.font_size(16 * theme.typography().font_scale)
                     .color(theme.colors().on_surface)
                     .font_weight(FontWeight::Bold)
                );
            });
    }
}
```

### 10.6 Integration with Existing Config System
The theme system extends the existing `PianoThemeConfig` to include full application themes:
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

### 10.7 Theme Creation UI
The theme settings page should include:
1. **Theme Gallery**: Visual preview of all themes
2. **Theme Editor**: Color pickers, typography selectors
3. **Live Preview**: Real-time theme application
4. **Import/Export**: Share themes as JSON files
5. **Reset to Default**: Restore system themes

---

## 11. Design Tokens for Implementation

### Color Constants (Rust)
**Existing constants from ply_scene.rs (use these for rendering):**
```rust
pub const COLOR_BACKGROUND: Color = Color::new(0.055, 0.055, 0.075, 1.0); // #0e0e13 (14, 14, 19)
pub const COLOR_SURFACE_CONTAINER: Color = Color::new(0.098, 0.098, 0.122, 1.0); // #19191f (25, 25, 31)
pub const COLOR_SURFACE_CONTAINER_HIGHEST: Color = Color::new(0.145, 0.145, 0.173, 1.0); // #25252c (37, 37, 44)
pub const COLOR_PRIMARY: Color = Color::new(0.859, 0.565, 1.0, 1.0); // #db90ff (219, 144, 255)
pub const COLOR_PRIMARY_CONTAINER: Color = Color::new(0.827, 0.482, 1.0, 1.0); // #d37bff (211, 123, 255)
pub const COLOR_SECONDARY: Color = Color::new(0.373, 0.620, 1.0, 1.0); // #5f9eff (95, 158, 255)
pub const COLOR_TERTIARY: Color = Color::new(1.0, 0.431, 0.502, 1.0); // #ff6e80 (255, 110, 128)
pub const COLOR_ON_SURFACE: Color = Color::new(0.973, 0.961, 0.992, 1.0); // #f8f5fd (248, 245, 253)
```

**Theme system colors (u8 values for serialization):**
```rust
// Use these u8 values in ThemeColors struct for consistency
pub const THEME_BACKGROUND: (u8, u8, u8) = (14, 14, 19);
pub const THEME_SURFACE_CONTAINER: (u8, u8, u8) = (25, 25, 31);
pub const THEME_SURFACE_CONTAINER_HIGHEST: (u8, u8, u8) = (37, 37, 44);
pub const THEME_PRIMARY: (u8, u8, u8) = (219, 144, 255);
pub const THEME_PRIMARY_CONTAINER: (u8, u8, u8) = (211, 123, 255);
pub const THEME_SECONDARY: (u8, u8, u8) = (95, 158, 255);
pub const THEME_TERTIARY: (u8, u8, u8) = (255, 110, 128);
pub const THEME_ON_SURFACE: (u8, u8, u8) = (248, 245, 253);
```

### Typography Constants
```rust
pub const FONT_DISPLAY: &str = "Space Grotesk";
pub const FONT_BODY: &str = "Inter";
pub const FONT_LABEL: &str = "Inter";
pub const FONT_MONOSPACE: &str = "JetBrains Mono"; // For technical readouts
```

### Spacing Constants
```rust
pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;
pub const SPACING_2XL: f32 = 32.0;
```

---

## 11. Version History

- **v1.0** (2026-03-22): Initial design system based on "Sonic Obsidian" concept
- **Includes**: Color system, typography, spacing, component specifications, accessibility guidelines
- **Next**: Implementation guidelines and code examples

---

## 12. Usage Examples

### Creating a Primary Button (PLY Engine)
```rust
ui.element()
    .width(layout::Sizing::Fixed(200.0))
    .height(layout::Sizing::Fixed(48.0))
    .background_color(COLOR_PRIMARY)
    .corner_radius(8.0)
    .layout(|l| l.padding(24).direction(TopToBottom).justify_center())
    .children(|ui| {
        ui.text("Save Changes", |t| 
            t.font_size(16)
             .color(Color::BLACK)
             .font_weight(FontWeight::Bold)
        );
    });
```

### Creating a Glass Panel
```rust
ui.element()
    .width(layout::Sizing::Fill)
    .height(layout::Sizing::Hug)
    .background_color(Color::new(0.145, 0.145, 0.173, 0.6))
    .corner_radius(12.0)
    .layout(|l| l.padding(24).gap(16))
    .children(|ui| {
        // Content here
    });
```

---

**Document Status**: Final v1.0  
**Last Updated**: 2026-03-22  
**Author**: Neothesia Design System Team  
**Review Cycle**: Quarterly