# gdo-deployment-center-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
