# cults_construction_system

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_objects_formalized.md

## Task

Implement the Cults construction execution system — the mechanics of Recruits entering buildings under construction, proportional speedup, completion consumption, and cancellation.

### Overview

When Recruits reach a Cults building under construction, they enter it (become hidden/consumed into the building). The building's construction speed scales linearly with the number of Recruits inside. On completion, all Recruits inside are consumed (despawned). On cancellation, all Recruits are ejected and returned to play.

### New component: CultsConstructionState

Add a component to track Cults building construction state:
- `assigned_recruits: Vec<Entity>` — entities of Recruits inside the building
- `construction_progress: u32` — frames of progress completed
- `total_construction_frames: u32` — total frames needed (base, with 1 Recruit)

Attach this component when a Cults building is spawned under-construction (from the placement task).

### Recruit enter behavior

When a Recruit with a ConstructBuilding command reaches the target building:
- Add the Recruit entity to the building's `CultsConstructionState.assigned_recruits`
- Set the Recruit to Visibility::Hidden (or despawn — but keeping entity allows cancellation refund)
- Remove the Recruit's movement/command state

### Construction tick system (`cults_construction_tick_system`)

Each frame, for each entity with CultsConstructionState + ConstructionHP:
- If `assigned_recruits` is empty, do nothing (construction paused)
- Progress = number of assigned Recruits per frame (1 Recruit = 1 frame progress per frame, 2 Recruits = 2 frames per frame, etc.)
- Increment `construction_progress` by the number of assigned Recruits
- Update ConstructionHP proportionally (hp = max_hp * (construction_progress / total_construction_frames), minimum 10% per ConstructionHP Rule)
- When `construction_progress >= total_construction_frames`:
  - Set HP to full max HP, remove ConstructionHP component
  - Despawn all Recruit entities in `assigned_recruits` (they are consumed)
  - Remove CultsConstructionState component (building is complete)
  - The building becomes fully operational

### Cancellation

When a Cults building under construction is cancelled (via a cancel command or destruction):
- For each Recruit in `assigned_recruits`:
  - Restore Visibility to Visible
  - Place them near the building's position (find valid nearby tile)
  - Restore their command/movement state to idle
- Despawn the building entity
- No resource cost to refund (Cults buildings cost Recruits, not crystals — the Recruits themselves are the cost)

### Multiple Recruits speedup

The proportional speedup is inherent in the tick system: progress per frame = count of assigned Recruits. With base construction requiring N frames with 1 Recruit, having K Recruits completes in N/K frames.

### Tests

- Test that 1 Recruit completes construction in exactly `total_construction_frames` frames
- Test that 2 Recruits complete in half the frames
- Test that cancellation returns all assigned Recruits
- Test that Recruits are despawned on completion
- Test ConstructionHP scales correctly during construction

## Technical Context

### New Component: CultsConstructionState

Define in `artifacts/developer/src/game/types/structures.rs` alongside `ConstructionHP` (line 11) and `RecruitmentCenterState` (line 499):

```rust
#[derive(Component, Clone, Debug)]
pub struct CultsConstructionState {
    pub assigned_recruits: Vec<Entity>,
    pub construction_progress: u32,
    pub total_construction_frames: u32,
}
```

Export via `pub use structures::*` already in `game/types/mod.rs` line 10.

### New UnitCommand Variant: ConstructBuilding

Add `ConstructBuilding(Entity)` to the `UnitCommand` enum in `artifacts/developer/src/game/units/types/state/commands.rs` (line 8). This is the command issued to Recruits that tells them to walk toward and enter a Cults building under construction. Add an `is_available()` entry — should return `true` unconditionally (or add an `is_cults` param later).

**Note**: The `ConstructBuilding` variant does NOT exist yet — no code references it. This task must add it.

### Recruit Enter Behavior System

Create a new behavior system in `artifacts/developer/src/game/units/systems/behaviors.rs`. Follow the `building_tunnel_behavior_system` pattern (line 1010-1182) which handles Agent tunnel construction:

**Pattern to follow**:
1. Query Recruit units with `UnitCommand::ConstructBuilding(target_entity)`
2. Check distance to target building (use `BUILD_ARRIVAL_THRESHOLD = 2.0` from behaviors.rs line 999)
3. If not arrived: set `LocomotionChannel::Moving(vec![target_pos])` + `OrientationChannel::Turning(target_pos)` — same as tunnel building (line 1139-1140)
4. If arrived:
   - Add recruit entity to `CultsConstructionState.assigned_recruits` on the target building
   - Set `*visibility = Visibility::Hidden` (same pattern as tunnel Agent hiding, line 1128)
   - Set `*command = UnitCommand::Idle` and `*locomotion = LocomotionChannel::Stationary`
   - The recruit entity stays alive but hidden (not despawned — needed for cancellation refund)

**Key difference from tunnel pattern**: Multiple recruits can enter the same building (tunnel is single-Agent). The tunnel Agent despawns on completion; Cults recruits are kept alive-but-hidden until completion/cancellation.

**Query signature for the behavior system**:
```rust
fn cults_recruit_enter_construction_system(
    mut commands: Commands,
    mut recruits: Query<(Entity, &Transform, &mut UnitCommand, &mut Visibility, &mut LocomotionChannel, &mut OrientationChannel), With<Unit>>,
    mut buildings: Query<(Entity, &Transform, &mut CultsConstructionState), With<ConstructionHP>>,
)
```

**Note on Recruit components**: Recruits are spawned with minimal components in `spawn_cults_recruit()` (`artifacts/developer/src/game/utils.rs` line 1043). They currently have `UnitCommand` but NOT `LocomotionChannel`/`OrientationChannel`/`BaseBehaviorState`. If the sibling task that formalizes Recruit stats doesn't add these, this system should either:
- Use the simpler `MoveTarget`/`Path` movement pipeline (older pattern, used by combat behaviors)
- Or just check distance + teleport if movement components aren't present (placeholder approach)

### Construction Tick System: cults_construction_tick_system

Add in `artifacts/developer/src/game/world/faction.rs` near the existing `construction_hp_tick_system` (line 946). **Important**: This system REPLACES the generic `construction_hp_tick_system` for Cults buildings. The generic system increments progress by `1.0/build_frames` per frame, but Cults buildings should only progress when recruits are assigned and progress scales with recruit count.

```rust
pub fn cults_construction_tick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP, &mut CultsConstructionState)>,
) {
    for (entity, mut obj, mut construction, mut cults_state) in query.iter_mut() {
        let recruit_count = cults_state.assigned_recruits.len() as u32;
        if recruit_count == 0 { continue; } // Paused
        
        cults_state.construction_progress += recruit_count;
        
        // Update ConstructionHP progress (0.0 to 1.0)
        let progress = (cults_state.construction_progress as f32 / cults_state.total_construction_frames as f32).min(1.0);
        construction.progress = progress;
        
        // Update HP using ConstructionHP::hp_fraction()
        if let Some(max_hp) = obj.max_hp {
            obj.hp = Some(max_hp * ConstructionHP::hp_fraction(progress));
        }
        
        // Check completion
        if cults_state.construction_progress >= cults_state.total_construction_frames {
            // Despawn all assigned recruits
            for recruit_entity in cults_state.assigned_recruits.drain(..) {
                commands.entity(recruit_entity).despawn();
            }
            // Remove construction components
            commands.entity(entity).remove::<ConstructionHP>();
            commands.entity(entity).remove::<CultsConstructionState>();
        }
    }
}
```

**Critical interaction**: The generic `construction_hp_tick_system` (line 946) will ALSO match entities with `ConstructionHP` and auto-increment progress by `1/build_frames` each frame. To avoid double-progression, either:
1. **(Preferred)** Add `Without<CultsConstructionState>` filter to the generic system's query so it skips Cults buildings
2. Or run `cults_construction_tick_system` BEFORE the generic one and remove `ConstructionHP` before it runs

### System Registration

Register in `artifacts/developer/src/game/world/mod.rs` line 109-122, in the `FixedUpdate` `DiagCategory::Construction` set alongside other tick systems:

```rust
faction::cults_construction_tick_system,  // Add here
```

Also add the recruit enter behavior system. If placed in behaviors.rs, register in `artifacts/developer/src/game/units/mod.rs` in the appropriate phase. If placed in faction.rs, register in world/mod.rs.

### Cancellation System: cults_construction_cancel_system

Add in `artifacts/developer/src/game/world/faction.rs`. This handles both:
1. **Explicit cancel** (command panel cancel button) — the command panel task will call this
2. **Building destruction** (HP reaches 0) — detect via `!obj.is_alive()` check

For destruction-based cancellation, this system should run BEFORE `remove_dead_entities_system` (registered in `artifacts/developer/src/game/combat/mod.rs` line 43):

```rust
pub fn cults_construction_cancel_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ObjectInstance, &Transform, &mut CultsConstructionState)>,
) {
    for (entity, obj, transform, mut cults_state) in query.iter_mut() {
        if !obj.is_alive() {
            // Building was destroyed — eject all recruits
            for recruit_entity in cults_state.assigned_recruits.drain(..) {
                if commands.get_entity(recruit_entity).is_ok() {
                    commands.entity(recruit_entity)
                        .insert(Visibility::Inherited)
                        .insert(UnitCommand::Idle);
                    // Position near building (offset slightly)
                    // Transform modification needs a separate query or direct insert
                }
            }
        }
    }
}
```

**Ejection positioning**: When restoring recruits, place them near the building. Use the building's `Transform.translation` as base position and offset each recruit by a small amount (e.g., 1.0 unit spacing). The tunnel ejection system (`ejection_tick_system` at faction.rs line 1816) uses tunnel Side A position — follow a similar approach of placing at a fixed offset.

### Interaction with remove_dead_entities_system

The `remove_dead_entities_system` (`artifacts/developer/src/game/combat/systems/core.rs` line 776) despawns entities where `!obj.is_alive()`. The cancel system must run BEFORE this to eject recruits before the building entity is despawned. Register with `.before(systems::remove_dead_entities_system)` in `artifacts/developer/src/game/combat/mod.rs` line 42-43, following the pattern of `cults_unit_death_tracking_system`.

### Modifying the Generic construction_hp_tick_system

In `artifacts/developer/src/game/world/faction.rs` line 948, add `Without<CultsConstructionState>` to the query:

```rust
// Before:
mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP)>,
// After:
mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP), Without<CultsConstructionState>>,
```

This ensures the generic tick system skips Cults buildings (they have their own tick logic).

### Tests

Place tests in a `#[cfg(test)] mod tests` block within faction.rs or in a new test module. Use the existing test patterns:

- `spawn_cults_recruit()` (`utils.rs:1043`) to create recruit entities
- `spawn_cults_storage()` (`utils.rs:988`) for a building entity, then insert `CultsConstructionState` + `ConstructionHP` on it
- Use `TestApp` from `shared/testing/test_app.rs` for App setup (needs `MinimalPlugins` + `Assets<Mesh>` + `Assets<StandardMaterial>`)
- Run systems with `app.world_mut().run_system_once(system_fn)` pattern
- See existing construction tests at behaviors.rs line 2370+ for the tunnel construction test pattern

### Key Files Summary

| File | Change |
|------|--------|
| `game/types/structures.rs` | Add `CultsConstructionState` component |
| `game/units/types/state/commands.rs` | Add `UnitCommand::ConstructBuilding(Entity)` variant |
| `game/units/systems/behaviors.rs` | Add recruit enter construction behavior system |
| `game/world/faction.rs` | Add `cults_construction_tick_system`, `cults_construction_cancel_system`; modify `construction_hp_tick_system` query to exclude Cults buildings |
| `game/world/mod.rs` | Register new systems in FixedUpdate/DiagCategory::Construction |
| `game/combat/mod.rs` | Register cancel system before `remove_dead_entities_system` (if destruction-based cancel is here) |

## Dependencies

- **spawn_cults_recruit / spawn_cults_storage** (already exist in `game/utils.rs`): Used to spawn test entities. These are placeholder stubs — this task's system should work with whatever components are present on Recruits.
- **ConstructionHP** (exists in `game/types/structures.rs`): The standard construction HP component. Cults construction manipulates its `progress` field directly rather than relying on the generic tick.
- **Sibling task: cults_recruit_interface** (or equivalent): The command panel task that issues `UnitCommand::ConstructBuilding(entity)` commands and provides the cancel button. This task defines the mechanics; the interface task triggers them.
- **Sibling task: cults_recruit_placement** (or equivalent): The placement task spawns buildings with `CultsConstructionState` + `ConstructionHP`. This task assumes those components exist on the entity.
- **remove_dead_entities_system** (exists in `game/combat/systems/core.rs`): The cancel system must run before this to eject recruits before building despawn.
