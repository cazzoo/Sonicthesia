# PLY Rendering Migration Plan

## Overview

This document outlines the complete migration from WGPU rendering to PLY/macroquad rendering in Neothesia.

## Current State

- ✅ PLY integration for data management, input, UI, and audio (completed)
- ❌ Rendering still uses WGPU
- ❌ Visual indicators are misleading ("PLY ENGINE ACTIVE" but PLY doesn't render)

## Target State

- ✅ All rendering uses PLY/macroquad
- ✅ No WGPU dependencies
- ✅ Accurate "PLY RENDERING ACTIVE" indicators
- ✅ Full PLY integration complete

## Migration Steps

### Phase 1: Core Infrastructure (Current)

1. **Add macroquad dependency** ✅
   - Added to `neothesia/Cargo.toml`

2. **Create macroquad rendering modules** ✅
   - Created `neothesia/src/render/ply/macroquad_renderer.rs`
   - Implements `MacroquadWaterfallRenderer`, `MacroquadKeyboardRenderer`, `MacroquadGuidelineRenderer`

3. **Create macroquad main entry point** ✅
   - Created `neothesia/src/main_macroquad.rs`
   - Uses `#[macroquad::main]` instead of winit event loop

### Phase 2: Context System (Next)

4. **Create MacroquadContext**
   - Replace `Context` with `MacroquadContext`
   - Remove WGPU dependencies
   - Add macroquad state management

5. **Update scene trait**
   - Add `render_ply()` method to `Scene` trait
   - Keep existing `render()` for gradual migration

### Phase 3: Scene Migration

6. **Update PlayingScene**
   - Implement `render_ply()` using macroquad renderers
   - Remove WGPU render pass usage
   - Update visual indicators to "PLY RENDERING ACTIVE"

7. **Update MenuScene**
   - Implement `render_ply()` for menu rendering
   - Use PLY UI framework for all UI elements

8. **Update other scenes**
   - FreeplayScene
   - ScoreScene
   - All other scenes

### Phase 4: Cleanup

9. **Remove WGPU dependencies**
   - Remove `wgpu` and `wgpu-jumpstart` from dependencies
   - Remove WGPU-specific code
   - Remove old render methods

10. **Update documentation**
    - Update PLY_INTEGRATION_STATUS.md
    - Document the complete migration
    - Update README if needed

### Phase 5: Testing

11. **Test all rendering**
    - Waterfall rendering
    - Keyboard rendering
    - Guidelines rendering
    - UI elements
    - All scenes

12. **Performance testing**
    - Ensure performance is acceptable
    - Profile and optimize if needed

## Technical Challenges

### Challenge 1: Window System Incompatibility

**Problem:** winit and macroquad both want to control the window and event loop.

**Solution:** Complete replacement of winit with macroquad's window system.

### Challenge 2: Rendering Context

**Problem:** WGPU uses render passes, macroquad uses immediate mode drawing.

**Solution:** Rewrite all rendering to use macroquad's drawing functions.

### Challenge 3: Asset Loading

**Problem:** WGPU and macroquad have different asset loading systems.

**Solution:** Use macroquad's asset loading functions.

### Challenge 4: Text Rendering

**Problem:** Current text rendering uses cosmic-text with WGPU.

**Solution:** Use macroquad's built-in text rendering.

## Estimated Effort

- Phase 1: Core Infrastructure - ✅ COMPLETED
- Phase 2: Context System - 4-6 hours
- Phase 3: Scene Migration - 8-12 hours
- Phase 4: Cleanup - 2-4 hours
- Phase 5: Testing - 4-6 hours

**Total: 18-28 hours of development work**

## Risks

1. **Performance:** macroquad may have different performance characteristics than WGPU
2. **Compatibility:** Some features may not translate directly to macroquad
3. **Stability:** Major architectural changes can introduce bugs
4. **Time:** This is a significant undertaking that may take longer than estimated

## Mitigation Strategies

1. **Gradual Migration:** Keep both systems running during transition
2. **Thorough Testing:** Test each phase before proceeding
3. **Performance Monitoring:** Profile performance at each step
4. **Rollback Plan:** Keep WGPU version working until macroquad version is stable

## Next Steps

1. Implement `MacroquadContext`
2. Update `Scene` trait with `render_ply()` method
3. Migrate `PlayingScene` to use PLY rendering
4. Test thoroughly before proceeding to other scenes
5. Remove WGPU dependencies once all scenes are migrated

## Success Criteria

- ✅ All rendering uses PLY/macroquad
- ✅ No WGPU dependencies in final build
- ✅ "PLY RENDERING ACTIVE" indicators are accurate
- ✅ All visual elements render correctly
- ✅ Performance is acceptable
- ✅ No regressions in functionality
