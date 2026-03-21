# turret-base-behavior-target-relay

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/combat/systems/behaviors.rs`** — this file contains the combat behavior systems that need TurretCommandState integration

### Systems to modify

**1. `attacking_object_behavior_system` (line 20-109)**
This system handles `UnitCommand::AttackTarget(entity)`. It needs to set `TurretCommandState.locked_target = Some(target_entity)` for turret units when the target is in range and engagement begins.

Current query (line 22-26):
```rust
(Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand,
 &UnitBaseEnum, &Owner, Option<&DomainEnum>, &GridPosition, Option<&MoveTarget>),
With<Unit>
```
**Add `Option<&mut TurretCommandState>`** to the query tuple. Then, after line 70 where `attack_state.current_target` is set (in-range engagement), add:
```rust
if let Some(mut turret) = turret_state {
    turret.locked_target = Some(target_entity);
}
```
Also when target is destroyed (line 44-50, idle transition), clear locked_target:
```rust
if let Some(mut turret) = turret_state {
    turret.locked_target = None;
}
```

**2. Systems that do NOT need modification (by design):**
- `attacking_location_behavior_system` (line 116): AttackGround targets a location not an entity — per design line 443, TurretCommandState.locked_target is NOT set
- `attack_move_behavior_system` (line 200): TurretAutonomousScanning operates independently during attack-move — do NOT set locked_target
- `hold_position_behavior_system` (line 358): For turret units, this system already has `Without<Turret>` filter (line 363) — turret hold-position targeting is handled by the autonomous scanning system
- `patrol_scanning_system` (line 444): Switches to `UnitCommand::AttackTarget` on engagement — the attacking_object_behavior_system will then relay the target

**3. `stopping_behavior_system` in `artifacts/developer/src/game/units/systems/behaviors.rs` (line 334-363)**
**Already implemented correctly** — it queries `Option<&mut TurretCommandState>` (line 341) and clears `locked_target` on line 353-355. No changes needed here.

### Import requirements
The combat behaviors file (`game/combat/systems/behaviors.rs`) imports `use crate::game::units::types::*;` on line 5. `TurretCommandState` is re-exported through `game::units::types::state::commands` → `game::units::types::state` → `game::units::types`, so it's **already accessible** — no new import needed.

### Key types

**TurretCommandState** (`game/units/types/state/commands.rs` line 208-213):
```rust
pub struct TurretCommandState {
    pub locked_target: Option<Entity>,
}
```
- Only present on units whose UnitBase has_turret = true (spawned in `game/units/systems/core.rs` line 120)
- Use `Option<&mut TurretCommandState>` in queries to handle both turret and non-turret units

**UnitBaseEnum.data().has_turret** — indicates turret units:
- WheeledVehicle: has_turret = true
- TrackedVehicle: has_turret = true
- LightInfantry/HeavyInfantry: has_turret = false
- Glider: has_turret = false

### Reference pattern
The `stopping_behavior_system` in `game/units/systems/behaviors.rs` (line 334-363) is the canonical example of how to query and mutate `TurretCommandState`:
```rust
pub fn stopping_behavior_system(
    mut query: Query<(
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut BaseBehaviorState,
        &UnitCommand,
        &Velocity,
        Option<&mut TurretCommandState>,  // <-- this pattern
    )>,
) {
    // ...
    if let Some(mut turret) = turret_state {
        turret.locked_target = None;        // <-- clear pattern
    }
}
```

### Testing approach
Add unit tests in the existing `#[cfg(test)] mod tests` at line 526 of combat/systems/behaviors.rs:
- Test that attacking_object_behavior_system sets locked_target when target is in range (spawn turret unit with TurretCommandState + AttackTarget command + target within range)
- Test that locked_target is cleared when target entity is despawned
- Test that non-turret units (without TurretCommandState) still work correctly (the Option query handles this)
- Follow the existing test patterns in that file (data-driven assertions, no App-based tests needed for pure logic checks)

### System ordering
All combat behavior systems run in `DiagCategory::Combat` (registered in `game/combat/mod.rs` lines 26-41). No explicit ordering is needed between them — the `attacking_object_behavior_system` sets `locked_target`, and the sibling `turret_engagement_system` (separate task) reads it on the next frame.

## Dependencies

- **turret-engagement-system** (sibling planned_task): That system reads `TurretCommandState.locked_target` to drive turret rotation and firing channels. This relay task populates `locked_target` from base commands. Both can be implemented independently — without the engagement system, setting locked_target has no effect yet; without this relay, the engagement system only receives targets from autonomous scanning.
- **turret_autonomous_scanning_system** (existing, `game/combat/systems/core.rs` line 265): Already queries `&mut TurretCommandState` and sets `locked_target` during autonomous scanning. This relay task handles the command-driven path (explicit Attack orders). Both systems write to the same field — no conflict because attacking_object takes priority (when unit has AttackTarget command, scanning doesn't run for that unit since attack_state already has a target).
- **stopping_behavior_system** (existing, already implemented): Clears `locked_target` on Stop command. No changes needed — already correct.
