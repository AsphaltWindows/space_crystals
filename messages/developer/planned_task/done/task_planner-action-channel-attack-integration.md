# action-channel-attack-integration

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-action-channels.md

## Task

Integrate BaseAttackChannel and TurretAttackChannel/TurretOrientationChannel with the combat systems. Currently the combat module uses its own `AttackState`/`AttackPhase` enum (in `game/combat/types.rs`) and does NOT write to the action channel components. The turret systems similarly don't use `TurretAttackChannel` or `TurretOrientationChannel`.

**What to implement:**

1. **BaseAttackChannel integration**: The attack phase system (`attack_phase_system` in `game/combat/systems/core.rs`) should write to `BaseAttackChannel` as it transitions through phases:
   - `AttackPhase::Aiming` → `BaseAttackChannel::Aiming(target_entity)`
   - `AttackPhase::Firing` → `BaseAttackChannel::Firing(target_entity)`
   - `AttackPhase::Cooldown` → `BaseAttackChannel::Cooldown`
   - `AttackPhase::Reloading` → `BaseAttackChannel::Reloading`
   - `AttackPhase::None` → `BaseAttackChannel::None`
   - For turret units: `BaseAttackChannel` must always remain `None`

2. **Cross-channel constraint enforcement**: When `BaseAttackChannel` is active (non-turret units):
   - `Aiming`: Override `OrientationChannel` to `Turning(target_pos)`, enforce `LocomotionChannel::Stationary`
   - `Firing`/`Cooldown`: Enforce `LocomotionChannel::Stationary`, lock `OrientationChannel::Maintaining`
   - `Reloading`: No constraints — locomotion and orientation free
   This replaces the current `base_action_constraints()` pattern in the movement systems.

3. **TurretAttackChannel integration**: The turret attack/scanning systems should write to `TurretAttackChannel`:
   - Turret aiming → `TurretAttackChannel::Aiming(target_entity)`
   - Turret firing → `TurretAttackChannel::Firing(target_entity)`
   - Turret cooldown → `TurretAttackChannel::Cooldown`
   - Turret reloading → `TurretAttackChannel::Reloading`
   - No target → `TurretAttackChannel::Inactive`

4. **TurretOrientationChannel integration**: The turret rotation system (`turret_rotation_system` in `game/combat/turret.rs`) should read from `TurretOrientationChannel`:
   - `TurretTurning(target_pos)`: rotate turret toward target, constrained by `TurnAngle`
   - `TurretMaintaining`: hold current turret facing
   - When `TurretAttackChannel::Aiming(target)`, override `TurretOrientationChannel` to face the target

5. **Interruptibility**: When a non-turret unit receives a new command during `BaseAttackChannel::Aiming` or `Reloading`, the attack sequence cancels (channel resets to None). During `Firing` or `Cooldown`, commands are queued/rejected.

**Design reference:** `artifacts/designer/design/control_system.md` — BaseAttackChannel, TurretActionChannels sections.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/combat/systems/core.rs`** — Primary file. Contains:
   - `attack_phase_system` (line 38): The main attack phase progression system. Currently transitions `AttackState.phase` through None→Aiming→Firing→Cooldown→Reloading cycle. Must be extended to write the corresponding `BaseAttackChannel` value in sync with each phase transition. Key: only write to `BaseAttackChannel` for non-turret units (use `Without<Turret>` filter or check `has_turret` via `UnitBaseEnum`). The query (line 42) currently has `(Entity, &Transform, &AttackCapability, &mut AttackState, &Owner, &UnitCommand, Option<&DomainEnum>, &GridPosition)` — you'll need to add `&mut BaseAttackChannel` (optional for turret units) and `Option<&Turret>` to distinguish.
   - `attack_command_system` (line 16): Currently sets `AttackState.current_target` and resets phase. Should also reset `BaseAttackChannel` to `None` when a new target is accepted on non-turret units.
   - `turret_autonomous_scanning_system` (line 265): Sets `AttackState` for turret units when they find targets. Must also write to `TurretAttackChannel::Aiming(target)` when a target is acquired.
   - `base_auto_target_system` (line 355): Sets `AttackState` for non-turret idle units. Must also write to `BaseAttackChannel::Aiming(target)` when a target is acquired.

2. **`artifacts/developer/src/game/combat/turret.rs`** — Contains 3 turret systems:
   - `turret_aiming_system` (line 6): Currently reads `AttackState` to set `Turret.target_angle`. Must be updated to also write `TurretOrientationChannel::Turning(target_pos)` when aiming, or `TurretOrientationChannel::Maintaining` otherwise. The query needs `&mut TurretOrientationChannel` added.
   - `turret_rotation_system` (line 53): Currently reads `Turret.target_angle` directly. Should be extended to also read `TurretOrientationChannel` — when `TurretOrientationChannel::Turning(pos)`, compute the target angle from position. This integrates the channel-driven pattern. Alternatively, keep `turret_aiming_system` as the channel-to-angle translator and leave this system reading `Turret.target_angle`.
   - `update_turret_visual_system` (line 77): No changes needed.

3. **`artifacts/developer/src/game/combat/types.rs`** — Contains `AttackPhase::base_action_constraints()` (line 120). This method already encodes the constraint rules per phase. The new cross-channel constraint system should use this as a reference or call it directly. No modification needed here unless you want to add a helper that maps `AttackPhase` directly to channel values.

4. **`artifacts/developer/src/game/combat/mod.rs`** — Plugin registration. Register new constraint enforcement system in `CombatPlugin::build()` (line 26). The cross-channel constraint system should run AFTER `attack_phase_system` but BEFORE the locomotion/orientation consumer systems (from the sibling task). Add to `DiagCategory::Combat` set.

5. **`artifacts/developer/src/game/units/types/state/behavior.rs`** — Channel type definitions. All types are already defined (lines 58-127): `BaseAttackChannel`, `TurretOrientationChannel`, `TurretAttackChannel`. No modifications needed.

### Key Types and Components

- **`AttackPhase`** (combat/types.rs:94): `None`, `Aiming`, `Firing`, `Cooldown`, `Reloading` — mirrors `BaseAttackChannel` variants exactly
- **`AttackState`** (combat/types.rs:137): Component with `phase: AttackPhase`, `time_in_phase: f32`, `current_target: Option<AttackTarget>`. `target_entity()` returns `Option<Entity>`
- **`AttackPhase::is_interruptible()`** (combat/types.rs:114): Returns true for `None`, `Aiming`, `Reloading`; false for `Firing`, `Cooldown`. Use this for interruptibility checks
- **`AttackPhase::base_action_constraints(is_turret_source)`** (combat/types.rs:120): Returns `PhaseActionConstraints { base_can_move, base_can_turn }`. Reference for constraint rules. For non-turret UnitBase: Aiming blocks move, allows turn; Firing/Cooldown blocks both; Reloading allows both
- **`Turret`** component (combat/types.rs:236): `turn_angle`, `turn_rate`, `current_angle`, `target_angle`. Present only on turret-bearing units. Use `With<Turret>`/`Without<Turret>` as a filter to distinguish turret from non-turret units
- **`TurretCommandState`** (units/types/state/commands.rs:209): `locked_target: Option<Entity>`. Used by base behaviors to lock turret onto a specific target (e.g., Attack command). When `None`, turret falls back to autonomous scanning
- **`BaseAttackChannel`** (behavior.rs:85): `Aiming(Entity)`, `Firing(Entity)`, `Cooldown`, `Reloading`, `None`. Default: `None`
- **`TurretAttackChannel`** (behavior.rs:115): `Aiming(Entity)`, `Firing(Entity)`, `Cooldown`, `Reloading`, `Inactive`. Default: `Inactive`
- **`TurretOrientationChannel`** (behavior.rs:104): `Turning(Vec3)`, `Maintaining`. Default: `Maintaining`
- **`LocomotionChannel`** (behavior.rs:59): `Moving(Vec<Vec3>)`, `Reversing(Vec<Vec3>)`, `Stopping`, `Stationary`. Used for cross-channel constraint enforcement
- **`OrientationChannel`** (behavior.rs:74): `Turning(Vec3)`, `Maintaining`. Used for cross-channel constraint enforcement

### Spawning Pattern

Units already spawn with the correct channel components (see `game/units/systems/core.rs` lines 99-128 and `game/utils.rs` lines 474-477):
- Non-turret units (LightInfantry, HeavyInfantry): get `BaseAttackChannel::default()` + `LocomotionChannel::default()` + `OrientationChannel::default()`
- Turret units (WheeledVehicle, TrackedVehicle, etc.): get `TurretCommandState::default()` + `TurretBehaviorState::default()` + `TurretOrientationChannel::default()` + `TurretAttackChannel::default()` — but NO `BaseAttackChannel`

### Imports Needed

In `combat/systems/core.rs`, add:
```rust
use crate::game::units::types::state::behavior::{BaseAttackChannel, LocomotionChannel, OrientationChannel};
```

In `combat/turret.rs`, add:
```rust
use crate::game::units::types::state::behavior::{TurretAttackChannel, TurretOrientationChannel};
```

### Implementation Strategy

**Approach A (Recommended): Sync system pattern**
Create a dedicated `attack_channel_sync_system` that runs after `attack_phase_system`. This system reads `AttackState` and writes the corresponding channel values. Separates concerns cleanly:
- For non-turret units (`Without<Turret>`): map `AttackState.phase` → `BaseAttackChannel`, apply `LocomotionChannel`/`OrientationChannel` constraints
- For turret units (`With<Turret>`): map `AttackState.phase` → `TurretAttackChannel`, compute target position and write `TurretOrientationChannel`

Query for non-turret sync:
```rust
Query<(&AttackState, &mut BaseAttackChannel, &mut LocomotionChannel, &mut OrientationChannel), (With<Unit>, Without<Turret>)>
```

Query for turret sync:
```rust
Query<(&AttackState, &mut TurretAttackChannel, &mut TurretOrientationChannel), (With<Unit>, With<Turret>)>
```

To get the target position for orientation, you need a secondary query:
```rust
Query<&Transform, With<ObjectInstance>>
```

**Approach B: Inline into existing systems**
Directly modify `attack_phase_system`, `turret_autonomous_scanning_system`, etc. to write channels at each transition point. More intrusive but fewer systems.

### System Ordering

```
attack_command_system (sets targets)
  → attack_phase_system (progresses phases)
  → turret_autonomous_scanning_system (acquires turret targets)
  → base_auto_target_system (acquires base targets)
  → [NEW] attack_channel_sync_system (syncs phases to channels, enforces constraints)
  → [SIBLING TASK] locomotion/orientation consumer systems (reads channels, drives movement)
```

All in `DiagCategory::Combat`. The new sync system should be registered in `CombatPlugin::build()` after the existing systems.

### Existing Tests

- `attack_phase_system` has tests in `combat/systems/core.rs` (line ~450+) — extend with channel state assertions
- `AttackPhase::base_action_constraints()` has comprehensive tests in `combat/types.rs` (lines 401-545) — use as specification for constraint enforcement
- `turret_aiming_system` and `turret_rotation_system` have no dedicated tests — add channel integration tests
- Channel type tests exist in `behavior.rs` (lines 489-591) — verify all variant combinations

### Test Strategy

Test the sync system independently:
1. Non-turret unit: set `AttackState.phase = Aiming`, run sync, assert `BaseAttackChannel::Aiming(target)` + `LocomotionChannel::Stationary` + `OrientationChannel::Turning(target_pos)`
2. Non-turret unit: set phase = Firing, assert `LocomotionChannel::Stationary` + `OrientationChannel::Maintaining`
3. Non-turret unit: set phase = Reloading, assert channels unconstrained (whatever behavior wrote stays)
4. Turret unit: set phase = Aiming, assert `TurretAttackChannel::Aiming(target)` + `TurretOrientationChannel::Turning(target_pos)`
5. Turret unit with no target: assert `TurretAttackChannel::Inactive`
6. Interruptibility: change `UnitCommand` during Aiming, verify channel resets; change during Firing, verify no reset

## Dependencies

- **`action-channel-locomotion-orientation`** (sibling planned_task, already sent to developer): This task creates the locomotion/orientation consumer systems that READ from `LocomotionChannel` and `OrientationChannel`. The attack integration task WRITES to those same channels for constraint enforcement (e.g., forcing `LocomotionChannel::Stationary` during Aiming). The channel types already exist so the code can be written independently, but the cross-channel constraints will only have observable movement effects once the locomotion consumer is implemented.
- **`attack_phase_system`** (existing, `game/combat/systems/core.rs`): The core system this task extends. Must not break existing attack phase progression logic.
- **`turret_aiming_system`** / **`turret_rotation_system`** (existing, `game/combat/turret.rs`): The existing turret systems that must be adapted to use channels. Currently functional without channels.
