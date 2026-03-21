# tunnel_arrival_validation

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-worker-built-validation.md

## Task

Add arrival validation to `build_tunnel_behavior_system` in `artifacts/developer/src/game/units/systems/behaviors.rs`.

Currently, when a `BuildingTunnelBehavior` unit arrives at its target (the `BuildTunnelPhase::MovingToSite` branch, ~line 929), it only checks single-agent enforcement and cost before spawning the tunnel. It does NOT validate that the tiles are still buildable and unoccupied.

**What to implement:**

After the `BUILD_ARRIVAL_THRESHOLD` distance check passes (line 929) and before the single-agent enforcement check (line 932), add a call to `can_worker_place_structure()` to validate the placement:

1. Convert the target world position to grid coordinates using `world_to_grid(target, 1.0)`
2. Get the tunnel's footprint size (4x4 — use `ObjectEnum::Tunnel.object_type().size`)
3. Call `can_worker_place_structure(grid_x, grid_z, size_x, size_z, &tiles, &structures)`
4. If validation fails (`Err`): cancel the build — set `UnitCommand::Idle`, `BaseBehaviorState::None`, `LocomotionChannel::Stationary`, `OrientationChannel::Maintaining`, remove `BuildingTunnelBehavior`, log the rejection, and `continue`

**System parameter additions needed:**
The `build_tunnel_behavior_system` currently does not query tiles or structures. You need to add these query parameters:
- `tiles: Query<(&GridPosition, &TilePreset), With<Tile>>`
- `structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>`

These are the same types used by `BuildingStructureBehavior`'s system (see ~line 475 for the pattern).

**Reference implementation:** The `building_behavior_system` (~line 464) already does exactly this pattern — arrival check → `can_worker_place_structure()` → proceed or cancel. Follow the same pattern.

**Tests to add:**
- Test that arrival at a valid location proceeds to construction (existing tests may cover this)
- Test that arrival at a location with an unbuildable tile cancels the build (agent idles, behavior removed)
- Test that arrival at a location overlapping an existing structure cancels the build

## Technical Context

### File to modify
- `artifacts/developer/src/game/units/systems/behaviors.rs` — the only file that needs changes

### Function: `building_tunnel_behavior_system` (line 892)
Current signature (lines 893-910):
```rust
pub fn building_tunnel_behavior_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(Entity, &Transform, &Owner, &mut BuildingTunnelBehavior, &mut LocomotionChannel, &mut OrientationChannel, &mut UnitCommand, &mut BaseBehaviorState, &mut Visibility)>,
    tunnel_query: Query<(Entity, &Owner), With<TunnelState>>,
    tunnel_hp_query: Query<&ObjectInstance, With<ConstructionHP>>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
)
```

**Add two query parameters** after `syndicate_players`:
```rust
tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
structures: Query<(&GridPosition, &StructureInstance, &ObjectInstance)>,
```

### Insertion point (line 929)
The validation goes AFTER `if distance < BUILD_ARRIVAL_THRESHOLD {` (line 929) and BEFORE the single-agent enforcement check (line 932). Insert this block:
```rust
// Validate placement — tiles must be buildable and unoccupied
let (grid_x, grid_z) = world_to_grid(target, 1.0);
let (size_x, size_z) = ObjectEnum::Tunnel.object_type().size;
let valid = can_worker_place_structure(grid_x, grid_z, size_x, size_z, &tiles, &structures);
if valid.is_err() {
    info!("Agent: Build tunnel rejected — placement invalid at ({}, {}): {}", grid_x, grid_z, valid.unwrap_err());
    *command = UnitCommand::Idle;
    *behavior = BaseBehaviorState::None;
    *locomotion = LocomotionChannel::Stationary;
    *orientation = OrientationChannel::Maintaining;
    commands.entity(entity).remove::<BuildingTunnelBehavior>();
    continue;
}
```

### Imports already present (line 22)
```rust
use crate::game::world::utils::{can_worker_place_structure, world_to_grid};
```
All needed types (`GridPosition`, `TilePreset`, `Tile`, `StructureInstance`, `ObjectInstance`) are already imported at lines 16, 21, 23.

### Key detail: Tunnel size is (4, 4)
The task description says 1x1 — this is WRONG. `ObjectEnum::Tunnel.object_type().size` returns `(4, 4)` (see `artifacts/developer/src/game/types/objects.rs` line 290). Use `ObjectEnum::Tunnel.object_type().size` dynamically to get the correct value.

### Note: `ObjectEnum` import
`ObjectEnum` is used elsewhere in the file's test module (line 1059: `use crate::types::ObjectEnum;`). In the main function, it may need to be referenced as `crate::types::ObjectEnum` or added to an existing import line. Check what's already imported — the type `ObjectInstance` (from `crate::game::types`) is imported at line 16, but `ObjectEnum` (from `crate::types`) may need a separate import. Grep for existing `ObjectEnum` usage in the non-test portion to confirm.

### Cancellation pattern
The cancellation pattern (idle + remove behavior) is used 3 times already in this function:
- Line 937-944: location_taken rejection
- Line 950-956: no player rejection
- Line 977-984: insufficient funds rejection
Follow the exact same pattern.

### Test considerations — CRITICAL
**Existing arrival tests will break** after adding the `tiles` and `structures` query parameters. The system will now require entities matching those queries. Currently, tests like `building_tunnel_arrival_spawns_tunnel` (line 1613) don't spawn any tile entities, so `can_worker_place_structure` will return `Err("No tile at position")` and the tunnel will NOT be spawned.

**You must update all existing arrival tests** that expect the tunnel to be spawned (at minimum):
- `building_tunnel_arrival_spawns_tunnel` (line 1613)
- `building_tunnel_cost_deducted` (line 1644)
- `building_tunnel_cost_deducted_second_tunnel` (line 1663)
- `building_tunnel_insufficient_funds_cancels` (line 1688) — may still pass since it cancels before placement
- `building_tunnel_constructing_increments_frames` (line 1714) — starts in Constructing, won't be affected
- `building_tunnel_arrival_spawns_tunnel_entity` (line 1911)
- `building_tunnel_spawned_tunnel_has_construction_hp` (line 1935)
- `building_tunnel_spawned_tunnel_starts_at_10_percent_hp` (line 1957)
- `building_tunnel_rejects_second_agent_at_same_location` (line 1979)
- `building_tunnel_allows_agent_at_different_location` (line 2035)
- `building_tunnel_allows_after_first_agent_finishes` (line 2091)

For each test where the agent is at the target and expects the tunnel to spawn, add 4x4 buildable tiles at grid position (32, 32) (since `world_to_grid(Vec3(0.5, 0, 0.5), 1.0)` = (32, 32)). Follow the pattern from `building_behavior_arrived_valid_tiles_completes` (line 1400):
```rust
for dx in 0..4 {
    for dz in 0..4 {
        world.spawn((
            GridPosition { x: 32 + dx, z: 32 + dz },
            TilePreset {
                value: crate::game::world::types::TilePresetEnum::Plane,
                name: "Plane".to_string(),
                texture: None,
                buildable: true,
                traversible: true,
                rugged: false,
                drillable: false,
                recruitable: false,
            },
            Tile,
        ));
    }
}
```

Note: these tests use `App::new()` + `app.world_mut()`, not bare `World::new()`, so adapt the tile spawn accordingly (use `app.world_mut()` to spawn).

Consider creating a helper function like `spawn_buildable_tiles_4x4(world: &mut World, grid_x: i32, grid_z: i32)` to reduce boilerplate across all tests.

### New tests to add
1. **`building_tunnel_arrival_unbuildable_tile_cancels`**: Agent at target, tiles exist but one is `buildable: false` → agent idles, `BuildingTunnelBehavior` removed, no tunnel spawned
2. **`building_tunnel_arrival_structure_overlap_cancels`**: Agent at target, tiles are buildable, but an existing structure with `ObjectInstance` overlaps → agent idles, behavior removed
3. Optionally: **`building_tunnel_arrival_no_tiles_cancels`**: Agent at target, no tiles spawned → cancels (this may be implicitly tested by the existing tests before you fix them)

For the structure overlap test, spawn a structure entity with all 3 required components: `(GridPosition, StructureInstance, ObjectInstance)`. Use `ObjectInstance { object_type: ObjectEnum::Tunnel, ..default }` or similar. Check how `ObjectInstance` is constructed — see line 1479 area. Note: the existing `building_behavior_arrived_structure_overlap_cancels` test (line 1443) spawns only `GridPosition` + `StructureInstance` WITHOUT `ObjectInstance`, which means the structure won't match the query — this is likely a pre-existing bug in that test. For YOUR test, include all three components so the overlap is actually detected.

## Dependencies

None — this is a standalone validation addition to an existing system. All required functions (`can_worker_place_structure`, `world_to_grid`) and types are already imported in the file.
