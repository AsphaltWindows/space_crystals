# action-channels-attackmove-target-lock

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-action_channels_r1.md

## Task

Fix Attack-Move so that when a unit acquires a target, it stays locked on that target through the full attack cycle (Aiming → Firing → Cooldown → Reloading → repeat) until the target dies or moves out of range, before resuming the Attack-Move path.

### Root Cause

The `attack_move_behavior_system` (combat/systems/behaviors.rs, ~lines 218-373) has a leash check (~lines 264-291) that disengages the target when the unit drifts more than `ATTACK_MOVE_LEASH_DISTANCE` (6.0 gu) from its `AttackMoveOrigin`. This fires prematurely during the multi-phase attack cycle: after one shot, if the unit has any momentum during Cooldown/Reloading that carries it past the leash distance, it disengages and resumes pathing to the original destination. The result is the unit fires once then moves on.

### Required Changes

1. **Sustained target lock**: When `attack_move_behavior_system` has an active target (`attack_state.target_entity().is_some()`) AND the target is still alive and within attack range, do NOT apply the leash check. The leash should only disengage the unit if the target has moved out of weapon range OR died — not based on distance from origin.

2. **Leash rework**: The `AttackMoveOrigin` / leash distance pattern should be reworked:
   - While actively engaged (AttackPhase != None and target is valid and in range): stay locked, ignore leash.
   - If target dies or moves out of range: clear target, record current position as new origin, resume scanning/pathing.
   - The 6.0 gu leash should only apply to how far the unit pursues a fleeing target beyond its weapon range (chase distance), not to drift during the attack cycle itself.

3. **Re-engagement after kill**: After a target dies, the unit should scan for new enemies before resuming movement. Only resume movement toward the Attack-Move destination if no enemies are in range.

### Verification

- Send a unit on Attack-Move near enemies. When it engages, it should keep attacking that target through multiple Aiming → Firing → Cooldown → Reloading cycles until the target dies.
- After the target dies, unit should scan for new nearby enemies before resuming movement.
- If target flees far out of range (beyond weapon range + leash), unit should disengage and resume Attack-Move.

### Key Files
- `artifacts/developer/src/game/combat/systems/behaviors.rs` — attack_move_behavior_system (~lines 218-373)
- `artifacts/developer/src/game/combat/types.rs` — ATTACK_MOVE_LEASH_DISTANCE constant

## Technical Context

### Primary File: `artifacts/developer/src/game/combat/systems/behaviors.rs`

**Function**: `attack_move_behavior_system` (lines 218-373)

The system queries units with `UnitCommand::AttackMove(destination)` and processes them each tick. Current flow:

1. **Arrival check** (lines 249-261): If within 0.5 gu of destination → go Idle, remove MoveTarget/Path/AttackMoveOrigin.
2. **Leash check** (lines 263-316): If engaged (`attack_state.target_entity().is_some()`), checks distance from `AttackMoveOrigin`. If > 6.0 gu → disengages, re-pathfinds. Also checks if target is dead → disengages. **THIS IS THE BUG**: leash check runs regardless of attack phase, causing premature disengage.
3. **Scanning** (lines 318-336): If not engaged, scans for nearest enemy within `SightRange`.
4. **Engagement** (lines 337-351): Sets `AttackState` target, inserts `AttackMoveOrigin(pos)`. Infantry (non-turret, non-glider) removes MoveTarget/Path to stop moving.
5. **Pathing** (lines 352-371): If no enemies, ensures MoveTarget exists toward destination.

### Required Modifications to `attack_move_behavior_system`:

**Replace the leash check block (lines 263-316)** with this logic:

```
if let Some(target_entity) = attack_state.target_entity() {
    // Check if target is still alive
    if let Ok((_te, target_transform, _to, _td, _tg)) = potential_targets.get(target_entity) {
        // Target alive — check if still in weapon range
        let dist_to_target = pos.distance(target_transform.translation);
        let weapon_range = _attack_cap.range;

        if dist_to_target <= weapon_range + ATTACK_MOVE_LEASH_DISTANCE {
            // In range (weapon range + chase buffer) — stay locked
            continue; // Let attack_phase_system handle the engagement
        } else {
            // Target fled beyond weapon range + chase distance — disengage
            disengage_and_resume(...)
        }
    } else {
        // Target dead — scan for new enemies before resuming path
        // (inline the scanning logic or factor into helper)
        // If new enemy found → engage it, update AttackMoveOrigin to current pos
        // If no enemy found → disengage and resume path
    }
    continue;
}
```

**Key change**: The `AttackCapability` component is already in the query as `_attack_cap` (line 226, field index 2). Use `_attack_cap.range` (the `range: f32` field on `AttackCapability`, types.rs line 38) as the weapon range.

**On target death — scan before resuming**: Instead of immediately re-pathfinding when target dies, copy the scanning logic (lines 318-336) to check for new nearby enemies first. If found, engage the new target and update `AttackMoveOrigin` to current position. Only resume pathing if no enemies in scan range.

### Secondary File: `artifacts/developer/src/game/combat/types.rs`

- `ATTACK_MOVE_LEASH_DISTANCE: f32 = 6.0` (line 380): Semantics change from "max drift from origin" to "max chase distance beyond weapon range". The constant value stays the same, but update the doc comment (line 378) to reflect the new meaning.
- `AttackMoveOrigin` (line 385): Semantics change — now represents where the unit was when it started chasing (updated when re-engaging after a kill), not the original scan position. Update doc comment (line 382).
- `AttackCapability.range` (line 38): Weapon range in grid units. Read-only — do not modify.
- `AttackPhase` (line 95): `None | Aiming | Firing | Cooldown | Reloading`. Check `phase != AttackPhase::None` to confirm actively attacking.

### Key Types

- `AttackState` (types.rs:138): `phase: AttackPhase`, `current_target: Option<AttackTarget>`, `time_in_phase: f32`
- `AttackState::target_entity()` (types.rs:156): Returns `Option<Entity>` — `Some` only for `UnitTarget` variant
- `AttackCapability` (types.rs:36): Has `range: f32` (weapon range in gu)
- `AttackMoveOrigin(Vec3)` (types.rs:385): Component, tuple struct wrapping position
- `AttackPhase` (types.rs:95): Enum with `None`, `Aiming`, `Firing`, `Cooldown`, `Reloading`
- `MoveTarget(Vec3)` / `Path { waypoints, current_waypoint }`: Legacy movement components used by combat behaviors

### Existing Tests (behaviors.rs lines 557-738)

Update these tests to match new semantics:
- `attack_move_leash_distance_is_6_gu` (line 564): Keep as-is, value unchanged.
- `within_leash_distance` (line 710) / `beyond_leash_distance` (line 718): These test raw distance math — may need updating if the leash semantics change to weapon_range + leash.
- Add new tests:
  - Unit stays engaged through full attack cycle (Aiming → Firing → Cooldown → Reloading → Aiming)
  - Unit disengages when target moves beyond weapon_range + ATTACK_MOVE_LEASH_DISTANCE
  - Unit scans for new targets after kill before resuming movement
  - Unit resumes Attack-Move path when no enemies remain after kill

### System Ordering / ECS Considerations

- `attack_move_behavior_system` runs in the combat behavior set alongside `attack_phase_system` (core.rs:44) which drives the AttackPhase state machine. No explicit ordering between them — they communicate via `AttackState` component.
- The `attack_phase_system` reads `AttackState` and advances phases on timer. The behavior system should NOT advance phases — just set/clear targets.
- `potential_targets` query (line 231) requires `With<ObjectInstance>` — dead entities are despawned, so `.get(target_entity).is_err()` correctly detects death.

### Pattern: Extracting Helper Functions

The re-pathfind-to-destination logic is duplicated 4 times in the current function (lines 277-289, 301-313, 359-369, and the "no enemies" block). Consider extracting a helper, but this is optional — the task description doesn't require it. Focus on the behavioral fix.

## Dependencies

- **`attack_phase_system`** (combat/systems/core.rs:44): Drives `AttackState.phase` through the cycle. This task depends on it continuing to work unchanged — the fix relies on `phase != AttackPhase::None` to detect active engagement. No changes needed to this system.
- **`base_auto_target_system`** (combat/systems/core.rs:355): Auto-targets idle units. Should NOT interfere with AttackMove units since they have `UnitCommand::AttackMove`, not `UnitCommand::Idle`. No conflict.
- No dependency on other planned_tasks — this is a self-contained behavioral fix within the existing `attack_move_behavior_system`.
