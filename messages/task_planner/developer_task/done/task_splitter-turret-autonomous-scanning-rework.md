# turret-autonomous-scanning-rework

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

**Key files:**
- `game/combat/systems/core.rs` — turret_autonomous_scanning_system (~line 265)
- `game/units/types/state/commands.rs` — TurretCommandState component (line 203)
- `game/units/types/state/behavior.rs` — TurretBehaviorState (line 45), TurretAttackChannel (line 115)
