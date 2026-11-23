# hud-bar-content-redistribution

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-hud_bar_layout.md

## Task

Migrate content rendering from the monolithic `update_selected_units_grid_system` into separate per-section update systems for Portrait, InfoPanel, and SelectionPanel. This completes the HUD Bar layout by populating each section with appropriate content.

### What to do:

1. **Portrait section** (`update_portrait_section_system`):
   - When 1+ objects selected, display the active selection group's representative portrait (object type name, faction-colored icon/box, type info)
   - Single-select: show the one selected object's portrait
   - Multi-select: show the ActiveGroup's type portrait
   - When nothing selected: section is hidden (handled by visibility system from scaffold task)

2. **InfoPanel section** (`update_info_panel_section_system`):
   - When 1 object selected: show detailed stats (HP, armor, damage, speed, special properties — the content currently rendered in `update_selected_units_grid_system` for single-unit/single-structure/single-resource cases)
   - When 2+ selected: show summary info for the active group (count, type name)
   - Migrate the single-unit detail display, single-structure detail display, and single-resource detail display from the existing system into this new system

3. **SelectionPanel section** (`update_selection_panel_section_system`):
   - When 2+ objects selected: show the grid of unit portrait icons (the multi-select grid currently rendered by `update_selected_units_grid_system`)
   - Portraits belonging to the ActiveGroup should have a highlight
   - Migrate the multi-select icon grid rendering from the existing system

4. **Remove** the old `update_selected_units_grid_system` and the `UnitsGridSection` marker component. Remove the temporary `UnitsGridSection` attachment from the scaffold task. Clean up any orphaned icon marker components (`UnitIcon`, `StructureIcon`, `ResourceIcon`) if they are only used by the old system — or repurpose them for the new systems.

5. **Update system registration** in the UI plugin — replace the old system with the three new ones.

6. **Health bar updates**: The existing `unit_health_bars` and `struct_health_bars` queries in the old system need to be moved to whichever new system renders them (likely InfoPanel for single-select, SelectionPanel for multi-select icons).

### Key files:
- `src/ui/hud.rs` — refactor/replace `update_selected_units_grid_system`
- `src/ui/types.rs` — clean up old markers, potentially add new ones
- `src/ui/mod.rs` — update system registration

### Dependencies:
- Depends on `hud-bar-five-section-scaffold` (the 5-section layout must exist first)
