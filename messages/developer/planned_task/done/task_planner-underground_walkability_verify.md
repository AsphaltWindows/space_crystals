# underground-walkability-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-underground-surface-walkability.md

## Task

Verify that underground tunnel expansions do NOT block surface unit movement. This functionality appears to already be fully implemented:

- `rebuild_occupancy_map` in `src/game/units/systems/core.rs` (around line 1092) already skips structures with `DomainEnum::Underground` when populating `blocked_tiles` and `structure_tiles`.
- `spawn_headquarters` in `src/game/utils.rs` assigns `DomainEnum::Underground` to the HQ expansion entity.

**What to do:**

1. Read the existing implementation and confirm it correctly handles this case.
2. If there is NOT already a test that verifies surface units can pathfind through tiles occupied by underground expansions, add one. The test should:
   - Spawn a Tunnel (surface structure) and an underground expansion (e.g., Headquarters) within its area.
   - Confirm the Tunnel's tiles are in `blocked_tiles` (surface structure blocks movement).
   - Confirm the HQ's tiles are NOT in `blocked_tiles` (underground expansion does not block).
3. If the implementation is already correct and tested, report completion with no code changes needed.

## Technical Context

### Implementation to verify

**File: `artifacts/developer/src/game/units/systems/core.rs`** (line 1062-1107)

The `rebuild_occupancy_map` system has two loops:
1. **Units loop** (line 1070-1090): Iterates units, skips non-Ground domain units (`is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground)`). Only Ground units get their tiles added to `blocked_tiles` and their collision bodies to `ground_bodies`.
2. **Structures loop** (line 1092-1106): Iterates structures with `StructureInstance` marker. On line 1094, checks `is_underground = domain_opt.map_or(false, |d| *d == DomainEnum::Underground)` and skips underground structures. Surface structures get all their tiles (based on `object_type.size`) added to both `blocked_tiles` and `structure_tiles`.

This correctly means underground expansions (HQ) do NOT block surface movement. The logic is sound.

**File: `artifacts/developer/src/game/utils.rs`** (line 797-814)

`spawn_headquarters` spawns the HQ entity with `DomainEnum::Underground` component (line 807), confirming underground expansions carry the correct domain marker.

### Where to add the test

**File: `artifacts/developer/src/game/units/systems/core.rs`** — in the `mod tests` block starting at line 1408.

There are NO existing tests for `rebuild_occupancy_map`. The `OccupancyMap` data structure has unit tests in `artifacts/developer/src/game/units/types/types.rs` (line 300-355), but those only test the map's API (insert/query/clear), not the system that populates it.

### Test pattern to follow

Existing tests in the same file (e.g., `grid_position_sync_updates_moved_unit` at line 1422) use:
```rust
use bevy::ecs::system::RunSystemOnce;  // already imported at line 1410

let mut world = World::new();
// spawn entities...
world.run_system_once(system_name).unwrap();
// assert results...
```

Since `rebuild_occupancy_map` takes `ResMut<OccupancyMap>`, you must init the resource first:
```rust
world.insert_resource(OccupancyMap::default());
```

### Key types and their locations

- **`OccupancyMap`**: `artifacts/developer/src/game/units/types/types.rs` line 82 — Resource with `blocked_tiles: HashSet<(i32,i32)>`, `structure_tiles: HashSet<(i32,i32)>`, `ground_bodies: Vec<CollisionBody>`
- **`ObjectInstance`**: `artifacts/developer/src/game/types/objects.rs` line 55 — needs `object_type: ObjectEnum` (for the `.object_type().size` lookup)
- **`StructureInstance`**: `artifacts/developer/src/game/types/objects.rs` line 128 — marker component, use `StructureInstance::default()`
- **`DomainEnum`**: `artifacts/developer/src/shared/types.rs` line 410 — `Ground`, `Underground`, etc. Imported via `use crate::types::*`
- **`ObjectEnum::Tunnel`**: size (4, 4) — surface structure
- **`ObjectEnum::Headquarters`**: size (2, 2) — underground expansion
- **`GridPosition`**: already available via imports, needs `x: i32, z: i32`

### Suggested test structure

Write at least two tests:
1. **`rebuild_occupancy_map_surface_structure_blocks_tiles`**: Spawn a surface Tunnel at grid (10, 10) with `StructureInstance::default()`, `ObjectInstance::destructible(ObjectEnum::Tunnel, 100.0)`, and a `GridPosition`. Run `rebuild_occupancy_map`. Assert tiles (10,10) through (13,13) are in `blocked_tiles` and `structure_tiles`.

2. **`rebuild_occupancy_map_underground_structure_does_not_block`**: Spawn an underground HQ at grid (10, 10) with `StructureInstance::default()`, `ObjectInstance::destructible(ObjectEnum::Headquarters, 100.0)`, `DomainEnum::Underground`, and a `GridPosition`. Run `rebuild_occupancy_map`. Assert tiles (10,10) through (11,11) are NOT in `blocked_tiles`.

3. **`rebuild_occupancy_map_surface_tunnel_blocks_but_underground_hq_does_not`** (combined): Spawn both a surface Tunnel and underground HQ at overlapping or nearby positions. Assert Tunnel tiles are blocked, HQ tiles (where they don't overlap with Tunnel) are not.

### Import notes

All necessary types are already imported via `super::*` in the test module — `OccupancyMap`, `ObjectInstance`, `StructureInstance`, `ObjectEnum`, `DomainEnum`, `GridPosition`, `Unit`, etc. The only test-specific import needed is `bevy::ecs::system::RunSystemOnce` which is already at line 1410.

## Dependencies

None — this is a standalone verification task. The implementation is already complete; this task only adds test coverage.
