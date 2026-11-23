# attack_command_rightclick_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_command_system_r1.md

## Task

Fix right-click attack command so units pursue and attack enemy targets instead of stopping in place.

**Bug**: When a unit is selected and the player right-clicks an enemy unit or building, the unit stops moving instead of pursuing and attacking. The UnitCommand::AttackTarget is being issued (core.rs ~line 351) but the unit does not move toward the target.

**Investigation areas**:
- Check system ordering: `attacking_object_behavior_system` (combat/systems/behaviors.rs line 22) must run after the UnitCommand component is inserted. The system reads `UnitCommand::AttackTarget(target)` and sets MoveTarget+Path for approach when out of range (line 96-118). Verify this system is actually matching entities — check that all required query components (AttackCapability, AttackState, UnitCommand, UnitBaseEnum, Owner, GridPosition) are present on spawned units.
- Check if `clear_movement_state_full` (called at core.rs line 349 before insert) is removing something that prevents the behavior system from working on the same or next tick.
- Check if there is a conflicting system (e.g., `behavior_completion_system`, `stopping_behavior_system`) that resets AttackTarget to Idle before the attack behavior system runs.
- Check if the `command_dequeue_system` (commands.rs line 228) is prematurely dequeuing when the command was inserted directly (not queued).
- Test with a Peacekeeper right-clicking an enemy unit — verify the unit moves toward and attacks it.
- Test with a unit right-clicking an enemy building — verify same behavior.

## Technical Context

### Attack Command Issuance (right-click on enemy)

The right-click handler is `right_click_move_command` at `artifacts/developer/src/game/units/systems/core.rs:243`. The attack branch (lines 339-358) fires when `should_attack` is true (CommandType::Attack mode or Default + right-click + enemy):

1. Calls `clear_movement_state_full(&mut entity_cmds)` — removes MoveTarget, Path, HoldingPosition, IdleOrigin, and inserts Velocity(Vec3::ZERO) (all via deferred Commands)
2. Calls `issue_or_queue_command(..., UnitCommand::AttackTarget(target_entity), shift_held)` — when !shift_held, clears CommandQueue and inserts UnitCommand via deferred Commands

This system runs in `DiagCategory::Movement` (registered at `artifacts/developer/src/game/units/mod.rs:87`).

### Attack Approach System

The approach movement is handled by `attacking_object_behavior_system` at `artifacts/developer/src/game/combat/systems/behaviors.rs:22`. It:
- Queries entities with: `(Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand, &UnitBaseEnum, &Owner, Option<&DomainEnum>, &GridPosition, Option<&MoveTarget>, Option<&mut TurretCommandState>)` with `With<Unit>`
- Filters for `UnitCommand::AttackTarget(target)`
- When out of range (line 95-119): calls `find_path_for_domain` and inserts `MoveTarget` + `Path` via Commands
- When in range (line 73-94): sets AttackState.current_target, removes MoveTarget/Path (infantry stops to fire)
- Registered in `DiagCategory::Combat` at `artifacts/developer/src/game/combat/mod.rs:29`

### System Ordering Chain

From `artifacts/developer/src/simulation/mod.rs:30-41`:
- `DiagCategory::Movement` runs after `DiagCategory::UiHud`
- `DiagCategory::Combat` runs after `DiagCategory::Movement`
- `DiagCategory::Commands` has NO ordering constraint (free-floating)

Bevy 0.17 automatically inserts `apply_deferred` sync points between ordered system sets. So the deferred UnitCommand::AttackTarget insert from Movement SHOULD be applied before Combat systems read it. The MoveTarget+Path inserts from Combat are applied at the end of Combat, available to Movement systems on the next frame.

### Potential Root Causes to Investigate

**1. Deferred command timing (most likely)**: Verify that the deferred `UnitCommand::AttackTarget` insert from `right_click_move_command` is actually applied before `attacking_object_behavior_system` runs. Bevy 0.17 auto-inserts `apply_deferred` between ordered sets, but confirm this is working. If NOT, the combat system still sees the old `UnitCommand::Idle` and skips the entity.

**Debug approach**: Add a log line inside `attacking_object_behavior_system` at the start that counts how many entities have `UnitCommand::AttackTarget`. If it's always 0 on the first frame after right-click, the commands aren't being applied in time.

**2. attack_channel_sync_system interference**: `attack_channel_sync_system` (combat/systems/core.rs:575) runs in Combat, same set as `attacking_object_behavior_system`, with no relative ordering. It writes `LocomotionChannel::Stationary` when `AttackPhase` is Aiming/Firing/Cooldown. If `attack_command_system` (core.rs:22) sets AttackPhase too early, this could force the locomotion channel to Stationary, conflicting with the approach path.

However: `attack_command_system` sets `AttackPhase::None` (line 35), and the phase system only transitions to Aiming when in range (line 92-98). So this should NOT be the issue unless there's a race.

**3. behavior_completion_system false positive**: `behavior_completion_system` (units/systems/behaviors.rs:378) sets `UnitCommand::Idle` for Move/Reverse/Stop commands when `LocomotionChannel::Stationary` + velocity near zero. It does NOT trigger for `AttackTarget` (line 388-394 explicitly only matches Move/Reverse/Stop). This should NOT interfere.

**4. command_dequeue_system race**: `command_dequeue_system` (commands.rs:228) only acts when command is `Idle` and queue is non-empty. Since we're inserting `AttackTarget` directly (not queueing), this shouldn't interfere. BUT: `DiagCategory::Commands` is unordered relative to Combat and Movement. If command_dequeue runs between the deferred insert of AttackTarget and the combat system reading it, there could be a brief Idle window. This is unlikely but worth checking.

**5. Two movement pipelines conflict**: The `attacking_object_behavior_system` uses the OLD MoveTarget/Path pipeline for approach. Peacekeepers have both `TurnRateMovementParams` (for old pipeline) and `LocomotionChannel`/`OrientationChannel` (for new pipeline). The `turn_rate_movement_system` (core.rs:1285) processes MoveTarget+Path for entities with TurnRateMovementParams. The `channel_turnrate_locomotion_system` (core.rs:2552) processes channel state for entities WITHOUT MoveTarget. These should not conflict because they use mutual `Without<>` exclusions. BUT: after `clear_movement_state_full` removes MoveTarget and before the combat system re-adds it, the channel system might pick up the entity for one frame and do something unexpected.

### Key Files to Investigate and Modify

1. **`artifacts/developer/src/game/combat/systems/behaviors.rs`** (line 22-121) — `attacking_object_behavior_system`: The core system that drives attack approach. Add debug logging to diagnose whether it's matching entities and finding paths.

2. **`artifacts/developer/src/game/units/systems/core.rs`** (line 339-358) — The attack branch of `right_click_move_command`: Where the command is issued. May need to insert approach path here directly (like the Move command does at line 754-764) instead of delegating to the combat behavior system.

3. **`artifacts/developer/src/game/combat/mod.rs`** (line 26-45) — `CombatPlugin` registration: System ordering within the Combat set. No internal ordering between `attacking_object_behavior_system`, `attack_command_system`, `attack_phase_system`, and `attack_channel_sync_system`.

4. **`artifacts/developer/src/simulation/mod.rs`** (line 30-41) — DiagCategory ordering: May need to add ordering for Commands set.

5. **`artifacts/developer/src/game/units/utils.rs`** (line 141-148) — `clear_movement_state_full`: Used before attack command insert.

### Peacekeeper Spawn Components (verified complete for query match)

`spawn_peacekeeper` at `artifacts/developer/src/game/utils.rs:399` spawns with ALL components needed by `attacking_object_behavior_system`'s query:
- `Unit` marker (line 455)
- `AttackCapability` (line 467)
- `AttackState::default()` (line 468)
- `UnitCommand::Idle` (line 469)
- `UnitBaseEnum` via `type_data.unit_base` (line 462)
- `Owner` (line 457)
- `GridPosition` (line 461)
- `Transform` (line 454)
- `ObjectInstance` (line 456) — needed by targets query, not units query

### Pattern Reference: How Move Command Works (for comparison)

The Move command (ground-click, core.rs lines 730-768) does pathfinding INLINE — it calls `find_path_for_domain` and inserts MoveTarget + Path + UnitCommand::Move all in one deferred Commands batch. This means the movement system sees everything on the NEXT frame in one atomic update.

The Attack command (entity-click, core.rs lines 339-358) does NOT do inline pathfinding. It only inserts UnitCommand::AttackTarget and clears movement state, relying on `attacking_object_behavior_system` (in the later Combat set) to do the pathfinding. This two-step approach is architecturally sound but introduces a potential timing gap.

### Recommended Fix Approach

If the deferred command timing is confirmed as the issue, the most robust fix is to make the attack branch in `right_click_move_command` insert the initial approach path inline (matching the Move command pattern), while keeping `attacking_object_behavior_system` for ongoing pursuit (re-pathing when target moves). This eliminates the one-frame gap.

Alternatively, if the issue is something else (e.g., a conflicting system resetting the command), add targeted debug logging first to identify the exact frame where the command gets lost.

### Testing Requirements

- Write a unit test that spawns a unit with AttackTarget, a valid target entity, the required resources (GridMap, OccupancyMap, tiles, ElevationMap), and runs `attacking_object_behavior_system` — verify MoveTarget+Path are inserted.
- Write an integration test for the right-click-enemy flow using the existing `create_right_click_entity_world` test helper (core.rs:4116) with `CursorTargetEnum::EnemyObject` — verify UnitCommand::AttackTarget is set on the selected unit.
- Manual test: select Peacekeeper, right-click enemy unit, verify pursuit and engagement.

## Dependencies

None — this is a bug fix on existing systems. The relevant systems (`right_click_move_command`, `attacking_object_behavior_system`, `attack_phase_system`) all already exist and are registered.
