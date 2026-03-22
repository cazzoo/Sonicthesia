# Design System Strategy: The Neon Pulse

## 1. Overview & Creative North Star
**Creative North Star: "The Sonic Obsidian"**

This design system is engineered to move beyond the "gamer aesthetic" into the realm of high-end, editorial performance software. We are not just building a tool; we are building an instrument. The "Sonic Obsidian" concept treats the screen as a dark, infinite stage where information does not sit *on* the surface but glows *within* it.

To break the "template" look, we employ **Intentional Asymmetry**. Key functional areas should use the Spacing Scale to create breathing room (e.g., a massive `24` unit gutter on one side of a display element) to evoke the feeling of a premium physical mixing console. Overlapping elements—where a high-index Glassmorphism panel partially obscures a background visualization—create a sense of three-dimensional depth and technical sophistication.

---

## 2. Colors: The Radiance of the Void
Our palette is anchored in `#0e0e13` (Background), providing a vacuum that allows our neon accents to hum with energy.

### The "No-Line" Rule
Explicitly prohibit 1px solid borders for sectioning. Structural definition must be achieved through background shifts. Place a `surface_container_low` (`#131318`) panel against the `background` (`#0e0e13`) to create a boundary. If the user cannot see the change, increase the container tier, do not add a line.

### Surface Hierarchy & Nesting
Think of the UI as layers of smoke and glass. 
- **Base Layer:** `surface` (#0e0e13)
- **Primary Workspaces:** `surface_container` (#19191f)
- **Interactive Modules:** `surface_container_highest` (#25252c)

By nesting a `surface_container_highest` card inside a `surface_container_low` section, you create a natural "lift" that feels integrated into the environment.

### The "Glass & Gradient" Rule
Standard flat buttons are forbidden for primary actions. Use a **Signature Texture**: a linear gradient from `primary` (#db90ff) to `primary_container` (#d37bff) at a 135-degree angle. For floating overlays (like performance HUDs), use `surface_variant` at 60% opacity with a `20px` backdrop blur to achieve a premium glassmorphism effect.

---

## 3. Typography: The Editorial Tech
We pair the geometric precision of **Space Grotesk** for high-impact displays with the Swiss-style clarity of **Inter** for functional data.

- **Display & Headlines (Space Grotesk):** Use `display-lg` and `headline-md` to create a bold, "poster" feel for track titles or performance modes. The wide apertures of Space Grotesk feel "high-tech" without being clichéd.
- **Functional Data (Inter):** All `body` and `label` styles utilize Inter. For technical readouts (BPM, frequency, timestamps), utilize the `label-md` or `label-sm` with a `monospace` font-feature setting to ensure numerical alignment.
- **Hierarchy:** Use extreme contrast. A `display-lg` title should be paired with a `label-sm` subtitle in `on_surface_variant` (#acaab1) to create an authoritative, editorial hierarchy.

---

## 4. Elevation & Depth: Tonal Layering
We do not use shadows to represent light sources; we use "Glows" to represent energy.

- **The Layering Principle:** Depth is achieved by stacking `surface-container` tiers. The closer an object is to the user, the "brighter" (more elevated) its container becomes. 
- **Ambient Shadows:** For floating menus, use a shadow with a blur of `32px`, `0px` offset, and 6% opacity using the `primary` (#db90ff) color. This creates a "neon halo" rather than a muddy grey drop shadow.
- **The "Ghost Border" Fallback:** If a tactile edge is required for accessibility, use a `1px` stroke of `outline_variant` (#48474d) at 20% opacity. It should be felt, not seen.
- **Active State Glow:** Active states (e.g., a selected track) should use a `primary` (#db90ff) 1px border with a `primary_dim` outer glow (4px blur).

---

## 5. Components

### Buttons
- **Primary:** Gradient fill (`primary` to `primary_container`), `DEFAULT` (0.5rem) roundedness. 
- **Secondary:** Transparent background with a "Ghost Border." On hover, fill with `primary` at 10% opacity.
- **Tertiary:** Text-only using `primary_dim`.

### Input Fields
- **Base:** `surface_container_highest` fill, no border. 
- **Active:** 1px `secondary` (#5f9eff) "Ghost Border" and a subtle `secondary` glow.
- **Typography:** Use `title-sm` for user input to ensure legibility in dark environments.

### Performance Chips
- **Status:** Use `tertiary` (#ff6e80) for "Live" states. 
- **Interaction:** Selection chips should use `secondary_container` with `on_secondary_container` text.

### The Performance Card (Custom)
- Forbid dividers. Separate the "Track Name" from "Metadata" using a `1.5` (0.375rem) vertical spacing gap.
- Use a `surface_container_highest` background.
- Apply a `2px` left-accent border using the `tertiary` color to denote "Active/Playing" status.

### Tooltips
- Use `inverse_surface` (#fbf8ff) with `inverse_on_surface` text. This provides a sharp, high-contrast break from the dark theme, ensuring critical info is never missed.

---

## 6. Do’s and Don’ts

### Do:
- **Do** use `24` (6rem) or `20` (5rem) spacing for outer margins to create an "expansive" premium feel.
- **Do** use `9999px` (full) roundedness for small tags and chips, but stick to `DEFAULT` (0.5rem) for main containers.
- **Do** use `primary_fixed_dim` for icons to ensure they "pop" against the obsidian background.

### Don’t:
- **Don’t** use pure white (#FFFFFF). Always use `on_surface` (#f8f5fd) to prevent "retina burn" in dark environments.
- **Don’t** use a divider line to separate list items. Use a `1px` vertical gap (Spacing `px`) that reveals the darker `background` color underneath.
- **Don’t** use standard "Error Red." Use the system `error` (#ff6e84) which is tuned to vibrate correctly against deep purples.