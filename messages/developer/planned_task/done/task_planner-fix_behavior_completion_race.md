# behaviors_split

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-behaviors_split.md

## Task

Split the monolithic `src/game/units/systems/behaviors.rs` (4,481 lines) into a `behaviors/` directory module with 6 files. This is a pure structural refactoring with no behavioral changes.

Read `artifacts/software_architect/architecture/behaviors-split.md` for the full plan.

### Summary of Changes

1. **Create** `src/game/units/systems/behaviors/` directory with these files:
   - `mod.rs` (~30 lines) -- re-exports all public systems and shared constants
   - `movement.rs` (~1,265 lines incl. tests) -- moving_to_location_system, moving_to_object_system, reversing_to_location_system, stopping_behavior_system, behavior_completion_system, plus helper functions (update_path_index, set_glider_circling, extract_path_data, has_path_deviation)
   - `tunnel_interaction.rs` (~625 lines incl. tests) -- enter_command_dispatch_system, entering_tunnel_behavior_system, enter_armory_dispatch_system, entering_armory_behavior_system
   - `construction.rs` (~750 lines incl. tests) -- building_behavior_system, building_tunnel_behavior_system, cults_recruit_enter_construction_system
   - `resource_gathering.rs` (~1,030 lines incl. tests) -- gathering_resource_behavior_system, dropping_off_resources_behavior_system, plus shared helpers (tunnel_side_world_position, find_nearest_own_tunnel, drop_off_side_for_carry) with pub(super) visibility
   - `supply_chopper.rs` (~710 lines incl. tests) -- all supply_chopper_* systems and supply_chopper_repair_system

2. **Delete** `src/game/units/systems/behaviors.rs` (replaced by directory module)

3. **No changes to** `src/game/units/systems/mod.rs` -- `pub mod behaviors` resolves to the new directory module automatically

### Key Requirements

- All existing unit tests (~2,810 lines) must pass in their new submodule locations with only import path updates
- Shared helpers used across submodules must use `pub(super)` visibility
- Constants like WAYPOINT_REACHED_THRESHOLD must remain accessible to all consuming submodules
- No file should exceed 1,300 lines
- System registration paths must work via re-exports from mod.rs
- `cargo check`, `cargo test --lib`, and `cargo test --test qa` must all pass

## Technical Context

### File to Split

`artifacts/developer/src/game/units/systems/behaviors.rs` (4,481 lines)

### Exact Code Boundaries (line numbers in current file)

**Imports and constants**: Lines 1-45
- 29 lines of imports (use statements)
- 5 constants: WAYPOINT_REACHED_THRESHOLD (33), PATH_DEVIATION_THRESHOLD (36), STOPPED_VELOCITY_THRESHOLD (39), OBJECT_PROXIMITY_DISTANCE (42), TARGET_MOVED_THRESHOLD (45)

**Movement systems** (lines 52-454): 
- `moving_to_location_system` (52-153)
- `moving_to_object_system` (155-265)
- `reversing_to_location_system` (267-338)
- `stopping_behavior_system` (340-376)
- `behavior_completion_system` (378-409)
- Helper functions: `update_path_index` (414-424), `set_glider_circling` (426-432), `extract_path_data` (434-444), `has_path_deviation` (446-454)

**Tunnel interaction systems** (lines 456-679):
- `TUNNEL_ARRIVAL_THRESHOLD` constant (456)
- `enter_command_dispatch_system` (466-523)
- `entering_tunnel_behavior_system` (525-573)
- `ARMORY_ARRIVAL_THRESHOLD` constant (576)
- `enter_armory_dispatch_system` (583-624)
- `entering_armory_behavior_system` (626-679)

**Construction + building systems** (lines 681-1289):
- `BUILD_ARRIVAL_THRESHOLD` constant (681)
- `building_behavior_system` (692-755)
- `building_tunnel_behavior_system` (1111-1289) -- NOTE: this is NOT contiguous with building_behavior_system; gathering code sits between them

**Resource gathering** (lines 757-1110):
- `GATHER_ARRIVAL_THRESHOLD` constant (757)
- `tunnel_side_world_position` (762-784)
- `find_nearest_own_tunnel` (786-810)
- `drop_off_side_for_carry` (813-826)
- `gathering_resource_behavior_system` (828-995)
- `dropping_off_resources_behavior_system` (997-1110)

**Supply chopper systems** (lines 1290-1597):
- Constants: `CHOPPER_PICKUP_DURATION` (1293), `CHOPPER_REPAIR_RATE` (1296)
- `supply_chopper_pickup_system` (1304-1387)
- `supply_chopper_attach_system` (1389-1462)
- `supply_chopper_dropoff_system` (1464-1528)
- `supply_chopper_detach_system` (1530-1554)
- `supply_chopper_repair_system` (1556-1597)

**Cults recruit construction** (lines 1598-1669):
- `RECRUIT_CONSTRUCTION_ARRIVAL_THRESHOLD` constant (1603)
- `cults_recruit_enter_construction_system` (1608-1669)

**Test module** (lines 1670-4481, ~2,810 lines): Single `#[cfg(test)] mod tests` block with section markers:
- Constants tests (1676-1703)
- update_path_index tests (1705-1785)
- set_glider_circling tests (1787-1829)
- extract_path_data tests (1831-1862)
- has_path_deviation tests (1864-1909)
- Reversing constraint tests (1911-1937)
- MoveObjectTarget tests (1939-1955)
- EnteringTunnel behavior + dispatch tests (1956-2139) -- includes helper spawners `spawn_agent_for_tunnel_test` (1959), `spawn_tunnel_for_test` (1977)
- BuildingStructureBehavior tests (2141-2312)
- BuildingTunnelBehavior tests (2313-2804)
- Single-Agent Construction Enforcement tests (2806-2953)
- Tunnel Arrival Validation tests (2955-3060)
- Gathering/DropOff helper function tests (3062-3139)
- GatheringResource behavior tests (3141-3365)
- DroppingOffResources behavior tests (3367-3533)
- Supply Chopper constants tests (3535-3545)
- Supply Chopper Pickup tests (3547-3699)
- Supply Chopper Attach tests (3701-3770)
- Supply Chopper Dropoff tests (3772-3844)
- Supply Chopper Detach tests (3846-3886)
- Supply Chopper Repair tests (3888-3976)
- behavior_completion_system tests (3978-4198)
- Cults Recruit Construction tests (4200-4312) -- includes `spawn_test_recruit_for_construction` (4204), `spawn_test_construction_building` (4215)
- EnterArmory dispatch tests (4315-4410) -- includes `spawn_test_armory` (4317), `spawn_test_recruit_for_armory` (4337)
- EnteringArmory behavior tests (4412-4481)

### Test Distribution to Submodules

- **movement.rs tests**: Constants (1676-1703), update_path_index (1705-1785), set_glider_circling (1787-1829), extract_path_data (1831-1862), has_path_deviation (1864-1909), Reversing constraint (1911-1937), MoveObjectTarget (1939-1955), behavior_completion (3978-4198) = ~800 lines
- **tunnel_interaction.rs tests**: EnteringTunnel (1956-2139), Tunnel Arrival Validation (2955-3060), EnterArmory dispatch (4315-4410), EnteringArmory behavior (4412-4481) = ~400 lines
- **construction.rs tests**: BuildingStructureBehavior (2141-2312), BuildingTunnelBehavior (2313-2804), Single-Agent Construction (2806-2953), Cults Recruit Construction (4200-4312) = ~950 lines
- **resource_gathering.rs tests**: Gathering/DropOff helpers (3062-3139), GatheringResource (3141-3365), DroppingOffResources (3367-3533) = ~470 lines
- **supply_chopper.rs tests**: All Supply Chopper test sections (3535-3976) = ~440 lines

### Re-export Requirements (mod.rs)

All `pub fn` systems referenced from `artifacts/developer/src/game/units/mod.rs` lines 20-44 must be re-exported from `behaviors/mod.rs`. The full list:
- From movement: `moving_to_location_system`, `moving_to_object_system`, `reversing_to_location_system`, `stopping_behavior_system`, `behavior_completion_system`, `extract_path_data`, `has_path_deviation`
- From tunnel_interaction: `enter_command_dispatch_system`, `entering_tunnel_behavior_system`, `enter_armory_dispatch_system`, `entering_armory_behavior_system`
- From construction: `building_behavior_system`, `building_tunnel_behavior_system`, `cults_recruit_enter_construction_system`
- From resource_gathering: `gathering_resource_behavior_system`, `dropping_off_resources_behavior_system`, `tunnel_side_world_position`
- From supply_chopper: `supply_chopper_pickup_system`, `supply_chopper_attach_system`, `supply_chopper_dropoff_system`, `supply_chopper_detach_system`, `supply_chopper_repair_system`

### Cross-Module Re-export Chain

`tunnel_side_world_position` has a special re-export chain:
- Defined in `resource_gathering.rs` (with `pub(super)` visibility per architecture doc -- BUT this needs to be `pub` since it is re-exported outside the behaviors module)
- Re-exported from `behaviors/mod.rs`
- Re-exported from `artifacts/developer/src/game/units/utils.rs` line 11: `pub use super::systems::behaviors::tunnel_side_world_position;`
- Consumed by `artifacts/developer/src/game/world/faction.rs` lines 519, 554, 2059, 2096

IMPORTANT: `tunnel_side_world_position` must be `pub` (not `pub(super)`) in resource_gathering.rs because it is used outside the behaviors module entirely (via utils.rs re-export to faction.rs). The architecture doc says `pub(super)` for shared helpers, but that only applies to `find_nearest_own_tunnel` and `drop_off_side_for_carry` which are only used within the behaviors submodules.

### Import Handling

Each submodule needs its own subset of the imports from the current file (lines 1-29). The import block is large; distribute only the imports each file actually uses:

- **movement.rs** needs: `BaseBehaviorState`, `LocomotionChannel`, `OrientationChannel`, `UnitCommand`, `Velocity`, `MoveObjectTarget`, `MoveTarget`
- **tunnel_interaction.rs** needs: `EnteringTunnelBehavior`, `InTunnelNetwork`, `EnteringArmoryBehavior`, `LocomotionChannel`, `OrientationChannel`, `UnitCommand`, `Velocity`, `TunnelState`, `ArmoryState`, `ARMORY_INTERNAL_RECRUIT_CAPACITY`, `can_enter_tunnel`, `Owner`, `UnitBaseEnum`, `ObjectEnum`
- **construction.rs** needs: `BuildingStructureBehavior`, `BuildingTunnelBehavior`, `BuildTunnelPhase`, `LocomotionChannel`, `OrientationChannel`, `UnitCommand`, `ConstructionHP`, `ObjectInstance`, `StructureInstance`, `TunnelState`, `CultsConstructionState`, `tunnel_construction_cost`, `SyndicatePlayerResources`, `Player`, `Tile`, `TilePreset`, `SpaceCrystalPatch`, `can_worker_place_structure`, `world_to_grid`, `spawn_tunnel_under_construction`, `GridPosition`, `Owner`, `Unit`, `ObjectEnum`, `SymmetryTypeEnum`
- **resource_gathering.rs** needs: `GatheringResourceBehavior`, `GatherPhase`, `DroppingOffResourcesBehavior`, `DropOffPhase`, `LocomotionChannel`, `OrientationChannel`, `UnitCommand`, `AgentCarryState`, mining/carry constants, `StructureInstance`, `TunnelState`, `SyndicatePlayerResources`, `Player`, `SupplyDeliveryStation`, `SpaceCrystalPatch`, `Owner`
- **supply_chopper.rs** needs: `PickingUpSuppliesBehavior`, `PickUpPhase`, `AttachingToTowerBehavior`, `DroppingOffSuppliesBehavior`, `LocomotionChannel`, `OrientationChannel`, `UnitCommand`, `SupplyChopperState`, `SupplyTowerState`, `SupplyDeliveryStation`, `GdoPlayerResources`, `ConstructionHP`, `Owner`

### System Registration (No Changes Needed)

`artifacts/developer/src/game/units/mod.rs` references systems as `systems::behaviors::function_name`. Since `mod.rs` uses `pub mod behaviors;` (line 3) and the new `behaviors/mod.rs` will re-export all public systems, all registration paths resolve without changes.

### Constants Strategy

- Movement constants (WAYPOINT_REACHED_THRESHOLD, PATH_DEVIATION_THRESHOLD, STOPPED_VELOCITY_THRESHOLD, OBJECT_PROXIMITY_DISTANCE, TARGET_MOVED_THRESHOLD) go in `movement.rs`
- TUNNEL_ARRIVAL_THRESHOLD, ARMORY_ARRIVAL_THRESHOLD go in `tunnel_interaction.rs`
- BUILD_ARRIVAL_THRESHOLD goes in `construction.rs`
- GATHER_ARRIVAL_THRESHOLD goes in `resource_gathering.rs`
- CHOPPER_PICKUP_DURATION, CHOPPER_REPAIR_RATE go in `supply_chopper.rs`
- RECRUIT_CONSTRUCTION_ARRIVAL_THRESHOLD goes in `construction.rs` (with the cults_recruit_enter_construction_system)

No constants need cross-file sharing -- each constant is only used within its own domain's systems and tests.

### Non-Contiguous Code Warning

`building_tunnel_behavior_system` (lines 1111-1289) is NOT adjacent to `building_behavior_system` (lines 692-755). The resource gathering code (lines 757-1110) sits between them. When splitting, building_tunnel_behavior_system must be moved to `construction.rs` alongside building_behavior_system, not left in resource_gathering.rs.

### Test Module `use super::*` Pattern

Each submodule's test block should use `use super::*;` which imports everything from its parent submodule. For tests that need types from sibling submodules (e.g., construction tests that need tunnel_side_world_position), add explicit imports: `use super::resource_gathering::tunnel_side_world_position;`

The current test module also imports `use bevy::ecs::system::RunSystemOnce;` and `use crate::types::ObjectEnum;` -- these need to be replicated in each submodule's test block that uses them.

### Verification Commands

After completing the split:
1. `cargo check` -- confirms compilation
2. `cargo test --lib` -- confirms all 1907+ unit tests pass (the behaviors tests are a significant chunk)
3. `cargo test --test qa` -- confirms integration tests still compile and pass at same rate

## Dependencies

None. This is a standalone structural refactoring. The file being split (`behaviors.rs`) has no pending modifications from other tasks. The other three split tasks (command_panel, core_systems, faction) affect disjoint files and can proceed in parallel.
