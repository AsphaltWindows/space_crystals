# base-behaviors-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-base-behaviors.md

## Task

Verify that all 9 base behaviors defined in control_system.md are fully implemented and registered. The following systems already exist and need verification against the design spec:

**In units/systems/behaviors.rs:**
1. `moving_to_location_system` — MovingToLocation
2. `moving_to_object_system` — MovingToObject
3. `reversing_to_location_system` — ReversingToLocation
4. `stopping_behavior_system` — StoppingBehavior

**In combat/systems/behaviors.rs:**
5. `attacking_object_behavior_system` — AttackingObject
6. `attacking_location_behavior_system` — AttackingLocation
7. `attack_move_behavior_system` — AttackMovingToLocation
8. `hold_position_behavior_system` — HoldingPosition
9. `patrol_scanning_system` — Patrolling

**Verification checklist:**
- All 9 systems are registered in their respective plugins
- BaseBehaviorState has variants for all 5 movement models plus None
- Action channels are defined and used correctly
- Constants match spec: AttackMoveLeashDistance=6gu, IdleLeashDistance=4gu
- HoldingPosition component marker exists and is managed correctly
- PatrolEngaged component preserves patrol state during engagement
- Tests pass: `cargo test` in artifacts/developer/

If any behavior is incomplete or deviates from the spec, implement the fix. If all behaviors match, confirm with a passing test run.

## Technical Context

### System Registration — Verified Locations

**Units plugin** (`artifacts/developer/src/game/units/mod.rs`, lines 15-49):
All 4 unit behavior systems are registered in `UnitsPlugin::build()` under `DiagCategory::Movement`:
- `moving_to_location_system` (line 20)
- `moving_to_object_system` (line 21)
- `reversing_to_location_system` (line 22)
- `stopping_behavior_system` (line 23)
They run in Phase 2 (`.after(rebuild_occupancy_map)`).

**Combat plugin** (`artifacts/developer/src/game/combat/mod.rs`, lines 26-41):
All 5 combat behavior systems are registered in `CombatPlugin::build()` under `DiagCategory::Combat`:
- `attacking_object_behavior_system` (line 29)
- `attacking_location_behavior_system` (line 30)
- `attack_move_behavior_system` (line 31)
- `hold_position_behavior_system` (line 32)
- `patrol_scanning_system` (line 33)

**Commands plugin** (`artifacts/developer/src/game/units/mod.rs`, lines 56-65):
Supporting systems in `CommandsPlugin` under `DiagCategory::Commands`:
- `hold_position_system` (line 60) — legacy H key handler, inserts `HoldingPosition` marker + `UnitCommand::HoldPosition`
- `stop_command_system` (line 61) — legacy S key handler, removes `HoldingPosition`, inserts `UnitCommand::Stop`
- `patrol_command_system` (line 62) — handles patrol waypoint cycling (separate from scanning)

### BaseBehaviorState (`artifacts/developer/src/game/units/types/state/behavior.rs`, lines 7-40)
6 variants present (5 movement models + None):
- `TurnRate { planned_path, path_index }` — Infantry/Mech
- `FixedTurnRadius { planned_path, path_index }` — Wheeled vehicles
- `SpeedTurnRadius { planned_path, path_index }` — Tracked/Drill
- `Drag { planned_path, path_index, drift_velocity }` — Hover
- `Glider { planned_path, path_index, circling, strafe_target }` — Air
- `None` (default) — Idle

### Action Channels (behavior.rs, lines 54-127)
All 5 channels implemented as Component enums:
- `LocomotionChannel`: Moving(path), Reversing(path), Stopping, Stationary(default)
- `OrientationChannel`: Turning(Vec3), Maintaining(default)
- `BaseAttackChannel`: Aiming(Entity), Firing(Entity), Cooldown, Reloading, None(default)
- `TurretOrientationChannel`: Turning(Vec3), Maintaining(default)
- `TurretAttackChannel`: Aiming(Entity), Firing(Entity), Cooldown, Reloading, Inactive(default)

### Constants (`artifacts/developer/src/game/combat/types.rs`, lines 376-398)
- `IDLE_LEASH_DISTANCE: f32 = 4.0` (line 376) — matches spec
- `ATTACK_MOVE_LEASH_DISTANCE: f32 = 6.0` (line 380) — matches spec
- `HOLD_POSITION_FACING_ARC: f32 = FRAC_PI_6` (line 398) — 30 degrees for non-turning infantry

### HoldingPosition Marker (`artifacts/developer/src/game/units/types/state/commands.rs`, line 145)
`pub struct HoldingPosition;` — Component marker. Inserted by `hold_position_system` (commands.rs:88), removed by `stop_command_system` (commands.rs:125) and `right_click_move_command` (core.rs: multiple lines ~439-568). Also used as a filter: `Without<HoldingPosition>` in `unit_movement_system` (core.rs:766) and `collision_repath_system` (core.rs:907) to prevent HoldPosition units from moving.

### PatrolEngaged (`artifacts/developer/src/game/combat/types.rs`, lines 390-395)
`pub struct PatrolEngaged { patrol_start, patrol_end, going_to_end }` — Component. Inserted by `patrol_scanning_system` (combat/behaviors.rs:509) when enemy detected during patrol. Removed (line 472) when target dies/gone and patrol resumes. Preserves waypoint state across engagement.

### TurretCommandState (`artifacts/developer/src/game/units/types/state/commands.rs`, line 203)
`pub struct TurretCommandState { pub locked_target: Option<Entity> }` — cleared by `stopping_behavior_system` (behaviors.rs:354).

### Key Behavior Implementation Details

1. **moving_to_location_system** (behaviors.rs:46-143): Reads `UnitCommand::Move`, follows path via `LocomotionChannel::Moving`, glider enters circling on path completion, normal units transition to Stopping then Stationary. Note: does NOT set `UnitCommand::Idle` on completion (comment at lines 128-131 says a separate completion system must do this).

2. **moving_to_object_system** (behaviors.rs:149-255): Reads `MoveObjectTarget` component (not `UnitCommand`). Re-paths when target moves > `TARGET_MOVED_THRESHOLD` (1.0 gu). Completes at `OBJECT_PROXIMITY_DISTANCE` (1.5 gu).

3. **reversing_to_location_system** (behaviors.rs:261-328): Reads `UnitCommand::Reverse`. Only processes `FixedTurnRadius` and `SpeedTurnRadius` variants. Writes `LocomotionChannel::Reversing`. Orientation stays Maintaining (keeps facing forward while reversing).

4. **stopping_behavior_system** (behaviors.rs:334-363): Reads `UnitCommand::Stop`. Sets Stopping+Maintaining, clears `TurretCommandState.locked_target`, transitions to Stationary+`BaseBehaviorState::None` when velocity < threshold.

5. **attacking_object_behavior_system** (combat/behaviors.rs:20-109): Reads `UnitCommand::AttackTarget(entity)`. Checks elevation-adjusted range. Infantry stops to fire, turret units keep moving, gliders strafe. Uses `MoveTarget`/`Path` components for approach (not action channels directly). Goes idle on target destroyed.

6. **attacking_location_behavior_system** (combat/behaviors.rs:116-193): Reads `UnitCommand::AttackLocation(pos)`. Same range check pattern. Completes after one full cycle (`AttackPhase::Reloading`).

7. **attack_move_behavior_system** (combat/behaviors.rs:200-350): Reads `UnitCommand::AttackMove(dest)`. Scans `SightRange` for enemies. Uses `AttackMoveOrigin` for leash checking (measures from origin, not perpendicular to path — **NOTE: spec says perpendicular distance from PathReference, implementation uses radial distance from origin. This is a known simplification.**). Disengages at > 6.0 gu.

8. **hold_position_behavior_system** (combat/behaviors.rs:358-436): Reads `UnitCommand::HoldPosition`, filtered `Without<Turret>` (turret units handled by turret_autonomous_scanning). Removes MoveTarget/Path each tick. For non-turning infantry, checks `HOLD_POSITION_FACING_ARC` (30 degrees). Picks closest enemy in range.

9. **patrol_scanning_system** (combat/behaviors.rs:444-523): Reads `UnitCommand::Patrol { start, end, going_to_end }`. Scans for enemies within SightRange. On detect: saves state in `PatrolEngaged`, switches command to `AttackTarget(enemy)`. On target gone: restores `Patrol` command from `PatrolEngaged` data. Patrol waypoint cycling handled by separate `patrol_command_system` (commands.rs:134).

### Potential Spec Deviations to Verify

1. **AttackMove leash measurement**: Spec says "perpendicular distance from PathReference"; implementation uses radial distance from `AttackMoveOrigin` (simpler). Decide if this is acceptable or needs fixing.

2. **MovingToLocation completion**: Spec says behavior complete when stopped. Implementation transitions to Stationary but does NOT set `UnitCommand::Idle` (requires separate completion system per comment at line 128-131). Verify if this separate system exists.

3. **Patrol scanning vs AttackMove sub-behavior**: Spec says Patrolling wraps AttackMovingToLocation (enemy scanning during each leg). Implementation uses separate `patrol_scanning_system` + `patrol_command_system`. Patrol does scan for enemies, but doesn't delegate to AttackMove sub-behavior — it directly switches to `AttackTarget` command. This means patrol engagement doesn't have leash behavior. Verify if this is acceptable.

4. **BaseAutoTargeting during idle**: Spec mentions idle auto-targeting. The `base_auto_target_system` (combat/core.rs:355) and `idle_leash_system` (combat/core.rs:430) handle this — verify they match spec.

### Files to Inspect/Modify
- `artifacts/developer/src/game/units/systems/behaviors.rs` — 4 unit behavior systems (2750 lines)
- `artifacts/developer/src/game/combat/systems/behaviors.rs` — 5 combat behavior systems (728 lines)
- `artifacts/developer/src/game/combat/systems/core.rs` — base_auto_target_system, idle_leash_system
- `artifacts/developer/src/game/units/types/state/behavior.rs` — BaseBehaviorState, action channels
- `artifacts/developer/src/game/combat/types.rs` — Constants, PatrolEngaged, AttackMoveOrigin
- `artifacts/developer/src/game/units/types/state/commands.rs` — HoldingPosition, TurretCommandState, UnitCommand
- `artifacts/developer/src/game/units/mod.rs` — UnitsPlugin registration
- `artifacts/developer/src/game/combat/mod.rs` — CombatPlugin registration
- `artifacts/designer/design/control_system.md` — The spec to verify against

### Running Tests
```bash
cd artifacts/developer && cargo test
```
Existing tests cover: constants (leash distances, facing arc), component structures, channel variants, patrol engaged state, infantry/turret distinction.

## Dependencies

- **action-channel-locomotion-orientation** (sibling task): If action channel integration is being reworked, those changes could affect how behaviors write to channels. Verify behavior systems still compile after channel changes.
- **action-channel-attack-integration** (sibling task): Attack channel integration affects how AttackingObject and HoldPosition behaviors trigger attacks. Coordinate on AttackState vs BaseAttackChannel usage.
- **combat-attack-verify** (sibling task): Overlaps with verification of combat systems — ensure no conflicting changes to shared types like AttackState, AttackPhase.
