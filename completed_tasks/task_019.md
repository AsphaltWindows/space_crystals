# Task 019: Refactor HUD Layout - Move Unit Stats to Center Section

## Status
**Completed** - 2026-02-01

Implementation moved all unit stats to the center section with a card-based layout:
- Single unit selection shows a detailed view with unit icon/color on the left and organized stats on the right
- Stats are grouped into identity (name, type, owner), combat (damage, range), movement (speed), and turret info
- Health bar is prominently displayed below the unit icon with numeric values
- Multiple units continue to display in a 4-column grid with compact cards
- Removed the right `UnitStatsSection` panel entirely
- Removed the `update_unit_stats_panel_system` function

## Description
Refactor the HUD layout so that the center section dynamically displays different content based on selection:
- When 1 unit is selected: Display detailed unit stats (name, health, damage, range, speed, turret info, owner, unit type)
- When multiple units are selected: Display grid of unit icons with health bars

The right section (UnitStatsSection) should be removed or repurposed, as it will no longer be used for displaying unit statistics.

## Why Needed
The current HUD design splits information between the center and right sections in a way that doesn't make optimal use of screen space. By consolidating unit information in the center section, we can:
- Make better use of the larger center area for detailed single-unit stats
- Create a more intuitive interface where all unit information is in one place
- Free up the right section for other potential features or remove it to maximize screen space

## Acceptance Criteria
- [x] When 1 unit is selected, the center section displays detailed unit stats including:
  - Unit name (large, prominent)
  - Health (current/max with visual health bar)
  - Unit base type (Light Infantry, Wheeled Vehicle, etc.)
  - Owner (with color indicator)
  - Movement speed
  - Attack damage and range
  - Turret information (if applicable: turn angle and turn rate)
- [x] When multiple units are selected, the center section displays a grid of unit icons with:
  - Unit color indicator
  - Unit name
  - Health values (current/max)
  - Basic stats (damage, range)
  - Visual health bars
- [x] When no units are selected, the center section shows an appropriate placeholder message
- [x] The right section (UnitStatsSection) is removed from the HUD layout
- [x] The system `update_unit_stats_panel_system` is removed or refactored as it's no longer needed
- [x] Layout is visually polished and stats are clearly readable
- [x] Health bars update in real-time as units take damage

## Relevant Files/Components
- `/home/iv/dev/space_crystals/src/hud.rs` (primary file to modify)
  - `setup_hud` function (lines 68-207): Remove right section creation
  - `update_selected_units_grid_system` function (lines 253-535): Refactor to show detailed stats for single unit
  - `update_unit_stats_panel_system` function (lines 548-891): Remove or refactor
  - `UnitStatsSection` component (lines 34-36): Can be removed
  - `UnitsGridSection` component (lines 30-32): Will handle both single and multiple unit displays

## Technical Considerations

**Layout Changes:**
- The center section currently uses `flex_grow: 1.0` which gives it flexible width between the minimap and right section
- After removing the right section, the center section will have even more space
- Need to ensure single-unit detailed view is well-organized and doesn't look sparse in the larger space
- Consider using a card-style layout for single unit stats with good spacing and visual hierarchy

**System Refactoring:**
- `update_selected_units_grid_system` currently handles 3 cases: no selection, 1 unit, multiple units
- For single unit case (lines 302-377), need to expand from simple icon to full detailed stats
- Can reuse much of the logic from `update_unit_stats_panel_system` (lines 567-734)
- The `update_unit_stats_panel_system` can be completely removed from the plugin's Update systems

**Component Cleanup:**
- `UnitStatsSection` marker component can be removed
- All queries using `With<UnitStatsSection>` should be removed
- The entity hierarchy for the right section should not be created in `setup_hud`

**Visual Design:**
- Single unit view should use larger fonts and better spacing given the increased space
- Consider organizing stats into logical groups (identity, combat, movement)
- Maintain consistent styling with the rest of the HUD (dark backgrounds, appropriate text colors)
- Health bars should remain prominent and update smoothly

**Edge Cases:**
- Ensure health bar updates work correctly in the new layout
- Handle units with and without turrets properly
- Verify that selection changes trigger proper UI updates
- Test with various unit types to ensure all stats display correctly

## Prerequisites
None

## Complexity
Medium
