# turret-base-behavior-target-relay

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-turret-behavior-system.md

## Task

Make base behaviors relay their targets to `TurretCommandState.locked_target` on turret units, connecting the command/behavior pipeline to turret engagement.

**Per design doc (control_system.md):**
- When base behavior has a target (e.g., AttackingObject, AttackingLocation): set `TurretCommandState.locked_target = Some(target)` for turret units
- When base behavior does NOT specify a target: turret retains previous TurretCommandState (do NOT clear it — let autonomous scanning manage it)
- Specific behaviors:
  - **AttackingObject**: set locked_target = attack target entity
  - **AttackingLocation/AttackGround**: locked_target is NOT set (turret fires at location, not a unit — per design line 443)
  - **AttackMovingToLocation**: do NOT set locked_target (turret autonomous scanning operates independently)
  - **MovingToLocation/Move**: do NOT set locked_target (turret autonomous scanning operates independently via TurretBehavior)
  - **HoldingPosition**: for turret units, base auto-targeting sets locked_target (already handled by the scanning system)
  - **Idle**: base auto-targeting sets locked_target (handled by scanning system)
  - **StoppingBehavior**: clear locked_target (turret falls back to autonomous scanning — per design line 535)

**Implementation approach:**
The existing `moving_to_location_behavior_system` in `game/units/systems/behaviors.rs` already queries `Option<&mut TurretCommandState>` (line 341). Extend this pattern to the combat behavior systems in `game/combat/systems/behaviors.rs` — specifically `attacking_object_behavior_system` and `stopping_behavior_system` — to set/clear `TurretCommandState.locked_target` as appropriate.

**Key files:**
- `game/combat/systems/behaviors.rs` — attacking_object_behavior_system, stopping_behavior_system
- `game/units/systems/behaviors.rs` — moving_to_location_behavior_system (reference pattern)
- `game/units/types/state/commands.rs` — TurretCommandState
