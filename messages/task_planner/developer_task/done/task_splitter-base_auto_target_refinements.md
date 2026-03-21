# base-auto-target-refinements

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-base-auto-targeting.md

## Task

Refine the existing base auto-targeting systems (`base_auto_target_system` in `combat/systems/core.rs` and `hold_position_behavior_system` in `combat/systems/behaviors.rs`) to match the full specification. The basic structure works but has several gaps:

### 1. Target Selection Priority (both systems)
Replace the current nearest-distance-only selection with a 3-tier priority:
1. **Threatening targets first** — enemies whose attack domain (`AttackCapability.attack_type` → `TargetDomainEnum`) can hit this unit's `DomainEnum`. Query `AttackCapability` on potential targets to determine this.
2. **Least rotation** — among equally-threatening candidates, prefer the one requiring least facing change (angle between unit's forward vector and direction to target).
3. **Closest distance** — final tiebreaker.

This priority applies to both `base_auto_target_system` (idle scanning) and `hold_position_behavior_system` (hold position scanning). Consider extracting a shared `select_best_target()` utility.

### 2. Use SightRange for Idle Scanning
In `base_auto_target_system`, idle units should scan within their `SightRange` component value (converted to space units), not `attack_cap.range`. Add `&SightRange` to the query. HoldPosition scanning in `hold_position_behavior_system` correctly uses attack range (weapon range) — leave that as-is.

### 3. Remove AttackMove from base_auto_target_system
The `base_auto_target_system` currently allows `UnitCommand::AttackMove(_)` in its match. Remove it — AttackMove has its own scanning logic in `attack_moving_to_location_behavior_system`. The allowed set should be only `Idle` and `HoldPosition` (HoldPosition is handled by hold_position_behavior_system, but base_auto_target_system also covers it for the IdleOrigin path — verify there's no conflict and adjust as needed).

### 4. Add ValidTarget Filtering
Both systems scan `potential_targets` without checking domain compatibility. Add `is_valid_target()` or at minimum `is_domain_compatible()` checks (from `combat/utils.rs`) to filter targets the unit can actually attack. Also check `Visibility` is not `Hidden`.

### Key files:
- `src/game/combat/systems/core.rs` — `base_auto_target_system`, `idle_leash_system`
- `src/game/combat/systems/behaviors.rs` — `hold_position_behavior_system`
- `src/game/combat/utils.rs` — `is_valid_target()`, `is_domain_compatible()`
- `src/game/combat/types.rs` — `IdleOrigin`, `IDLE_LEASH_DISTANCE`
- `src/shared/types.rs` — `SightRange`

### Constants:
- `IDLE_LEASH_DISTANCE` = 4.0 (already correct, in grid units mapped to space units via existing scale)
- `HOLD_POSITION_FACING_ARC` = PI/6 (already correct)

### Tests:
- Update existing tests in both files to reflect the new priority logic
- Add test: threatening target preferred over closer non-threatening target
- Add test: among equally threatening targets, least rotation preferred
- Add test: domain-incompatible targets are filtered out
- Add test: AttackMove no longer triggers base auto-targeting
