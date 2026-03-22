# dc_buildmenu_add_ef

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-dc_builds_extraction_facility.md

## Task

Add Extraction Facility to the Deployment Center's BuildMenu grid and construction cost table.

### Changes needed:

1. **command_panel.rs** - `StructureMenuState::DcBuildMenu` grid (around line 59): Add slot `(1, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))` — this places EF next to Supply Tower in the second row.

2. **structures.rs** - `DeploymentCenterState::construction_cost()` (around line 86): Add match arm `ObjectEnum::ExtractionFacility => Some(StructureCost { space_crystals: 200, build_frames: 320 })`.

3. **Update/add tests**: The existing test at structures.rs line 1181 asserts `construction_cost(&ObjectEnum::ExtractionFacility).is_none()` — this must be updated to assert `Some` with the correct values (200 SC, 320 frames). Add a test for the DcBuildMenu grid slot returning the correct action.

### Context:
- The DC build menu already has PowerPlant at (0,0), Barracks at (0,1), SupplyTower at (1,0). EF has no prerequisite (unlike SupplyTower which requires a PowerPlant).
- The construction/placement flow for DC-built structures is fully generic — DcBuild action triggers crystal deduction and sets current_construction, then the ef_construction_tick_system handles progress, and the placement flow is shared. No new systems needed.
- ExtractionFacility is already a valid ObjectEnum variant with spawn function, EF state, and construction tick system.

## Technical Context

### Files to modify

1. **`artifacts/developer/src/ui/command_panel.rs`** (line 59-64)
   - The `DcBuildMenu` grid match block currently has 4 entries:
     - `(0, 0) => DcBuild(PowerPlant)`
     - `(0, 1) => DcBuild(Barracks)`
     - `(1, 0) => DcBuild(SupplyTower)`
     - `(2, 0) => Back`
   - Add: `(1, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))` between the SupplyTower and Back arms.
   - The function signature is at line 42: `fn get_grid_slot_action(state, row, col, has_active_construction, bk_has_queue, caps, has_ready_plate, has_active_ef_construction)` — no signature change needed.

2. **`artifacts/developer/src/game/types/structures.rs`** (line 86-101)
   - `DeploymentCenterState::construction_cost()` match block. Add before the `_ => None` wildcard:
     ```rust
     ObjectEnum::ExtractionFacility => Some(StructureCost {
         space_crystals: 200,
         build_frames: 320,
     }),
     ```
   - Follow the exact pattern of existing arms (PowerPlant, Barracks, SupplyTower).

3. **`artifacts/developer/src/game/types/structures.rs`** (line 1178-1182)
   - Test `dc_construction_cost_invalid_returns_none` currently asserts EF returns `None`. **Remove** the `ExtractionFacility` line from this test.
   - **Add** a new test (follow the pattern of `dc_construction_cost_barracks` at line 1171):
     ```rust
     #[test]
     fn dc_construction_cost_extraction_facility() {
         let cost = DeploymentCenterState::construction_cost(&ObjectEnum::ExtractionFacility).unwrap();
         assert_eq!(cost.space_crystals, 200);
         assert_eq!(cost.build_frames, 320);
     }
     ```

4. **`artifacts/developer/src/ui/command_panel.rs`** (test section, near line 3777)
   - Add a new grid slot test following the pattern of `dc_build_menu_back_at_bottom_left` (line 3776):
     ```rust
     #[test]
     fn dc_build_menu_ef_at_row1_col1() {
         let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
         let caps = no_caps();
         let action = get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false);
         assert!(matches!(action, Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))));
     }
     ```

### Patterns to follow
- **Cost arm pattern**: Each DC-buildable structure is a direct match arm in `construction_cost()` returning `Some(StructureCost { ... })`. The wildcard `_ => None` catches everything else.
- **Grid slot pattern**: Static grid entries in the `DcBuildMenu` match use `(row, col) => Some(CommandButtonAction::DcBuild(ObjectEnum::Variant))`.
- **Test naming**: `dc_construction_cost_{structure_name}` for cost tests; `dc_build_menu_{description}` for grid tests.
- **Test helper**: `no_caps()` (line 2786) returns a default `SelectedUnitCapabilities` with all fields false — use for grid slot tests.

### No new imports needed
- `ObjectEnum::ExtractionFacility` is already in scope in both files.
- `CommandButtonAction::DcBuild` already exists and accepts any `ObjectEnum`.

### Downstream integration (already exists, no changes needed)
- `execute_command_action` (command_panel.rs:1154) handles `DcBuild` generically — calls `DeploymentCenterState::construction_cost()` and deducts crystals.
- `dc_construction_tick_system` (faction.rs:271) already ticks any DC construction using the cost table.
- `ef_construction_tick_system` (faction.rs:859) handles EF-specific internal construction after placement.
- Label rendering in `get_grid_slot_label` and cost display already handle `DcBuild` generically.

## Dependencies

None. ExtractionFacility already exists as a fully functional ObjectEnum variant with spawn logic, state management, and construction tick systems. This task only wires it into the DC's build menu and cost table.
