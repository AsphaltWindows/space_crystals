# cults_unit_control_tracking

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Implement the Unit Control tracking system that traces each Cults unit back to its originating Recruitment Center.

### OriginatingCenter Component
Create an `OriginatingCenter` component (in `game/types/cults.rs` or `shared/types.rs`):
- For Recruits: `OriginatingCenter(Entity)` — single center entity
- For trained units (consuming multiple Recruits): `OriginatingCenters(Vec<Entity>)` — one entry per consumed Recruit, preserving which center each came from

Consider using a single component design:
```rust
#[derive(Component, Clone, Debug)]
pub struct OriginatingCenters {
    /// Each entry is the RecruitmentCenter entity that produced the original Recruit.
    /// For a Recruit, this has exactly 1 entry.
    /// For a trained unit, this has N entries (one per consumed Recruit).
    pub centers: Vec<Entity>,
}
```

### Lineage Persistence Through Training
When a Recruit is trained into another unit (consumed during training):
- The new unit's `OriginatingCenters` should be the union of all consumed Recruits' centers
- Each Recruit contributes exactly 1 entry from its own OriginatingCenters
- Example: 3 Recruits from Center A + 2 from Center B → trained unit has centers = [A, A, A, B, B]

This means the trained unit's Unit Control cost (= number of Recruits consumed) is `centers.len()`, with each point attributed to the respective center.

Note: The actual training system will be implemented in a future feature. This task establishes the component and the death-tracking system.

### Death Tracking System
Create `cults_unit_death_tracking_system`:
- When a Cults unit with OriginatingCenters is destroyed (removed from world):
  - For each center Entity in the `centers` list, decrement that center's `RecruitmentCenterState.local_used` by 1
  - If the center entity no longer exists (was destroyed), skip it (the capacity is already gone)

Use `RemovedComponents<OriginatingCenters>` paired with a shadow/tracking resource, OR hook into the existing `remove_dead_entities_system` to process OriginatingCenters before entity removal.

### Integration with Production
The recruitment_center_auto_production task spawns Recruits with OriginatingCenters. Ensure the component is added during spawn:
```rust
OriginatingCenters { centers: vec![center_entity] }
```

### Registration
Register the death tracking system to run after damage/death systems but before entity cleanup.

### Tests
- Recruit spawned from Center A has OriginatingCenters { centers: [A] }
- Recruit dies → Center A's local_used decrements by 1
- Trained unit from 3 Recruits (2 from A, 1 from B) has centers = [A, A, B], len = 3
- Trained unit dies → Center A's local_used decrements by 2, Center B's by 1
- Unit dies after its originating center was destroyed → no panic, graceful skip
- local_used never goes below 0
