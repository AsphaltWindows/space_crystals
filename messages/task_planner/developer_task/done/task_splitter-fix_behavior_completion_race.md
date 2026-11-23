# fix_behavior_completion_race

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-core_systems_split_r1.md

## Task

Fix a race condition in `behavior_completion_system` (in `artifacts/developer/src/game/units/systems/behaviors/movement.rs`, around line 357) that causes 19 QA integration tests to fail with 'got Some(Idle)' errors.

### Root Cause

`behavior_completion_system` runs in the same unordered system group (Phase 2 behaviors) as the movement behavior systems (`moving_to_location_system`, etc.) in `UnitsPlugin::build()` (`src/game/units/mod.rs`). When a `UnitCommand::Move` (or other movement command) is freshly inserted on an entity, the unit still has `LocomotionChannel::Stationary` and zero `Velocity` because the movement behavior system has not yet processed it. If `behavior_completion_system` runs first in the frame, it sees:
- Command is `Move` (should_complete = true)
- `LocomotionChannel::Stationary` + velocity near zero
- Therefore it 'completes' the behavior immediately, resetting the command to `UnitCommand::Idle`

This means the movement behavior system never gets a chance to start the movement.

### Fix

Add a guard in `behavior_completion_system` so it only completes behaviors that have actually started. The simplest approach: check that `BaseBehaviorState` is not `BaseBehaviorState::None` before completing. A newly issued command will have `BaseBehaviorState::None` until the corresponding behavior system (e.g., `moving_to_location_system`) initializes it. Once the behavior system sets a real state (e.g., `BaseBehaviorState::TurnRate`), completion can proceed normally.

Add this guard early in the loop body (after the `should_complete` check):
```rust
// Don't complete commands whose behavior hasn't started yet
if matches!(behavior.as_ref(), BaseBehaviorState::None) {
    continue;
}
```

Alternatively (or additionally), add an explicit ordering constraint in `src/game/units/mod.rs` so `behavior_completion_system` runs `.after()` the movement behavior systems. However, the `BaseBehaviorState::None` guard is the more robust fix since it handles the semantic issue regardless of system ordering.

### Verification

1. `cargo check` must pass
2. `cargo test --lib` must pass (all unit tests)
3. `cargo test --test qa` must pass with 0 failures
4. Specifically verify these representative tests pass:
   - `cargo test --test qa movement_behaviors::step_1_move_to_location`
   - `cargo test --test qa autonomous_targeting::step_1_auto_targeting_engages_enemy`
   - `cargo test --test qa ground_unit_collision::step_2_move_through_idle_unit`
   - `cargo test --test qa pathfinding_diagonal_and_oscillation_fix::step_3_diagonal_move_command`
