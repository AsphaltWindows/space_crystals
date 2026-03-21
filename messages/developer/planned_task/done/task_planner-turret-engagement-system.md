# turret-engagement-system

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-turret-behavior-system.md

## Task

Implement a new `turret_engagement_system` that reads `TurretCommandState.locked_target` and drives turret rotation and firing via the turret action channels.

**System behavior:**
1. For each turret unit with `TurretCommandState`:
   - If `locked_target = None`: set `TurretAttackChannel::Inactive` and `TurretOrientationChannel::Maintaining`
   - If `locked_target = Some(target)`:
     a. Validate target still exists (if despawned, clear locked_target, set Inactive)
     b. Compute turret rotation needed toward target position
     c. Set `TurretOrientationChannel::Turning(target_position)`
     d. If turret is aligned with target (within tolerance): set `TurretAttackChannel::Aiming(target)` or `Firing(target)` based on attack phase
     e. If target moves out of weapon Range or TurnAngle arc: clear `locked_target` (allow re-scan)

2. Use the existing `compute_relative_turret_angle` helper and `Turret::can_reach_angle` for arc checks

3. The actual turret rotation (applying angular velocity) and attack execution (damage dealing) should integrate with or reuse existing turret rotation logic and the `attack_phase_system` — this system's job is to SET the channel values that those systems consume

**Components queried:** Transform, Turret, AttackCapability, TurretCommandState, TurretBehaviorState, TurretOrientationChannel, TurretAttackChannel, Owner, GridPosition

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/combat/systems/core.rs`** — add the new `turret_engagement_system` function. This file already contains `compute_relative_turret_angle` (line 244) and `turret_autonomous_scanning_system` (line 265), both of which this system works alongside.

### System registration
- **`artifacts/developer/src/game/combat/mod.rs`** — register `turret_engagement_system` in `CombatPlugin::build()` (line 26-41), within the `DiagCategory::Combat` system set. Add it after `turret_autonomous_scanning_system` (line 36) since it consumes the `locked_target` that scanning sets.
- In `systems/mod.rs` (`artifacts/developer/src/game/combat/systems/mod.rs`), the `pub use core::*;` re-export on line 7 will automatically make the new system visible — no additional export needed.

### Import additions needed in core.rs
- **Line 5** currently: `use crate::game::units::types::{UnitCommand, UnitControlCost};`
- Add `TurretCommandState` to this import (may already be added by the sibling scanning-rework task): `use crate::game::units::types::{UnitCommand, UnitControlCost, TurretCommandState};`
- Add channel types from behavior.rs: `use crate::game::units::types::state::behavior::{TurretOrientationChannel, TurretAttackChannel};`
  - These are defined in `artifacts/developer/src/game/units/types/state/behavior.rs` lines 104-126

### Key types and their definitions

**TurretCommandState** (`artifacts/developer/src/game/units/types/state/commands.rs` line 209):
```rust
pub struct TurretCommandState {
    pub locked_target: Option<Entity>,
}
```

**TurretOrientationChannel** (behavior.rs line 104):
```rust
pub enum TurretOrientationChannel {
    Turning(Vec3),       // target position to face
    #[default] Maintaining,  // hold current turret facing
}
```

**TurretAttackChannel** (behavior.rs line 115):
```rust
pub enum TurretAttackChannel {
    Aiming(Entity),   // aiming at target entity
    Firing(Entity),   // firing at target entity
    Cooldown,         // post-fire cooldown
    Reloading,        // reload cycle
    #[default] Inactive,  // turret not attacking
}
```

**AttackState** (`artifacts/developer/src/game/combat/types.rs` line 137):
```rust
pub struct AttackState {
    pub phase: AttackPhase,         // None, Aiming, Firing, Cooldown, Reloading
    pub time_in_phase: f32,
    pub current_target: Option<AttackTarget>,
}
```
- `attack_state.target_entity()` returns `Option<Entity>` — use to check for `UnitTarget` variant
- `AttackPhase` has `Aiming`, `Firing`, `Cooldown`, `Reloading`, `None`

**Turret** (`artifacts/developer/src/game/combat/types.rs` line 237):
```rust
pub struct Turret {
    pub turn_angle: f32,     // total arc (radians), full rotation = TAU
    pub turn_rate: f32,      // radians per second
    pub current_angle: f32,  // current turret angle relative to body
    pub target_angle: Option<f32>,  // target angle for rotation system
}
```
- `can_reach_angle(angle: f32) -> bool` — checks if angle is within turn_angle/2
- `clamp_angle(angle: f32) -> f32` — clamps to arc limits

**compute_relative_turret_angle** (core.rs line 244):
```rust
pub fn compute_relative_turret_angle(unit_transform: &Transform, target_pos: Vec3) -> f32
```
Returns angle in radians [-PI, PI] relative to unit's forward direction.

**AttackCapability** (types.rs line 36):
- `range: f32` — weapon range in grid units
- `min_range: f32` — minimum range

### Recommended query signature
```rust
pub fn turret_engagement_system(
    mut turret_units: Query<
        (&Transform, &Turret, &AttackCapability, &mut TurretCommandState,
         &mut TurretOrientationChannel, &mut TurretAttackChannel,
         &AttackState, &Owner, &GridPosition),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    elevation_map: Res<ElevationMap>,
)
```
- Follow the pattern from `turret_autonomous_scanning_system` (line 265) which uses a similar query structure
- Include `&AttackState` (read-only) to map `attack_state.phase` to the appropriate channel value (Aiming vs Firing vs Cooldown vs Reloading)
- Include `ElevationMap` + `GridPosition` + `DomainEnum` for elevation-adjusted range checks, matching the pattern used in `turret_autonomous_scanning_system` (lines 279-302) and `attack_phase_system` (lines 74-81)

### System logic pattern

**No target case**: When `locked_target` is `None`, set channels to idle:
```rust
*turret_orientation_channel = TurretOrientationChannel::Maintaining;
*turret_attack_channel = TurretAttackChannel::Inactive;
```

**Target validation**: Check target still exists in the targets query. If `targets.get(target_entity).is_err()`, clear `locked_target` and set channels to idle. Follow same pattern as `attack_phase_system` lines 57-61.

**Range check**: Compute distance and elevation-adjusted effective range. If target is out of range, clear `locked_target` (so autonomous scanning can re-acquire). Follow the elevation pattern from `turret_autonomous_scanning_system` lines 296-306.

**Arc check**: Use `compute_relative_turret_angle(transform, target_pos)` + `turret.can_reach_angle(relative_angle)`. If target has moved outside the turret's arc, clear `locked_target`.

**Orientation channel**: When target is valid and in range/arc:
```rust
*turret_orientation_channel = TurretOrientationChannel::Turning(target_transform.translation);
```

**Attack channel mapping from AttackState.phase**: The `attack_phase_system` already drives `AttackState` through its phases. This system maps the current phase to channel values:
```rust
match attack_state.phase {
    AttackPhase::Aiming => *turret_attack_channel = TurretAttackChannel::Aiming(target_entity),
    AttackPhase::Firing => *turret_attack_channel = TurretAttackChannel::Firing(target_entity),
    AttackPhase::Cooldown => *turret_attack_channel = TurretAttackChannel::Cooldown,
    AttackPhase::Reloading => *turret_attack_channel = TurretAttackChannel::Reloading,
    AttackPhase::None => *turret_attack_channel = TurretAttackChannel::Aiming(target_entity),
}
```

**Alignment tolerance**: For determining if the turret is "aligned" with the target, check `(turret.current_angle - relative_angle).abs() < tolerance`. The existing `turret_rotation_system` (turret.rs line 66) uses `0.01` radians as its convergence threshold — use the same or a slightly larger value (e.g., `0.05` radians ≈ 3 degrees).

### Integration with existing turret systems

The existing turret pipeline in `TurretPlugin` (mod.rs lines 46-56):
1. `turret_aiming_system` — reads `AttackState` to set `Turret.target_angle`. Currently the ONLY system that sets `target_angle`. This system reads `attack_state.phase` and only sets target_angle during Aiming/Reloading phases.
2. `turret_rotation_system` — reads `Turret.target_angle`, applies angular velocity to `Turret.current_angle`
3. `update_turret_visual_system` — reads `Turret.current_angle`, sets child visual rotation

**Important**: `turret_aiming_system` (turret.rs line 6) currently reads `AttackState` directly and sets `Turret.target_angle`. The new engagement system writes to `TurretOrientationChannel` instead. For the channel to take effect, eventually `turret_aiming_system` should read from `TurretOrientationChannel` — but that's a future integration task. For now, this system should ALSO set `Turret.target_angle` directly (add `&mut Turret` to the query) to ensure turret rotation works immediately, since no system currently reads `TurretOrientationChannel`.

Update the query to include `&mut Turret` and add:
```rust
let relative_angle = compute_relative_turret_angle(transform, target_transform.translation);
let clamped = turret.clamp_angle(relative_angle);
turret.target_angle = Some(clamped);
```
This mirrors what `turret_aiming_system` does (turret.rs lines 29-48) but driven by `TurretCommandState` instead of `AttackState`.

### Existing tests reference
- `compute_relative_turret_angle` tests at core.rs lines 631-675 — test angle computation
- `turret_autonomous_scanning_system` tests at core.rs lines 676-704 — tuple comparison tests for priority logic
- Add tests for the new system following Bevy App-based test patterns. Test: no target → channels idle; target valid + in range + in arc → channels set; target despawned → locked_target cleared; target out of range → locked_target cleared; target out of arc → locked_target cleared.

## Dependencies

- **turret-autonomous-scanning-rework** (sibling task): The scanning system writes `TurretCommandState.locked_target` which this system reads. Without the rework, `locked_target` will always be `None` (default). This system can be implemented independently — it will simply never engage until scanning starts populating `locked_target`. But for full functionality, the scanning rework should be completed first.
- **turret-base-behavior-target-relay** (sibling task): The relay system also writes to `locked_target` from base commands (e.g., explicit Attack command). Same situation — this system works regardless, just won't receive command-driven targets until relay is done.
- **attack_phase_system** (existing, core.rs line 38): This system reads `AttackState.phase` which `attack_phase_system` drives. The engagement system depends on `attack_phase_system` running to progress through attack phases. Both are in `DiagCategory::Combat` — no explicit ordering needed since the engagement system reads phase state that was set in a previous frame.
- **TurretPlugin systems** (existing, turret.rs): `turret_rotation_system` consumes `Turret.target_angle` which this system sets. Runs in `DiagCategory::Turrets` — separate from `DiagCategory::Combat`, no ordering conflict.
