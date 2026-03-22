# recruitment_center_auto_production

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Implement the auto-production system for Recruitment Centers and HUD unit control aggregation.

### Production System
Create `recruitment_center_production_system` in `game/world/faction.rs` (or a new cults module):

Each frame, for each RecruitmentCenter with effectiveness > 0:
1. Check if `local_used < local_capacity` — if at capacity, skip (do not produce)
2. Compute production frames needed: `base_frames = 12 * FRAMES_PER_SECOND` (12 seconds at 100%), scaled by effectiveness: `required_frames = (base_frames as f32 / effectiveness).ceil() as u32`. At 50% effectiveness, production takes 24 seconds (384 frames at 16 FPS).
3. Increment `production_progress += 1`
4. When `production_progress >= required_frames`:
   - Reset `production_progress = 0`
   - Spawn a Recruit unit at the center's grid position (use a placeholder spawn — the Recruit unit type will be defined in a future feature; for now spawn a minimal entity with ObjectEnum::CultsRecruit if the variant exists, or leave a clear TODO/stub)
   - Add `OriginatingCenter(center_entity)` component to the spawned Recruit (see unit_control_tracking task)
   - Increment `local_used += 1`
   - If `rally_point` is set, issue a Move command to the spawned Recruit

### Recruit Unit Stub
Add `ObjectEnum::CultsRecruit` to `shared/types.rs` with minimal object_type data:
- Size: 1x1 (unit, not structure)
- Groupable: true
- Faction: TheCults
- This is a placeholder — full Recruit stats/behaviors will come in a separate feature

Add a minimal `spawn_cults_recruit()` function that spawns a unit entity with ObjectEnum::CultsRecruit, Owner, basic Transform, Unit marker, Selectable, SelectionBounds.

### HUD Unit Control Aggregation
Update `update_resource_bar_system` in `hud.rs` for TheCults faction:
- Query all RecruitmentCenterState components owned by the local player
- Sum `local_capacity` across all centers → `unit_control_available`
- Sum `local_used` across all centers → `unit_control_used`
- Update CultsPlayerResources: `unit_control_available = sum_capacity`, `unit_control_used = sum_used`
- Display format: "UC: {used} / {available}"

Alternatively, create a dedicated `cults_unit_control_aggregation_system` that syncs RecruitmentCenterState totals into CultsPlayerResources each frame, and let the existing HUD display read from CultsPlayerResources as it already does.

### Registration
Register the production system and aggregation system. Production should run in the same phase as other production systems (barracks_production_tick_system, headquarters_production_tick_system).

### Tests
- RC at 100% effectiveness: produces a Recruit every 192 frames (12s * 16fps)
- RC at 50% effectiveness: produces a Recruit every 384 frames
- Production stops when local_used == local_capacity
- Production resumes when local_used drops below local_capacity (e.g., unit dies)
- HUD aggregation: 2 centers with capacity 20 and 10 → displays "UC: 0 / 30"
- Recruit spawns at center position and receives rally command if rally_point set
