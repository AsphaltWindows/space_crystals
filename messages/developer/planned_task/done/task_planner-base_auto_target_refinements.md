# base-auto-target-refinements

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-base-auto-targeting.md

## Task

Refine the existing base auto-targeting systems (`base_auto_target_system` in `combat/systems/core.rs` and `hold_position_behavior_system` in `combat/systems/behaviors.rs`) to match the full specification. Four changes:

1. **Target Selection Priority (both systems)**: Replace nearest-distance-only selection with 3-tier priority: threatening > least rotation > closest distance.
2. **Use SightRange for Idle Scanning**: `base_auto_target_system` idle units should scan within SightRange, not attack range.
3. **Remove AttackMove from base_auto_target_system**: Remove `UnitCommand::AttackMove(_)` from the allowed command match.
4. **Add ValidTarget Filtering**: Filter targets through `is_valid_target()` (destructible, visible, domain-compatible).

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/combat/systems/core.rs`** — `base_auto_target_system` (line 357) and test module (line 605)
2. **`artifacts/developer/src/game/combat/systems/behaviors.rs`** — `hold_position_behavior_system` (line 368) and test module (line 535)
3. **`artifacts/developer/src/game/combat/utils.rs`** — Already contains `is_valid_target()` (line 278) and `is_domain_compatible()` (line 262) — use these as-is, no changes needed
4. **(Optional)** Extract shared `select_best_target()` utility into `combat/utils.rs`

### Change 1: 3-Tier Target Selection Priority

Both `base_auto_target_system` and `hold_position_behavior_system` currently select the nearest enemy. Replace with the priority pattern already implemented in `turret_autonomous_scanning_system` (core.rs line 265-351). That system uses:

```rust
// Candidate: (threatening, rotation_abs, distance, entity)
let mut best_candidate: Option<(bool, f32, f32, Entity)> = None;
// ...
let threatening = can_threaten(target_attack_cap, &src_domain);
let rotation_abs = relative_angle.abs();
// Compare: prefer threatening > least rotation > closest
let is_better = match &best_candidate {
    None => true,
    Some((best_threat, best_rot, best_dist, _)) => {
        if threatening && !best_threat { true }
        else if threatening == *best_threat {
            if rotation_abs < *best_rot - 0.01 { true }
            else if (rotation_abs - best_rot).abs() < 0.01 { distance < *best_dist }
            else { false }
        } else { false }
    }
};
```

**Key implementation details:**

- The `can_threaten` function (core.rs line 235) is currently a stub returning `true`. It needs to be implemented to actually check if the target's `AttackCapability.target_domain` can hit the defender's `DomainEnum`. Use `is_domain_compatible()` from `combat/utils.rs` for this. Signature: `can_threaten(target_attack_cap: Option<&AttackCapability>, defender_domain: &DomainEnum) -> bool`. If target has no AttackCapability (structures without weapons), return false (not threatening).
- For rotation calculation in non-turret units, use `compute_relative_turret_angle()` (core.rs line 244) — it computes the angle between unit forward and direction to target.
- Both queries for `potential_targets` need to be extended to include `Option<&AttackCapability>` for threat assessment.

**Consider extracting a shared utility function:**
```rust
pub fn select_best_target(
    candidates: impl Iterator<Item = (Entity, bool, f32, f32)>, // (entity, threatening, rotation_abs, distance)
) -> Option<Entity>
```
Place this in `combat/utils.rs` alongside `is_valid_target()`.

### Change 2: SightRange for Idle Scanning

In `base_auto_target_system` (core.rs line 357):
- Add `Option<&SightRange>` to the unit query (SightRange is `pub struct SightRange(pub u32)` in `shared/types.rs`, a newtype over u32 grid units)
- Add `use crate::types::SightRange;` to imports (line 2: currently imports `Unit, Owner, DomainEnum, GridPosition`)
- When the command is `UnitCommand::Idle`, use `sight_range.map(|sr| sr.0 as f32).unwrap_or(attack_cap.range)` as the scan range
- When the command is `UnitCommand::HoldPosition`, continue using `attack_cap.range` (weapon range)
- NOTE: After removing AttackMove (change 3), HoldPosition is the only other case

### Change 3: Remove AttackMove from base_auto_target_system

In the match at line 377:
```rust
// BEFORE:
UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_) => {}
// AFTER:
UnitCommand::Idle | UnitCommand::HoldPosition => {}
```

AttackMove has its own scanning in `attack_move_behavior_system` (behaviors.rs line 210). Verify no conflict: `hold_position_behavior_system` handles HoldPosition scanning independently (line 381 checks `UnitCommand::HoldPosition`), so `base_auto_target_system` seeing HoldPosition for IdleOrigin insertion is fine since it only inserts IdleOrigin for `UnitCommand::Idle` (line 418).

### Change 4: ValidTarget Filtering

Both systems' inner target loops currently only check `is_enemy()`. Add filtering:

**New imports needed in core.rs:**
```rust
use crate::types::{Unit, Owner, DomainEnum, GridPosition, VisibilityStateEnum, SightRange};
use crate::game::combat::utils::is_valid_target;
```

**New imports needed in behaviors.rs:**
```rust
use crate::types::{..., VisibilityStateEnum};
use crate::game::combat::utils::is_valid_target;
```

**Query changes for potential_targets:**
- core.rs `base_auto_target_system` (line 363): Add `&ObjectInstance, &VisibilityStateEnum, Option<&AttackCapability>` to `potential_targets` query
- behaviors.rs `hold_position_behavior_system` (line 375): Add `&ObjectInstance, &VisibilityStateEnum, Option<&AttackCapability>` to `potential_targets` query

**Filter call (add after is_enemy check):**
```rust
if !is_valid_target(&target_obj, &target_vis, &tgt_domain, &attack_cap.target_domain) {
    continue;
}
```

Note: `is_valid_target()` checks destructibility, visibility state (must be `Visible`), and domain compatibility. The Bevy `Visibility` component is separate from `VisibilityStateEnum` (fog-of-war) — use `VisibilityStateEnum` here.

### Existing Tests to Update (core.rs line 708-770)

The `base_auto_target_allows_attack_move` test (line 725) must be changed to verify AttackMove is now BLOCKED. Several other tests reference the old 3-match pattern and need updating to the new 2-match (`Idle | HoldPosition`).

Also update the `can_threaten` stub tests (line 610-631) to reflect the real implementation.

### New Tests to Add

**In core.rs tests:**
- `base_auto_target_blocks_attack_move` — AttackMove is no longer in the allowed set
- `can_threaten_ground_attack_threatens_ground_unit` — target with TargetDomainEnum::Ground threatens DomainEnum::Ground defender
- `can_threaten_air_attack_does_not_threaten_ground_unit` — target with TargetDomainEnum::Air does NOT threaten DomainEnum::Ground defender  
- `can_threaten_no_attack_cap_not_threatening` — None AttackCapability returns false

**In behaviors.rs tests (or core.rs shared utility tests):**
- `select_best_target_threatening_over_closer` — threatening target wins despite worse distance
- `select_best_target_least_rotation_among_equal_threat` — least rotation wins when both threatening
- `select_best_target_closest_as_tiebreaker` — closest wins when threat and rotation equal

### System Registration

No changes needed to `CombatPlugin` (combat/mod.rs line 23) — both systems are already registered. No ordering changes required.

## Dependencies

- None — this task refines existing systems in-place with no new components or system registrations. The `is_valid_target()`, `is_domain_compatible()`, `can_threaten()`, and `compute_relative_turret_angle()` utilities already exist. `SightRange` and `VisibilityStateEnum` components are already spawned on relevant entities.
