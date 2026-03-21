# gdo-deployment-center-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-gdo-deployment-center.md

## Task

Verify and fix the DeploymentCenter structure implementation. Almost everything is already implemented correctly. The ONE fix needed:

**Fix SupplyTower build_frames in DC construction catalog:**
In `artifacts/developer/src/game/types/structures.rs`, `DeploymentCenterState::construction_cost()` for `ObjectEnum::SupplyTower` has `build_frames: 160` but the spec requires `build_frames: 240` (15 seconds at 16 fps).

**Verify these already-correct implementations (no changes expected):**
- ObjectType: size (4,4), destructible, sight_range=6, groupable=false
- StructureType: symmetry AAAA
- Constants: DC_MAX_HP=1000, DC_POINT_ARMOR=1, DC_FULL_ARMOR=16, DC_BUILD_RADIUS=12, DC_POWER=20
- DeploymentCenterState: current_construction, construction_progress, ready_to_place fields
- Construction catalog: PowerPlant (150 SC, 160 frames), Barracks (200 SC, 160 frames)
- SupplyTower prerequisite: has_power_plant check in command_panel.rs grid_button_enabled_ext and execute_command_action
- Cancellation: full refund during construction, 75% rounded down when ready_to_place
- spawn_deployment_center: all components (PowerValue, BuildRadiusExtension, DeploymentCenterState, SightRange)

**Update any tests** that assert SupplyTower build_frames == 160 to assert 240 instead.

## Technical Context

### Files to Change

1. **`artifacts/developer/src/game/types/structures.rs` line 102** — Change `build_frames: 160` to `build_frames: 240` in the `ObjectEnum::SupplyTower` arm of `DeploymentCenterState::construction_cost()` (lines 100-103).

2. **`artifacts/developer/src/game/types/structures.rs` line 1999** — Test `supply_tower_construction_cost` (line 1995-2000) asserts `cost.build_frames == 160`. Change to `assert_eq!(cost.build_frames, 240)`.

### Files to Verify (no changes expected)

- **`artifacts/developer/src/game/types/structures.rs`** — `DeploymentCenterState` struct definition, `construction_cost()` for PowerPlant/Barracks/ExtractionFacility, `cancellation_refund()` logic, DC constants.
- **`artifacts/developer/src/ui/command_panel.rs`** — DC build menu grid slots, SupplyTower prerequisite check (`grid_button_enabled_ext`), `execute_command_action` for DC construction. Test at line 3428 (`supply_tower_construction_cost_is_200`) only checks `space_crystals`, not `build_frames` — no change needed.
- **`artifacts/developer/src/game/world/faction.rs`** — `spawn_deployment_center` function, component bundle.
- **`artifacts/developer/src/game/utils.rs`** — DC-related utility functions if any.
- **`artifacts/developer/src/ui/hud.rs`** — DC construction progress display (uses `build_frames` from `construction_cost()` dynamically, so it auto-picks up the fix).

### Pattern Notes

- The `StructureCost` struct has two fields: `space_crystals: u32` and `build_frames: u32`.
- Progress bars in `command_panel.rs` (lines 604, 622, 1692) and `hud.rs` (lines 444, 507) use `cost.map(|c| c.build_frames as f32).unwrap_or(160.0)` — they dynamically read from `construction_cost()`, so the fix propagates automatically.

## Dependencies

None — this is a standalone data fix with a corresponding test update.
