# Task 018: Enhance Multi-Unit Selection HUD with Unit Stats

## Status
**Completed** - 2026-02-01

Implementation Notes:
- Modified `update_selected_units_grid_system` in `src/hud.rs` to display expanded unit cards with stats when multiple units are selected
- Changed grid from 6 columns to 4 columns for more horizontal space per unit card
- Each unit card now shows: unit name (truncated if needed), health (numeric HP), damage, and range stats
- Added color-coded owner indicator and health bar for each unit
- Modified `update_unit_stats_panel_system` to show aggregate information when multiple units selected:
  - Unit count header
  - Unit type breakdown (sorted by count, showing up to 5 types)
  - Total health (current/max)
  - Average damage and average range
- Single unit selection behavior unchanged (compact icon in center, full stats in right panel)
- No selection shows placeholder text
- Added helper function `get_health_color` for consistent health bar coloring
- Updated query to include `AttackCapability` and `MovementSpeed` for stats display

## Description
Enhance the HUD to display unit statistics in the center section (UnitsGridSection) when multiple units are selected. Currently, when more than one unit is selected, the center section shows a grid of unit icons with health bars, and the right section (UnitStatsSection) only shows a count like "2 units selected". The user wants to see unit stats displayed alongside or integrated with the unit grid when multiple units are selected.

## Why Needed
Currently, detailed unit stats (health, type, damage, range, speed, etc.) are only visible when exactly 1 unit is selected. When multiple units are selected, players lose visibility into unit statistics, making it harder to make tactical decisions. This enhancement provides players with better information during multi-unit selection, which is a common scenario in RTS gameplay.

## Acceptance Criteria
- When multiple units are selected (2+), the center UnitsGridSection shows both unit icons AND unit stats
- Stats displayed for each unit should include at minimum: unit name, health (numeric), damage, range
- Stats should be readable and well-organized (not cluttered)
- The unit icon grid layout should accommodate the additional stats information
- Health bars should remain visible and update correctly
- The right section (UnitStatsSection) can show aggregate information or be repurposed
- Single unit selection behavior remains unchanged (full stats in right panel)
- When no units are selected, show appropriate placeholder text
- UI should handle up to 12 selected units without overflow issues

## Relevant Files/Components
- `src/hud.rs` - Primary file to modify
  - `update_selected_units_grid_system` (lines 253-390) - Grid rendering logic
  - `update_unit_stats_panel_system` (lines 392-610) - Stats panel logic
  - `UnitsGridSection` component and related UI
  - `UnitStatsSection` component (may need repurposing for multi-select)

## Technical Considerations

**Current Behavior**:
- 0 units selected: Center shows "Selected Units" placeholder, right shows "Unit Stats (Select a unit)"
- 1 unit selected: Center shows single unit icon with health bar, right shows full stats panel
- 2+ units selected: Center shows grid of unit icons (6 columns, 2 rows max 12 units), right shows only count

**Design Options**:

**Option A: Stats Below Each Icon** (Recommended)
- Modify the unit icon in the grid to include stats text below the icon and health bar
- Keep icon layout compact (maybe 3-4 columns instead of 6)
- Show: Unit name (abbreviated if needed), HP (numeric), Damage, Range
- Right panel can show aggregate stats (total HP, average damage, unit type breakdown)

**Option B: Stats in Right Panel as List**
- Keep unit icons in center grid unchanged
- Modify right panel to show a scrollable list of all selected units with their stats
- Each entry shows: icon color, unit name, health, key stats
- More detailed but requires scrolling for many units

**Option C: Expandable Grid Items**
- Show compact icons by default
- Allow clicking/hovering on an icon to expand it with stats
- Most complex to implement

**Recommended Implementation (Option A)**:
1. Modify `update_selected_units_grid_system` for multi-select case
2. Change grid from 6 columns to 3-4 columns for more horizontal space
3. Expand each unit icon cell to include stats text:
   ```
   [Color Square]
   Unit Name
   HP: 100/150
   Dmg: 25 Rng: 7
   [Health Bar]
   ```
4. Adjust cell size from 50x60 to approximately 90x110
5. Modify right panel to show aggregate information:
   ```
   5 units selected

   Types:
   - 2x Heavy Tank
   - 3x Infantry

   Total Health: 450/550
   Avg Damage: 20
   Avg Range: 6.2
   ```

**Layout Calculations**:
- Center section has `flex_grow: 1.0`, so it takes remaining space after 200px (minimap) and 300px (stats)
- At 1280px window width: center has ~740px available
- With 3 columns at 100px each + gaps = ~340px used (plenty of room)
- With 4 columns at 90px each + gaps = ~400px used (still fits)

**Data Access**:
The system already queries all needed data:
```rust
selected_units: Query<(Entity, &UnitType, &UnitHealth, &Owner), (With<Unit>, With<Selected>)>
```

Need to add to the query:
```rust
selected_units: Query<(Entity, &UnitType, &UnitHealth, &Owner, &AttackCapability, &MovementSpeed), (With<Unit>, With<Selected>)>
```

**Text Formatting**:
- Use smaller font sizes (10-11px) for stats
- Use consistent color coding (damage in red-ish, range in blue-ish, speed in green-ish)
- Abbreviate long unit names if needed
- Use fixed-width sections for alignment

## Prerequisites
None - This is an enhancement to existing HUD functionality

## Complexity
Medium

## Notes
- The current grid system already handles up to 12 units effectively
- The UnitIcon component tracks which unit entity it represents
- Health bars already update dynamically
- Consider future extensibility for formations or group commands
- May want to add visual indicators for unit state (idle, moving, attacking, etc.) in a future task
- Could add filtering/sorting options in a future enhancement (e.g., group by unit type)
