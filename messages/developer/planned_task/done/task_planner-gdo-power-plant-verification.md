# gdo-power-plant-verification

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-gdo-power-plant.md

## Task

Verify that the PowerPlant structure implementation matches the design spec. The PowerPlant appears to be fully implemented already. Verify the following are correct and complete:

1. **ObjectEnum::PowerPlant** exists with correct ObjectType (name="Power Plant", size=(2,2), destructible=true, sight_range=3, groupable=true)
2. **StructureType** has correct armor (point_armor=1, full_armor=4) and symmetry (AAAA)
3. **Constants** are correct: PP_MAX_HP=350, PP_POINT_ARMOR=1, PP_FULL_ARMOR=4, PP_BUILD_RADIUS=1, PP_POWER=20
4. **spawn_power_plant()** spawns with all required components: ObjectInstance, StructureInstance, PowerValue(20), BuildRadiusExtension(1), SightRange(3), Owner, Selectable, GridPosition
5. **DeploymentCenter construction**: cost=150 SC, build_frames=160, cancellation refunds correct
6. **ConstructionHP rule**: starts at 10% HP, scales linearly, ConstructionHP component removed on completion
7. **Power grid integration**: compute_power_grid system tracks PowerValue, has_power_plant flag updated
8. **No ObjectInterfaceState**: command panel shows no commands for PowerPlant (info display only)
9. **Build area extension**: BuildRadiusExtension(1) expands GDO build area
10. **Tests pass**: run existing tests to confirm everything works

If everything checks out, no code changes are needed — just confirm the implementation is complete and correct. If any gaps are found, fix them.

## Technical Context

This is a **verification task** — the PowerPlant is already fully implemented. The developer should inspect the following files and run existing tests.

### Files to Inspect (all under `artifacts/developer/src/`)

1. **`game/types/objects.rs`** (lines 246-251, 326-329):
   - `ObjectEnum::PowerPlant` variant defined in the enum
   - `object_type()` returns: name="Power Plant", size=(2,2), destructible=true, sight_range=3, groupable=true
   - `structure_type()` returns: symmetry=AAAA
   - Existing tests at lines 1021-1057 cover all ObjectType fields and symmetry

2. **`game/types/structures.rs`** (lines 399-403):
   - Constants: `PP_MAX_HP=350.0`, `PP_POINT_ARMOR=1`, `PP_FULL_ARMOR=4`, `PP_BUILD_RADIUS=1`, `PP_POWER=20`
   - Existing tests at lines 1120-1139 verify all constant values
   - `DeploymentCenterState::construction_cost(&ObjectEnum::PowerPlant)` returns cost=150 SC, build_frames=160 (tested at line 1168-1172)
   - NOTE: Point/full armor constants exist but verify they're actually used in combat damage calculation (check `combat/` module)

3. **`game/utils.rs`** (lines 192-243):
   - `spawn_power_plant()` spawns: ObjectInstance::destructible(ObjectEnum::PowerPlant, PP_MAX_HP), StructureInstance, Owner, Selectable, SelectionBounds, GridPosition, PowerValue(PP_POWER), BuildRadiusExtension(PP_BUILD_RADIUS), SightRange(3)
   - All required components present per design spec

4. **`game/world/faction.rs`**:
   - `compute_power_grid()` (lines 162-188): iterates all buildings with PowerValue+ObjectInstance, tracks generated/consumed power, sets `has_power_plant = true` when ObjectEnum::PowerPlant found
   - `dc_construction_tick_system()` (line 195+): ticks DC construction progress
   - `construction_hp_tick_system()` (line 821+): HP starts at 10%, scales to 100% linearly, removes ConstructionHP on completion
   - DC construction completion (line ~1388): calls `spawn_power_plant()` when building_type is PowerPlant

5. **`ui/command_panel.rs`**:
   - PowerPlant selected → `StructureMenuState::Inert` (line 386) → no commands shown (line 137-139)
   - DC BuildMenu grid slot (0,0) → `DcBuild(ObjectEnum::PowerPlant)` (line 58)
   - Affordability check: `player_sc >= 150` (line 2251)
   - Button label: "[Q] PP\n150 SC" (line 2184)

6. **`game/world/utils.rs`**:
   - `expand_build_area()` function uses BuildRadiusExtension to expand GDO build area

7. **`shared/testing/harness.rs`** (line 131-135):
   - Test harness supports spawning PowerPlant via `spawn_power_plant()`

### Design Spec Reference
- `artifacts/designer/design/gdo_objects.md` lines 40-55: full PowerPlant spec
- ObjectInterfaceState: None (info display only) — implemented as `StructureMenuState::Inert`

### What to Verify
- Run `cargo test` in `artifacts/developer/` — existing tests cover all 10 checklist items
- Spot-check that armor constants (PP_POINT_ARMOR, PP_FULL_ARMOR) are wired into the damage system (grep for usage in `combat/` module)
- Confirm no missing components by comparing `spawn_power_plant()` output against the design spec component list

### Expected Outcome
No code changes needed. Report confirmation that all 10 points are verified.

## Dependencies

None — this is a standalone verification task with no dependencies on other planned tasks.
