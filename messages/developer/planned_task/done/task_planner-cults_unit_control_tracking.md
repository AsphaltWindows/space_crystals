# cults_unit_control_tracking

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Implement the Unit Control tracking system that traces each Cults unit back to its originating Recruitment Center. This includes:

1. An `OriginatingCenters` component that tracks which RC(s) produced the unit
2. A death tracking system that decrements each originating RC's `local_used` when a Cults unit dies
3. Integration point for the production system to attach the component at spawn time

## Technical Context

### Files to Create/Modify

**New component — `artifacts/developer/src/game/units/types/unit_data.rs`**
- Add `OriginatingCenters` component here, alongside existing unit-level components like `UnitControlCost` (line 98), `RuggedTerrainDefenseBonus`, `TunnelSpaceCost`
- Definition:
```rust
#[derive(Component, Clone, Debug)]
pub struct OriginatingCenters {
    pub centers: Vec<Entity>,
}
```
- Re-export from `artifacts/developer/src/game/units/types/mod.rs` (line 10 area, alongside other `pub use` statements)

**Death tracking system — `artifacts/developer/src/game/combat/systems/core.rs`**
- Add `cults_unit_death_tracking_system` in this file, near `remove_dead_entities_system` (line 757)
- This system must run BEFORE `remove_dead_entities_system` (which calls `commands.entity(e).despawn()` at line 801), so it can read `OriginatingCenters` before the entity is despawned
- Pattern to follow: mirror the existing `remove_dead_entities_system` structure — iterate entities checking `!obj.is_alive()`, then process `OriginatingCenters`
- Query signature:
```rust
pub fn cults_unit_death_tracking_system(
    dying_units: Query<(&ObjectInstance, &OriginatingCenters)>,
    mut rc_query: Query<&mut RecruitmentCenterState>,
) {
    for (obj, origins) in dying_units.iter() {
        if !obj.is_alive() {
            for &center_entity in &origins.centers {
                if let Ok(mut rc_state) = rc_query.get_mut(center_entity) {
                    rc_state.local_used = rc_state.local_used.saturating_sub(1);
                }
                // If center entity doesn't exist (destroyed), get_mut returns Err — gracefully skipped
            }
        }
    }
}
```
- Key: use `saturating_sub(1)` to prevent underflow, and let `get_mut` failure handle destroyed centers

**Register the system — `artifacts/developer/src/game/combat/mod.rs`**
- In `CombatPlugin::build()` (line 28-43), add system with ordering:
```rust
systems::cults_unit_death_tracking_system.before(systems::remove_dead_entities_system),
```
- Both systems are in `DiagCategory::Combat` set (line 43)
- Import `OriginatingCenters` and `RecruitmentCenterState` in `core.rs` imports

**Imports needed in `core.rs`:**
- `use crate::game::units::types::OriginatingCenters;` (or via the re-export path)
- `use crate::game::types::structures::RecruitmentCenterState;`

### Existing Types/Resources
- `RecruitmentCenterState` (`game/types/structures.rs` line 499): has `local_used: u32` field (line 509) — this is what gets decremented
- `ObjectInstance` (`game/types/objects.rs`): `is_alive()` method used to detect dead entities
- `CultsPlayerResources` (`game/types/factions.rs` line 171): has `unit_control_used: u32` — the GLOBAL counter. Note: the task only asks for per-center tracking via `local_used`. The global `unit_control_used` is managed separately by the existing TODO in `remove_dead_entities_system` (line 794-797)

### System Ordering
- `DiagCategory::Combat` runs after `DiagCategory::Movement` (simulation/mod.rs line 36)
- Within Combat: `apply_damage_system` → (implicitly parallel) → `remove_dead_entities_system`
- New ordering: `cults_unit_death_tracking_system` BEFORE `remove_dead_entities_system` (to read OriginatingCenters before entity despawn)
- `apply_damage_system` sets `current_hp` to 0 → `is_alive()` returns false → death tracking reads it → remove_dead despawns

### Integration with Production (future)
- The `recruitment_center_auto_production` task (sibling) will spawn Recruits with `OriginatingCenters { centers: vec![center_entity] }`
- That task also increments `local_used` on the RC — this task handles the reverse (decrement on death)

### Test Patterns
- Follow existing test patterns in `core.rs` (test module starts at line 806)
- Use `App::new()` with minimal setup — spawn entities with `ObjectInstance`, `OriginatingCenters`, `RecruitmentCenterState`
- Test cases per the task:
  1. Recruit with `centers: [A]` dies → A's local_used goes from 1 to 0
  2. Trained unit with `centers: [A, A, B]` dies → A's local_used decrements by 2, B's by 1
  3. Unit dies after center destroyed → no panic (get_mut returns Err)
  4. local_used never goes below 0 (saturating_sub)

### Bevy ECS Notes
- No `RemovedComponents` needed — the system checks `is_alive()` before despawn happens, which is simpler and more reliable
- The system is read-only on `ObjectInstance` and `OriginatingCenters`, mutable only on `RecruitmentCenterState`
- No conflicts with `remove_dead_entities_system` since that system doesn't query `RecruitmentCenterState`

## Dependencies

- **`recruitment_center_auto_production` (sibling planned_task)**: That task spawns Recruits with `OriginatingCenters` and increments `local_used`. This task defines the component and handles the decrement side. Either can land first — the component definition lives here, and the production system will use it. If production lands first, it can define a placeholder component that this task replaces.
- **`RecruitmentCenterState` (existing)**: Already implemented in `game/types/structures.rs` with `local_used` field. No changes needed.
- **`remove_dead_entities_system` (existing)**: Must run AFTER the new death tracking system. The ordering constraint is explicit via `.before()`.
