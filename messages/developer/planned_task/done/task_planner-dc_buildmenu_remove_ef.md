# dc-buildmenu-remove-ef

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-dc-ef-construction-rework.md

## Task

Remove ExtractionFacility from the DeploymentCenter's BuildMenu. Per the design doc (gdo_objects.md), DC only constructs PowerPlant, Barracks, and SupplyTower. ExtractionFacility is a separate structure with its own construction interface — it should not appear in DC's build menu.

## Technical Context

### Files to modify

**`artifacts/developer/src/ui/command_panel.rs`** (all changes in this file):

1. **Grid slot (line 60)**: Remove `(0, 2) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))` from the `StructureMenuState::DcBuildMenu` match arm (lines 57-64). The remaining entries are PowerPlant at (0,0), Barracks at (0,1), SupplyTower at (1,0), and Back at (2,0).

2. **Label (line 2273)**: Remove `CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n250 SC", hotkey)` from `grid_button_label()`. The wildcard `DcBuild(_)` on line 2275 will catch any unexpected variants.

3. **Availability (line 2340)**: Remove `CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => player_sc >= 250` from `grid_button_enabled_ext()`. The function has no wildcard for DcBuild — either add a `DcBuild(_) => false` fallthrough or just remove the line (the existing match has a final `_ => true` catch-all at the bottom).

4. **Tests**:
   - `dc_build_menu_grid_has_all_buildings_and_back` (line 4294): Remove the assertion at line 4299 that checks slot (0,2) for ExtractionFacility. Add a new assertion that slot (0,2) returns `None`.
   - `dc_build_menu_shows_structures` (line 3047): This test only checks PP and BK — no changes needed.
   - `supply_tower_build_button_in_dc_build_menu` (line 3675): No changes needed — SupplyTower stays at (1,0).

**`artifacts/developer/src/game/types/structures.rs`** (line 96):

5. **DeploymentCenterState::construction_cost()**: Remove the `ObjectEnum::ExtractionFacility` arm (lines 96-99) from the match. The DC should not know how to cost EF construction. Note: `ExtractionFacilityState::construction_cost()` (line ~201) is a SEPARATE method on the EF state — do NOT touch it; it's used by the EF's own construction interface.

6. **Test updates in structures.rs**: There are no existing tests that assert DC can build EF, but optionally add a test: `assert!(DeploymentCenterState::construction_cost(&ObjectEnum::ExtractionFacility).is_none())` near the existing tests at line 1183.

### Runtime safety

The `execute_command_action` handler for `CommandButtonAction::DcBuild(object_type)` (command_panel.rs line 1143) uses a generic match arm that calls `DeploymentCenterState::construction_cost(object_type)`. After removing EF from that cost function, even if somehow a DcBuild(EF) action were dispatched, the cost lookup would return `None` and the build would silently fail. This is safe.

### Patterns to follow

- Grid slot removal: simply delete the match arm line. See the existing `_ => None` catch-all.
- Label/enabled removal: delete the specific match arm. The function will fall through to existing wildcard or default arms.
- Test pattern: existing tests use `assert!(matches!(get_grid_slot_action(&state, row, col, false, false, &caps, false), Some(...)))` — for negative test use `.is_none()`.

## Dependencies

None — this is a standalone removal task. The EF's own construction interface (`EfIdle`, `EfConstructing`, `EfReadyToPlace` states, `ExtractionFacilityState::construction_cost()`, `EfBuildPlate` action) is a separate system that remains untouched.
