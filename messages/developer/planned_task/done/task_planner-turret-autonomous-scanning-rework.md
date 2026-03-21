# turret-autonomous-scanning-rework

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-turret-behavior-system.md

## Task

Rework the existing `turret_autonomous_scanning_system` in `game/combat/systems/core.rs` to use `TurretCommandState` instead of `AttackState`.

**Current state:** The system at line ~265 finds the best target and writes to `attack_state.current_target` and `attack_state.phase = AttackPhase::Aiming`. It checks `attack_state.target_entity().is_some()` to skip units that already have a target.

**Required changes:**
1. Query for `&mut TurretCommandState` instead of `&mut AttackState`
2. Skip scanning when `turret_command_state.locked_target.is_some()` (target already assigned by base behavior or previous scan)
3. On finding best target: set `turret_command_state.locked_target = Some(target_entity)` instead of writing to AttackState
4. When no valid targets exist: set `turret_command_state.locked_target = None` (turret inactive state — the engagement system will set TurretAttackChannel::Inactive)
5. Add target validity check: if `locked_target` refers to a despawned entity, clear it so scanning resumes
6. Keep the existing target selection algorithm unchanged (threatening > least rotation > closest)

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/combat/systems/core.rs`** — lines 265-349: `turret_autonomous_scanning_system`

### Import changes needed
- **Line 5** currently: `use crate::game::units::types::{UnitCommand, UnitControlCost};`
- Add `TurretCommandState` to this import: `use crate::game::units::types::{UnitCommand, UnitControlCost, TurretCommandState};`
- `TurretCommandState` is defined in `artifacts/developer/src/game/units/types/state/commands.rs` (line 208) and re-exported via `state/mod.rs` wildcards

### TurretCommandState component (commands.rs:208-213)
```rust
#[derive(Component, Clone, Debug, Default)]
pub struct TurretCommandState {
    pub locked_target: Option<Entity>,
}
```
- `locked_target: Option<Entity>` — `None` means autonomous scanning active, `Some(entity)` means target locked

### Current query signature (line 266-269)
```rust
mut units: Query<
    (Entity, &Transform, &Turret, &AttackCapability, &mut AttackState, &Owner, Option<&DomainEnum>, &GridPosition),
    With<Unit>
>,
```
**Replace** `&mut AttackState` with `&mut TurretCommandState` in the query tuple. The `With<Unit>` filter is still correct — turret units are a subset of units, and the query will only match entities that also have `TurretCommandState`.

### Key changes in the system body

1. **Skip check** (line 275): Change `attack_state.target_entity().is_some()` to `turret_command_state.locked_target.is_some()`

2. **Target validity check** (NEW — add before the skip check): If `locked_target` is `Some(entity)`, verify the entity still exists in `potential_targets` query. If `potential_targets.get(entity).is_err()`, set `locked_target = None` and fall through to scanning. Pattern:
```rust
if let Some(target) = turret_command_state.locked_target {
    if potential_targets.get(target).is_err() {
        turret_command_state.locked_target = None;
    } else {
        continue; // target still valid, skip scanning
    }
}
```

3. **On finding target** (lines 343-347): Replace:
```rust
attack_state.current_target = Some(AttackTarget::UnitTarget(target));
attack_state.phase = AttackPhase::Aiming;
attack_state.time_in_phase = 0.0;
```
With:
```rust
turret_command_state.locked_target = Some(target);
```

4. **No target found** (implicit else after the if-let): When no best_candidate found, `locked_target` stays `None` (already the case since we only enter the loop body when it was `None`). No explicit action needed.

### System registration (unchanged)
- Registered in `CombatPlugin` (`artifacts/developer/src/game/combat/mod.rs`, line 36) within `DiagCategory::Combat`
- No ordering changes needed — it runs in the same flat set as other combat systems

### Existing tests (core.rs lines 676-704)
- Three unit tests test priority logic via tuple comparison only (not the actual system). These tests are algorithm-logic tests and do NOT need to change since the priority algorithm stays the same.
- Consider adding a new test that verifies `locked_target` is set (instead of `AttackState`) after scanning. Follow the existing tuple-comparison pattern or use a full `App`-based test.

### Related types for reference (DO NOT modify)
- `AttackState` — remains on turret units but is no longer written by this system
- `TurretAttackChannel` (behavior.rs:114) — the downstream engagement system reads `locked_target` and writes to this channel (separate task)
- `TurretBehaviorState` (behavior.rs:44) — tracks scan heading/direction, not used by this system currently

## Dependencies

- **TurretCommandState component must exist**: Already implemented in `artifacts/developer/src/game/units/types/state/commands.rs` (line 208). Spawned on turret units via `artifacts/developer/src/game/units/utils.rs` (per insights). No dependency on other planned_tasks.
- **turret-engagement-system** (sibling task): The engagement system is the downstream consumer of `locked_target`. This task can be completed independently — after this rework, `locked_target` will be set but not yet consumed by any engagement system. That is the expected intermediate state.
- **turret-base-behavior-target-relay** (sibling task): The relay system will write to `locked_target` from base commands. This task's validity check handles despawned entities regardless of who wrote `locked_target`, so no ordering dependency.
