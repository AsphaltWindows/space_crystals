# action-channels-attackmove-target-lock

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-action_channels_r1.md

## Task

Fix Attack-Move so that when a unit acquires a target, it stays locked on that target through the full attack cycle (Aiming → Firing → Cooldown → Reloading → repeat) until the target dies or moves out of range, before resuming the Attack-Move path.

### Root Cause

The `attack_move_behavior_system` (combat/systems/behaviors.rs, ~lines 207-362) has a leash check (~lines 252-281) that disengages the target when the unit drifts more than `ATTACK_MOVE_LEASH_DISTANCE` (6.0 gu) from its `AttackMoveOrigin`. This fires prematurely during the multi-phase attack cycle: after one shot, if the unit has any momentum during Cooldown/Reloading that carries it past the leash distance, it disengages and resumes pathing to the original destination. The result is the unit fires once then moves on.

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
- `artifacts/developer/src/game/combat/systems/behaviors.rs` — attack_move_behavior_system (~lines 207-362)
- `artifacts/developer/src/game/combat/types.rs` — ATTACK_MOVE_LEASH_DISTANCE constant
