# Unit Behaviors Module Split

## Date: 2026-03-24
## Status: Architecture request sent
## Priority: 2 (parallel with other splits)
## Parent: codebase-refactoring-priorities.md

## Problem Statement

`src/game/units/systems/behaviors.rs` is 4,481 lines containing all unit behavior state machine systems: movement behaviors, resource gathering, tunnel interaction, supply chopper behaviors, and Cults recruitment. These are distinct behavior domains that share minimal logic, making the single file unnecessarily large and difficult to navigate.

## Current Structure Analysis

### 1. Movement Behaviors (lines 52-465, ~410 lines)
- `moving_to_location_system()` - handles UnitCommand::Move waypoint following
- `moving_to_object_system()` - handles moving toward a target entity
- `reversing_to_location_system()` - handles reverse movement
- `stopping_behavior_system()` - handles deceleration to stop
- `behavior_completion_system()` - transitions completed behaviors to Idle

### 2. Behavior Helpers (lines 411-465, ~55 lines)
- `update_path_index()` - updates path index in BaseBehaviorState
- `set_glider_circling()` - sets circling flag for gliders
- `extract_path_data()` - extracts path data from behavior state
- `has_path_deviation()` - checks if unit has drifted off path

### 3. Tunnel Enter/Exit Behaviors (lines 466-691, ~225 lines)
- `enter_command_dispatch_system()` - dispatches Enter commands to tunnel or armory
- `entering_tunnel_behavior_system()` - handles walking to tunnel and entering
- `enter_armory_dispatch_system()` - dispatches armory entry
- `entering_armory_behavior_system()` - handles walking to armory and entering

### 4. Building Behaviors (lines 692-1110, ~420 lines)
- `building_behavior_system()` - handles structure construction by agents
- `tunnel_side_world_position()` - utility for computing tunnel side positions
- `find_nearest_own_tunnel()` - finds nearest friendly tunnel
- `drop_off_side_for_carry()` - determines drop-off side based on carry type

### 5. Resource Gathering Behaviors (lines 828-1110, ~280 lines)
- `gathering_resource_behavior_system()` - handles agent resource gathering state machine
- `dropping_off_resources_behavior_system()` - handles agent resource drop-off

### 6. Tunnel Building Behavior (lines 1111-1289, ~180 lines)
- `building_tunnel_behavior_system()` - handles agent tunnel construction state machine

### 7. Supply Chopper Behaviors (lines 1290-1597, ~310 lines)
- `supply_chopper_pickup_system()` - chopper picking up supplies
- `supply_chopper_attach_system()` - chopper attaching to tower
- `supply_chopper_dropoff_system()` - chopper dropping off supplies
- `supply_chopper_detach_system()` - chopper detaching from tower
- `supply_chopper_repair_system()` - chopper repairing structures

### 8. Cults Recruitment Behavior (lines 1598-1669, ~70 lines)
- `cults_recruit_enter_construction_system()` - Cults recruit entering construction site

### 9. Tests (lines 1670-4481, ~2,810 lines)

## Proposed Refactoring

Convert `behaviors.rs` into a `behaviors/` directory module:

### New Directory Structure

```
src/game/units/systems/behaviors/
    mod.rs                  -- re-exports all public systems
    movement.rs             -- moving_to_location_system, moving_to_object_system, reversing_to_location_system, stopping_behavior_system, behavior_completion_system, helper functions
    tunnel_interaction.rs   -- enter_command_dispatch_system, entering_tunnel_behavior_system, enter_armory_dispatch_system, entering_armory_behavior_system
    construction.rs         -- building_behavior_system, building_tunnel_behavior_system, cults_recruit_enter_construction_system
    resource_gathering.rs   -- gathering_resource_behavior_system, dropping_off_resources_behavior_system, tunnel_side_world_position, find_nearest_own_tunnel, drop_off_side_for_carry
    supply_chopper.rs       -- all supply_chopper_* systems, supply_chopper_repair_system
```

### Key Decisions

1. **Movement behaviors stay together**: The 5 movement-related systems plus the 4 helper functions form a cohesive group. They all manipulate `BaseBehaviorState`, `LocomotionChannel`, and `OrientationChannel` in the same way.

2. **Construction groups building + tunnel building + cults construction**: These three systems share the same pattern (agent walks to site, performs action, structure progresses). They share helper utilities and the `BuildingStructureBehavior`/`BuildingTunnelBehavior` state types.

3. **Resource gathering is separate from construction**: Even though agents do both, the state machines are different (`GatherPhase` vs `BuildingStructureBehavior`). Resource gathering has its own drop-off logic and carry state management.

4. **Supply chopper is isolated**: The 5 chopper behavior systems are entirely self-contained with no shared logic with other behaviors. They operate on `SupplyChopperState` and `PickUpPhase`/`DropOffPhase` state machines.

5. **Shared helpers go with their primary consumer**: `tunnel_side_world_position`, `find_nearest_own_tunnel`, and `drop_off_side_for_carry` are used primarily by resource gathering and tunnel interaction. They go in `resource_gathering.rs` since that is their heaviest consumer, with `pub(super)` visibility so `tunnel_interaction.rs` can use them.

## Scope

### Files Created
- `src/game/units/systems/behaviors/mod.rs`
- `src/game/units/systems/behaviors/movement.rs`
- `src/game/units/systems/behaviors/tunnel_interaction.rs`
- `src/game/units/systems/behaviors/construction.rs`
- `src/game/units/systems/behaviors/resource_gathering.rs`
- `src/game/units/systems/behaviors/supply_chopper.rs`

### Files Deleted
- `src/game/units/systems/behaviors.rs` (replaced by directory module)

### Files Modified
- `src/game/units/systems/mod.rs` -- `pub mod behaviors` stays the same (resolves to `behaviors/mod.rs`)

### Expected Line Counts (approximate, including tests)
- movement.rs: ~465 + ~800 tests = ~1,265
- tunnel_interaction.rs: ~225 + ~400 tests = ~625
- construction.rs: ~250 + ~500 tests = ~750
- resource_gathering.rs: ~330 + ~700 tests = ~1,030
- supply_chopper.rs: ~310 + ~400 tests = ~710
- mod.rs: ~30 lines

No file exceeds ~1,300 lines. All are well within the target of 2,000 lines max.

## Constraints

- Behavioral equivalence: identical behavior before and after
- All existing unit tests and integration tests must pass
- System registration paths must work via re-exports
- The `WAYPOINT_REACHED_THRESHOLD` and other constants must remain accessible to the modules that use them (move to the appropriate submodule or to a shared constants section in mod.rs)

## Success Metrics

- `behaviors.rs` replaced by 6 files, none exceeding 1,300 lines
- Each file corresponds to a single behavior domain
- Adding a new behavior type (e.g., a new unit's unique behavior) requires creating one new file or editing one existing file
- Supply chopper behaviors are fully isolated from infantry behaviors
