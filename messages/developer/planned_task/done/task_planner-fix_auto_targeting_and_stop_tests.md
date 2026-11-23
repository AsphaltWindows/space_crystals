# fix_auto_targeting_and_stop_tests

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Fix 5 failing integration tests: 3 auto-targeting tests (Category A) and 2 stop command tests (Category B).

### Category A: Auto-targeting not engaging (3 tests)

**Root cause**: base_auto_target_system requires targets to pass is_valid_target(), which checks VisibilityStateEnum. The test harness spawn_unit_at_grid does not insert VisibilityStateEnum::Visible on spawned units, so enemy targets don't even match the potential_targets query.

**Fix**: After spawning enemy units in tests, set VisibilityStateEnum::Visible on them.

**Tests to fix**:
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting
- combat_behaviors::step_10_hold_position_engages_nearby

### Category B: Stop command behavior mismatch (2 tests)

**Test 1**: basic_combat_unit_interface_state::step_3_stop_immediate_command
- Fix: Change test expectation to assert Idle (correct behavior for Stop on stationary unit)

**Test 2**: movement_behaviors::step_8_stop_command
- Fix: Either increase frame count or investigate deceleration/system ordering

## Technical Context

### Category A Fix: VisibilityStateEnum on spawned units

**Primary file to modify**: `artifacts/developer/src/shared/testing/harness.rs`

The best approach is to fix `spawn_unit_at_grid` (line 47) itself to always insert `VisibilityStateEnum::Visible` on spawned units. This fixes all current and future auto-targeting tests without per-test changes.

**How it works**: `base_auto_target_system` (artifacts/developer/src/game/combat/systems/core.rs:474) queries potential targets with `&VisibilityStateEnum` as a required component (line 480):
```rust
potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition, &ObjectInstance, &VisibilityStateEnum, Option<&AttackCapability>), With<ObjectInstance>>,
```

If the component is absent, the entity doesn't match the query at all -- it's invisible to the auto-targeting system.

`is_valid_target` (artifacts/developer/src/game/combat/utils.rs:278) then checks `*target_visibility != VisibilityStateEnum::Visible` (line 290), filtering out non-Visible entities.

**Implementation**: After each spawn in `spawn_unit_at_grid`, insert `VisibilityStateEnum::Visible` on the returned entity. Since the method uses `run_system_once` which returns the Entity id, add after line 62/71/80:
```rust
// After getting the entity id from spawn
let entity_id = self.app.world_mut().run_system_once(...).unwrap();
self.app.world_mut().entity_mut(entity_id).insert(VisibilityStateEnum::Visible);
entity_id
```

OR, refactor to collect the entity and insert at the end of the method (cleaner):
```rust
pub fn spawn_unit_at_grid(&mut self, unit_type: ObjectEnum, grid_x: i32, grid_z: i32, owner: Owner) -> Entity {
    let entity = match unit_type {
        ObjectEnum::Peacekeeper => { /* existing spawn code */ },
        ObjectEnum::SyndicateAgent => { /* existing spawn code */ },
        ObjectEnum::SupplyChopper => { /* existing spawn code */ },
        _ => panic!(...),
    };
    // All test-spawned units should be visible for targeting/combat tests
    self.app.world_mut().entity_mut(entity).insert(VisibilityStateEnum::Visible);
    entity
}
```

**Import**: `VisibilityStateEnum` is already available via `use crate::types::*;` (line 3 of harness.rs) since it's defined in `artifacts/developer/src/shared/types.rs:435`.

**Also fix spawn_structure_at_grid** (line 111): Same issue -- structures spawned in tests should also have `VisibilityStateEnum::Visible` for consistency, especially for combat tests targeting structures.

### Category A: Test file changes (likely none needed)

The 3 failing tests in `tests/qa/autonomous_targeting.rs` and `tests/qa/combat_behaviors.rs` should pass without modification once spawn_unit_at_grid inserts `VisibilityStateEnum::Visible`. However, note the soft assertion pattern in these tests:
- step_1 (line 30): `if let Some(attack_state) = world.get::<AttackState>(player_unit)` -- silently passes if AttackState missing
- step_6 (line 62): same pattern
- step_10 (line 296): same pattern

These will be hardened in the companion `harden_assertions_and_cleanup` task. For this task, just ensure the tests pass with the harness fix.

### Category B Fix: Stop command tests

**File 1**: `artifacts/developer/tests/qa/basic_combat_unit_interface_state.rs` (line 37)

The test `step_3_stop_immediate_command` spawns a unit, issues Stop, steps once, then asserts `UnitCommand::Stop`. But:
1. `stopping_behavior_system` (artifacts/developer/src/game/units/systems/behaviors/movement.rs:318) runs, sees `UnitCommand::Stop`, sets `LocomotionChannel::Stopping`
2. Since velocity is already Vec3::ZERO (unit never moved), it checks `velocity.0.length() < STOPPED_VELOCITY_THRESHOLD` (0.01) -- TRUE
3. Sets `LocomotionChannel::Stationary` and `BaseBehaviorState::None`
4. `behavior_completion_system` (line 356) then sees `UnitCommand::Stop` + `LocomotionChannel::Stationary` + near-zero velocity, transitions to `UnitCommand::Idle`

**Fix**: Change assertion from `matches!(cmd, Some(UnitCommand::Stop))` to `matches!(cmd, Some(UnitCommand::Stop) | Some(UnitCommand::Idle))` with a comment explaining that Stop on a stationary unit completes instantly to Idle. Alternatively, assert specifically `UnitCommand::Idle` since that's the correct final state.

**File 2**: `artifacts/developer/tests/qa/movement_behaviors.rs` (line 136)

The test `step_8_stop_command` issues Move (5 frames), then Stop (10 frames), checks velocity < 0.1. The Peacekeeper has `deceleration: f32::MAX` (artifacts/developer/src/game/utils.rs:447), and the movement system (artifacts/developer/src/game/units/systems/core.rs:2668) has instant decel for `deceleration >= f32::MAX / 2.0` (sets speed to 0.0 immediately). 

Possible issue: the first frame after Stop may have delta_secs = 0 (Bevy's Time not yet ticked), causing the movement system to early-return (line 2526: `if delta < 0.0001 { return; }`). But subsequent frames should work.

**Investigation approach**: Run the test to see current behavior. If velocity is still ~1.0 after 10 frames, the issue is likely:
1. `stopping_behavior_system` runs but movement system already processed this frame (ordering)
2. OR the movement path system is still pushing velocity forward

**Fix options**:
- Increase frame count to 30-50 to be safe
- OR check if there's a system ordering issue between behaviors and movement (behaviors are Phase 2, movement is Phase 3 -- see artifacts/developer/src/game/units/mod.rs:18-58 for ordering)

The system ordering in mod.rs shows behaviors (Phase 2) run `.after(rebuild_occupancy_map)` and movement (Phase 3) runs `.after(behaviors)`. This is correct. With 10 frames and instant deceleration, the unit should stop. Try running the test first -- it may already work after the harness fix inserts VisibilityStateEnum (the test environment may have changed since the architect's analysis).

### Key types and their locations

- **VisibilityStateEnum**: `artifacts/developer/src/shared/types.rs:435` -- enum { Unexplored, Explored, Visible }
- **is_valid_target()**: `artifacts/developer/src/game/combat/utils.rs:278` -- checks destructible, visible, domain-compatible
- **base_auto_target_system()**: `artifacts/developer/src/game/combat/systems/core.rs:474` -- queries targets with &VisibilityStateEnum
- **stopping_behavior_system()**: `artifacts/developer/src/game/units/systems/behaviors/movement.rs:318` -- sets LocomotionChannel::Stopping, checks velocity threshold
- **behavior_completion_system()**: `artifacts/developer/src/game/units/systems/behaviors/movement.rs:356` -- Stop + Stationary + low velocity -> Idle
- **STOPPED_VELOCITY_THRESHOLD**: 0.01 (behaviors/movement.rs:17)
- **TurnRateMovementParams** for Peacekeeper: deceleration=f32::MAX, acceleration=f32::MAX (game/utils.rs:446-448)
- **channel_turnrate_locomotion_system**: `artifacts/developer/src/game/units/systems/core.rs:2516` -- handles Stopping with instant decel at line 2668-2672

### Verification commands

```bash
cd artifacts/developer && cargo test --test qa autonomous_targeting
cd artifacts/developer && cargo test --test qa combat_behaviors::step_10
cd artifacts/developer && cargo test --test qa basic_combat_unit_interface_state::step_3
cd artifacts/developer && cargo test --test qa movement_behaviors::step_8
cd artifacts/developer && cargo test --lib  # no regressions
```

## Dependencies

- **test_harness_selection_fix** (already completed): The Selection resource fix in set_selection() is independent but was a prerequisite for Category C tests. This task does not depend on it directly.
- **harden_assertions_and_cleanup** (parallel task): That task will harden the soft `if let Some` assertions in these same test files. This task should complete first so the tests pass, then the hardening task converts soft assertions to unwrap/expect. No blocking dependency -- they can proceed in parallel as long as this task fixes the root causes first.
