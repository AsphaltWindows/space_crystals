# action-channels-targeted-attack-fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-action_channels_r1.md

## Task

Fix targeted attack (Attack+click enemy / right-click enemy) so the unit pursues and attacks the target instead of stopping in place.

### Root Cause

The `attacking_object_behavior_system` (combat/systems/behaviors.rs, ~lines 22-121) sets `AttackPhase::Aiming` when it finds a target, then uses legacy `MoveTarget/Path` to approach out-of-range targets. However, `attack_channel_sync_system` (combat/systems/core.rs, ~lines 575-631) runs after and forces `LocomotionChannel::Stationary` whenever `AttackPhase` is Aiming/Firing/Cooldown — even when the unit is out of range and needs to approach.

### Required Changes

1. **In `attacking_object_behavior_system`**: When the target is OUT of range, write `LocomotionChannel::Moving(waypoints)` to drive approach movement via channels instead of (or in addition to) legacy MoveTarget/Path. When IN range, write `LocomotionChannel::Stationary`.

2. **In `attack_channel_sync_system`**: Only force `LocomotionChannel::Stationary` during Aiming/Firing/Cooldown when the unit is actually in attack range of the target. If out of range, do NOT override the locomotion channel — let the behavior system's approach movement persist. This may require passing range information through a component or checking distance in the sync system.

3. Ensure `OrientationChannel::Turning(target_pos)` is written when in range (already done by sync system) and during approach.

### Verification

- Select an infantry unit, press Attack, left-click an enemy → unit moves toward enemy and attacks.
- Right-click an enemy → same behavior.
- Unit transitions through Aiming → Firing → Cooldown → Reloading phases when in range.

## Technical Context

### File 1: `artifacts/developer/src/game/combat/systems/behaviors.rs` (PRIMARY)

**Current state** (lines 22-121): `attacking_object_behavior_system` queries `(Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand, &UnitBaseEnum, &Owner, Option<&DomainEnum>, &GridPosition, Option<&MoveTarget>, Option<&mut TurretCommandState>)`. It does NOT currently query `LocomotionChannel` or `OrientationChannel`.

**Changes needed**:
- Add `&mut LocomotionChannel` and `&mut OrientationChannel` to the query tuple
- Add the imports: `use crate::game::units::types::state::behavior::{LocomotionChannel, OrientationChannel};`
- **When IN range** (line 73 branch): Write `*locomotion = LocomotionChannel::Stationary;` and `*orientation = OrientationChannel::Turning(target_transform.translation);` for infantry (non-turret, non-glider). Currently it only does `commands.entity(entity).remove::<MoveTarget>().remove::<Path>()`.
- **When OUT of range** (line 95 branch): After computing the path (lines 103-118), also write `*locomotion = LocomotionChannel::Moving(smoothed.clone());` and `*orientation = OrientationChannel::Turning(target_transform.translation);`. The `smoothed` path from `smooth_path()` gives the Vec<Vec3> waypoints. Keep the `MoveTarget/Path` insert for now for compatibility, but the channel write is what will actually drive movement.

**Pattern to follow**: See `artifacts/developer/src/game/units/systems/behaviors/movement.rs` lines 65 and 103 — `moving_to_location_system` writes `*locomotion = LocomotionChannel::Moving(planned_path)` from waypoints. Also `artifacts/developer/src/game/units/systems/behaviors/construction.rs` line 87 — `*locomotion = LocomotionChannel::Moving(vec![target])` for single-target approach.

### File 2: `artifacts/developer/src/game/combat/systems/core.rs` (PRIMARY)

**Current state** (lines 575-631): `attack_channel_sync_system` queries `(&AttackState, &mut BaseAttackChannel, &mut LocomotionChannel, &mut OrientationChannel)` with `Without<Turret>`. It also queries `targets: Query<&Transform, With<ObjectInstance>>` but does NOT query `AttackCapability` or unit `Transform` — so it cannot check range.

**Changes needed**:
- Expand the unit query to include `&Transform` and `&AttackCapability` (and `Option<&DomainEnum>`, `&GridPosition` if you want elevation-adjusted range; simpler alternative: just use raw range from `AttackCapability.range` as a slightly conservative check)
- Add `ElevationMap` resource for elevation-adjusted range if desired (matches the pattern in `attack_phase_system` lines 79-87 and `attacking_object_behavior_system` lines 64-71)
- **In the `AttackPhase::Aiming` arm** (lines 585-601): Compute distance to target. If `distance <= effective_range`, force Stationary (existing behavior). If `distance > effective_range`, set `BaseAttackChannel` but do NOT overwrite `LocomotionChannel` — leave whatever the behavior system wrote.
- **In the `AttackPhase::Firing` arm** (lines 603-613): Keep forcing Stationary — firing always stops movement.
- **In the `AttackPhase::Cooldown` arm** (lines 615-619): Keep forcing Stationary — cooldown always stops movement.
- Note: `attack_phase_system` (line 101-102) already resets `AttackPhase` to `None` when out of range during Aiming, so in practice the Aiming+out-of-range case in sync may be brief, but handling it is still needed to prevent the one-frame stall.

**Important**: The `targets` query on line 580 queries `&Transform` — you'll need this to compute distance. Make sure the unit query also has `&Transform`.

### File 3: `artifacts/developer/src/game/combat/mod.rs` (SYSTEM ORDERING)

**Current ordering** (lines 26-45): Systems are listed in a flat tuple with no explicit ordering constraints between them within the set. Bevy does not guarantee execution order within a flat tuple. The comment on line 27-28 says behavior systems run "before" attack_phase_system, but this is NOT enforced by Bevy.

**Recommended**: The current system list order already has `attacking_object_behavior_system` (line 29) before `attack_channel_sync_system` (line 39), but for correctness, consider adding `.before()` / `.after()` constraints. However, this is a known existing issue — for this task, the flat tuple ordering appears to work in practice. Do NOT change ordering unless testing reveals issues.

### Key Types Reference

- `LocomotionChannel` — enum: `Moving(Vec<Vec3>)`, `Reversing(Vec<Vec3>)`, `Stopping`, `Stationary` (default). Component on all units. Defined in `artifacts/developer/src/game/units/types/state/behavior.rs` lines 58-69.
- `OrientationChannel` — enum: `Turning(Vec3)`, `Maintaining` (default). Component on all units. Same file lines 73-80.
- `BaseAttackChannel` — enum: `Aiming(Entity)`, `Firing(Entity)`, `Cooldown`, `Reloading`, `None` (default). Same file lines 84-97.
- `AttackState` — struct: `phase: AttackPhase`, `time_in_phase: f32`, `current_target: Option<AttackTarget>`. File: `artifacts/developer/src/game/combat/types.rs` line 138.
- `AttackPhase` — enum: `None`, `Aiming`, `Firing`, `Cooldown`, `Reloading`. `is_interruptible()` returns true for None/Aiming/Reloading. Same file line 95.
- `AttackCapability` — struct: `range: f32`, `damage`, `aim_time`, etc. Same file line 36. `is_melee()` method at line 70.
- `MoveTarget(Vec3)` and `Path { waypoints, current_waypoint }` — legacy movement components still used by `attacking_object_behavior_system`. Import from unit types.

### Existing Tests

- `attack_channel_sync_system` tests at `core.rs` line 1408+ — unit tests that simulate sync logic inline (not full ECS tests). These will need updating to account for the new range-check behavior. At minimum, add tests for:
  - Aiming + in-range → Stationary (existing, keep)
  - Aiming + out-of-range → locomotion NOT overwritten
- `attacking_object_behavior_system` tests at `behaviors.rs` line 754+ — verify that channel writes are tested for both in-range and out-of-range scenarios.

## Dependencies

- **Action channels infrastructure** (already implemented): `LocomotionChannel`, `OrientationChannel`, and `BaseAttackChannel` components are already defined and present on units. The channel-reading movement systems that consume `LocomotionChannel::Moving` are already in place.
- **`attack_phase_system`** (existing, no changes needed): Handles phase transitions (None→Aiming→Firing→Cooldown→Reloading). Already checks distance and resets to None when out of range (line 101-102). This system cooperates with the fix — when out of range, it keeps phase at None, so `attack_channel_sync_system` won't interfere.
