# Song Library Redesign - Implementation Plan

## Overview

Complete redesign of the Song Library and Song Selection screens based on the new "Sonic Obsidian Piano" design system. The implementation follows Neothesia's existing patterns while delivering a modern, production-grade UI/UX.

## Design System Analysis

### Color Palette (from design mockups)
- **Background**: `#0e0e13` (surface-dim)
- **Surface Container**: `#19191f` 
- **Surface Container High**: `#1f1f26`
- **Surface Container Highest**: `#25252c`
- **Primary**: `#db90ff` (purple accent)
- **Primary Container**: `#d37bff`
- **Secondary**: `#5f9eff` (blue accent)
- **Tertiary**: `#ff6e80` (pink/red accent)
- **On Surface**: `#f8f5fd` (main text)
- **On Surface Variant**: `#acaab1` (secondary text)

### Typography
- **Headlines**: Space Grotesk (font-headline)
- **Body/Labels**: Inter (font-body, font-label)

### Key Design Patterns
- Glassmorphism panels with blur effects
- Neon glow effects on primary elements
- Rounded corners (lg: 0.5rem, xl: 0.75rem)
- Status chips with color-coded backgrounds
- Progress bars with gradient fills

---

## Architecture Decisions

### 1. Screen Structure
Both screens will follow the established layout pattern:
- **Fixed Header** (top, z-50): Logo, Navigation, Search, Avatar
- **Fixed Sidebar** (left, below header): Library Explorer with categories
- **Scrollable Content** (center/right): Page-specific content only

### 2. Component Reuse
Leverage existing components from `neothesia/src/ui/components/`:
- `GlassPanel` - for card backgrounds and sections
- `StatusChip` - for difficulty levels, song status badges
- `ThemeCard` pattern - for song card interactions

### 3. New Components Required

| Component | Purpose | Reusability |
|-----------|---------|-------------|
| `SongCard` | Song grid item with progress, rating | High - song library |
| `StarRating` | 5-star difficulty/quality display | High - multiple screens |
| `ProgressBar` | Visual progress indicator | High - scores, loading |
| `ModeSelector` | Bento grid mode cards (Listen/Learn/Play) | Medium - song selection |
| `SessionConfig` | Difficulty, speed, visual hints panel | Medium - song selection |
| `Header` | Shared top navigation bar | High - all screens |
| `Sidebar` | Library Explorer navigation | High - library screens |

---

## Implementation Tasks

### Phase 1: Foundation Components

#### Task 1.1: Create Header Component
**File**: `neothesia/src/ui/components/header.rs`

```rust
pub struct Header {
    pub active_nav: NavItem, // Library, Practice, Settings
    pub search_query: String,
    pub is_search_focused: bool,
}
```

**Features**:
- Logo text "Sonic Obsidian Piano"
- Navigation links with active state (border-bottom indicator)
- Search input with icon
- Account/avatar button
- Fixed position, z-index management

#### Task 1.2: Create Sidebar Component
**File**: `neothesia/src/ui/components/sidebar.rs`

```rust
pub struct Sidebar {
    pub active_section: SidebarSection, // MIDI Library, Song Lists, Recordings
    pub smart_playlists: Vec<SmartPlaylist>, // Recent, Favorites, Difficult
}
```

**Features**:
- "Library Explorer" header with subtitle
- Main sections with icons and active indicator (left border)
- Smart Playlists section with smaller buttons
- Fixed below header, scrollable if needed

#### Task 1.3: Create SongCard Component
**File**: `neothesia/src/ui/components/song_card.rs`

```rust
pub struct SongCard {
    pub song: SongEntry,
    pub is_selected: bool,
    pub is_hovered: bool,
}
```

**Features**:
- Glass panel background with hover effect
- Icon container (music_note, piano, etc.)
- Star rating display (1-5 stars)
- Song title (headline font)
- Metadata line (genre • duration • format)
- Status chip (Learning, Active, New, etc.)
- Progress bar with percentage

#### Task 1.4: Create StarRating Component
**File**: `neothesia/src/ui/components/star_rating.rs`

```rust
pub struct StarRating {
    pub rating: u8, // 0-5
    pub max_stars: u8,
    pub size: f32,
}
```

**Features**:
- Render filled/empty stars using material icons
- Configurable star count and size
- Primary color for filled stars

#### Task 1.5: Create ProgressBar Component
**File**: `neothesia/src/ui/components/progress_bar.rs`

```rust
pub struct ProgressBar {
    pub progress: f32, // 0.0-1.0
    pub color: Color,
    pub height: f32,
}
```

**Features**:
- Background track (surface-container-highest)
- Filled portion with color
- Optional percentage text

### Phase 2: Song Library Screen

#### Task 2.1: Create SongLibraryPage
**File**: `neothesia/src/settings/pages/song_library.rs`

**Structure**:
```rust
pub struct SongLibraryPage {
    scroll_offset: f32,
    songs: Vec<SongEntry>,
    hovered_song: Option<usize>,
    selected_song: Option<usize>,
    // Bento stats section
    current_progress: Option<ProgressStats>,
    daily_drill: Option<DrillConfig>,
}
```

**Layout** (matching design):
1. **Header Section**: "Piano Repertoire" title + description + "Practice Now" button
2. **Bento Grid**: 
   - Current Progress card (glass panel, primary accent)
   - Quick Start / Daily Drill card (tertiary accent)
3. **Song Grid**: 3-column responsive grid of SongCard components
4. **Footer**: Connected device info, soundfont info

**Interactions**:
- Mouse hover highlights cards
- Click navigates to Song Selected screen
- Scroll affects only content area (header/sidebar fixed)
- Search filters song list

#### Task 2.2: Update Settings Navigation
**File**: `neothesia/src/settings/navigation.rs`

Add new tab:
```rust
pub enum SettingsTab {
    // ... existing tabs
    SongLibrary,  // NEW
}
```

Update `label()` and `icon()` methods.

### Phase 3: Song Selection Screen

#### Task 3.1: Create SongSelectedPage
**File**: `neothesia/src/settings/pages/song_selected.rs`

**Structure**:
```rust
pub struct SongSelectedPage {
    pub song: SongEntry,
    pub selected_mode: PlayMode, // Listen, Learn, Play
    pub session_config: SessionConfig,
}
```

**Layout** (matching design):
1. **Hero Section**: Full-width aspect ratio image with gradient overlay
   - Back button with arrow
   - Song badges (Masterpiece, Classical • Solo Piano)
   - Title and artist
   - Stats sidebar (Total Notes, Hand Breakdown, Duration)
2. **Mode Selection**: 3-column bento grid
   - **Listen**: 4K Visuals, Spatial Audio badges
   - **Learn**: Difficulty meter, Speed indicator, Fingering options
   - **Play**: Score HUD, Game Mode badges
3. **Session Configuration**: Glass panel with controls
   - Difficulty buttons (EASY/MED/HARD)
   - Playback speed slider
   - Visual assistance toggle (Fingering Numbers)
   - MIDI input device status
4. **Final CTA**: Large "START SESSION" button with gradient
   - "Add to Favorites" link

#### Task 3.2: Create ModeSelector Component
**File**: `neothesia/src/ui/components/mode_selector.rs`

**Features**:
- 3-column grid layout
- Each mode card with:
  - Icon container with mode-specific color
  - Title (headline font)
  - Description
  - Feature badges
  - Hover effects and active glow

#### Task 3.3: Create SessionConfig Component
**File**: `neothesia/src/ui/components/session_config.rs`

**Features**:
- Glass panel container
- Difficulty button group
- Speed slider with tick marks
- Toggle switch for fingering
- Device status indicator

### Phase 4: Integration

#### Task 4.1: Update PlyScene Integration
**File**: `neothesia/src/scene/ply_scene.rs`

- Update `PlySongLibraryScene` to use new `SongLibraryPage`
- Add `PlySongSelectedScene` for song selection
- Wire navigation between scenes

#### Task 4.2: Update Main Macroquad Entry
**File**: `neothesia/src/main_macroquad.rs`

- Register new scenes
- Handle scene transitions

#### Task 4.3: Export New Components
**File**: `neothesia/src/ui/components/mod.rs`

Add exports:
```rust
pub mod header;
pub mod sidebar;
pub mod song_card;
pub mod star_rating;
pub mod progress_bar;
pub mod mode_selector;
pub mod session_config;

pub use header::Header;
pub use sidebar::Sidebar;
pub use song_card::SongCard;
pub use star_rating::StarRating;
pub use progress_bar::ProgressBar;
pub use mode_selector::ModeSelector;
pub use session_config::SessionConfig;
```

---

## Technical Requirements

### Page Size Constraint
Each page file must be **<1000 lines** as per application standards. If complexity requires more:
- Extract sub-components
- Use composition over monolithic render methods
- Split into multiple smaller files

### Scrolling Behavior
- Header: Fixed at top (z-50)
- Sidebar: Fixed left, below header (height: calc(100vh - 64px))
- Content: Scrollable area only (pt-24 for header, ml-64 for sidebar)

### Design Fidelity
Every visual element must match the design mockups:
- Color values exact from CSS variables
- Spacing from design system (spacing::XL, spacing::LG, etc.)
- Border radius consistent (radius::LG, radius::XL)
- Typography hierarchy (headline vs body fonts)

### Error Handling
- Empty states (no songs found)
- Loading states (scanning library)
- Error states (failed to load song)
- Graceful fallbacks

---

## File Structure Summary

```
neothesia/src/
├── ui/components/
│   ├── mod.rs (updated)
│   ├── header.rs (NEW)
│   ├── sidebar.rs (NEW)
│   ├── song_card.rs (NEW)
│   ├── star_rating.rs (NEW)
│   ├── progress_bar.rs (NEW)
│   ├── mode_selector.rs (NEW)
│   └── session_config.rs (NEW)
├── settings/pages/
│   ├── mod.rs (updated)
│   ├── song_library.rs (NEW)
│   └── song_selected.rs (NEW)
├── settings/
│   └── navigation.rs (updated)
└── scene/
    └── ply_scene.rs (updated)
```

---

## Verification Checklist

- [ ] All colors match design mockups
- [ ] Typography uses correct fonts (Space Grotesk, Inter)
- [ ] Glass panels have correct blur/opacity
- [ ] Hover states match design interactions
- [ ] Star ratings render correctly (filled/empty)
- [ ] Progress bars show correct percentages
- [ ] Scrolling only affects content area
- [ ] Navigation between screens works
- [ ] Search filters song list
- [ ] Mode selection highlights active card
- [ ] Session config controls function
- [ ] Each file < 1000 lines
- [ ] No type suppression (as any, @ts-ignore)
- [ ] LSP diagnostics clean
