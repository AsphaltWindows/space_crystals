# sds_infopanel_visibility_rework

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-fog_of_war_selectability.md

## Task

Rework the SupplyDeliveryStation InfoPanel to be visibility-context-aware, and ensure NextDeliveryTime is always displayed (not gated behind CurrentSupplies==0).

### Current State

In `ui/hud.rs` (~line 810-831), the SDS InfoPanel currently shows:
- DeliverySize and DeliveryInterval: always shown
- CurrentSupplies: shown only when tile is Visible
- NextDeliveryTime (`time_until_next_delivery`): shown only when Visible AND CurrentSupplies==0

The design requires NextDeliveryTime to be shown in BOTH Visible and Explored contexts, and CurrentSupplies only in Visible context.

### Required Changes

1. **InfoPanel display logic** (`update_selected_units_grid_system` in `hud.rs`, SDS single-selection branch ~line 810-831):
   - Add `SelectedFaction` resource to the system parameters
   - Get the fog state for the SDS tile position
   - **Visible** (any player): show DeliverySize, DeliveryInterval, NextDeliveryTime, CurrentSupplies (all four fields)
   - **Explored** (GDO player only -- this is the only case where Explored SDS is selectable): show DeliverySize, DeliveryInterval, NextDeliveryTime (three fields, no CurrentSupplies)
   - NextDeliveryTime should always be displayed as a countdown regardless of CurrentSupplies value (remove the `if sds.current_supplies == 0` gate)
   - Format: `Next delivery: {:.1}s` as currently done

2. **Note on Explored+non-GDO and Unexplored**: These cases will never reach the InfoPanel because the selection system (modified in the companion task) prevents selection. No defensive coding needed for these cases, but the code should be clean and not show stale data.

3. **Tests**: Add/update tests verifying:
   - SDS InfoPanel in Visible state shows all 4 fields
   - SDS InfoPanel in Explored state (GDO) shows 3 fields (no CurrentSupplies)
   - NextDeliveryTime is shown regardless of CurrentSupplies value

## Technical Context

### Primary file to modify

**`artifacts/developer/src/ui/hud.rs`** -- the `update_selected_units_grid_system` function (starts at line 223).

### SDS InfoPanel code location (lines 756-834)

The SDS single-selection branch starts at line 756 (`else if let Some((res_entity, sds, obj_instance, sds_grid_pos)) = selected_sds.iter().next()`). The stats section that needs reworking is lines 808-832 (the `.with_children(|stats_parent|)` closure for the right-side stats panel).

Current fog-of-war logic (lines 820-831):
- Line 821: `let sds_visible = fog_map.get(local_player.0, sds_grid_pos.x, sds_grid_pos.z) == VisibilityStateEnum::Visible;`
- Lines 822-828: If visible, shows CurrentSupplies AND conditionally shows NextDeliveryTime (only when current_supplies == 0)
- Lines 829-831: Else shows `Current: ???`

### Changes needed in the stats closure (lines 820-831)

Replace the current binary visible/not-visible logic with a three-state check:

1. **Add `SelectedFaction` to system params** (line 223-238). Import from `crate::types::SelectedFaction` -- already available via `use crate::types::*;` on line 2. Add param: `selected_faction: Res<SelectedFaction>`.

2. **Get the fog state as an enum value** instead of a bool:
   ```rust
   let fog_state = fog_map.get(local_player.0, sds_grid_pos.x, sds_grid_pos.z);
   ```

3. **Rework the conditional display**:
   - If `fog_state == VisibilityStateEnum::Visible`: show all 4 stats (DeliverySize, DeliveryInterval, NextDeliveryTime unconditionally, CurrentSupplies)
   - If `fog_state == VisibilityStateEnum::Explored`: show 3 stats (DeliverySize, DeliveryInterval, NextDeliveryTime -- no CurrentSupplies). The GDO check is not needed in this branch since the selection system already prevents non-GDO from selecting SDS in Explored.
   - NextDeliveryTime display: remove the `if sds.current_supplies == 0` gate so it always shows. Use the same format: `"  Next delivery: {:.1}s"` with `sds.time_until_next_delivery`.

4. **Pattern to follow**: The SpaceCrystalsPatch section (lines 735-753) uses the same fog_map.get pattern. The SDS section should mirror this approach but with Explored as a distinct case instead of falling through to `???`.

### Key types and imports

- **`SelectedFaction`** -- `pub struct SelectedFaction(pub FactionEnum)` in `artifacts/developer/src/shared/types.rs` line 24. Already glob-imported via `use crate::types::*;` on hud.rs line 2.
- **`FactionEnum::GlobalDefenseOrdinance`** -- the GDO variant, in `artifacts/developer/src/shared/types.rs` line 275+.
- **`VisibilityStateEnum`** -- enum with Unexplored, Explored, Visible variants. Already used in hud.rs (line 492, 736, 821).
- **`FogOfWarMap`** -- already a system param (`fog_map: Res<FogOfWarMap>` on line 237).
- **`LocalPlayer`** -- already a system param (`local_player: Res<LocalPlayer>` on line 238).
- **`SupplyDeliveryStation`** struct fields: `delivery_size: u32`, `delivery_interval: f32`, `current_supplies: u32`, `time_until_next_delivery: f32` (defined in `artifacts/developer/src/game/world/types.rs` line 172-177).

### Existing test patterns (hud.rs lines 1989-2036)

The existing fog-of-war tests in hud.rs (starting line 1989) are **unit tests on the FogOfWarMap data structure** -- they test `fog.get()`/`fog.set()` behavior, not the actual system rendering. The new tests should follow this same style:

- Test that when fog state is Visible, the display logic would include all 4 fields
- Test that when fog state is Explored, the display logic would include 3 fields (no CurrentSupplies)
- Test that NextDeliveryTime is shown regardless of current_supplies value

Since the existing tests are data-level (not system-level), new tests can follow the same pattern: create a `FogOfWarMap`, set visibility states, and assert the conditions that drive display. For example:
- Assert `fog.get() == VisibilityStateEnum::Visible` means all fields shown
- Assert `fog.get() == VisibilityStateEnum::Explored` means NextDeliveryTime shown but CurrentSupplies not
- Test that the `sds.time_until_next_delivery` field is available regardless of `sds.current_supplies` value (construct SDS with current_supplies > 0 and verify time_until_next_delivery is still a valid field)

### System parameter addition

The system signature at line 223 currently has 14 params. Adding `selected_faction: Res<SelectedFaction>` brings it to 15. Bevy 0.17 supports up to 16 params in a system function, so this is within limits.

### TestApp setup consideration

If writing system-level tests using `TestApp`, note that `SelectedFaction` must be inserted as a resource. The `TestApp::new(faction)` helper (in `artifacts/developer/src/shared/testing/test_app.rs` line 104-108) already inserts `SelectedFaction`, so any tests using TestApp will have it available.

## Dependencies

- **Companion task (selection gating)**: This task depends on the companion selection-system task from the same parent feature (`fog_of_war_selectability`) which prevents non-GDO players from selecting SDS in Explored areas. Without that selection gating, the Explored branch in this task's InfoPanel code could theoretically be reached by non-GDO players. However, the two tasks can be implemented independently -- the InfoPanel code simply shows the appropriate fields based on fog state, and the selection system controls who can reach it.
- **Existing FogOfWarMap infrastructure**: Already implemented and working. No new fog-of-war systems needed.
- **SelectedFaction resource**: Already exists and is inserted during game setup. No new resource creation needed.
