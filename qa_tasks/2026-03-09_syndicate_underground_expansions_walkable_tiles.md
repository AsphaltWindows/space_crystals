# Developer Task: Syndicate Underground Expansions Must Not Block Surface Movement

## Summary
Underground Syndicate expansions (HQ and future expansion types) incorrectly mark their tile footprints as impassable on the surface. The `rebuild_occupancy_map` system treats all structures identically — it has no domain filtering. This traps units (especially the starting Agent) on tiles occupied by underground expansions, blocking all Syndicate gameplay.

## Original Ticket
Underground expansion placement should not alter the traversability of surface tiles. Surface units must walk freely over underground expansion footprints. Only the Tunnel itself (a surface structure) should occupy tiles.

## Technical Context

### Root Cause
`rebuild_occupancy_map()` at `src/game/units/systems/core.rs:1019-1060` marks ALL structure footprints as blocked, with no domain filtering. Compare with unit handling (lines 1027-1031) which correctly skips non-Ground units via `DomainEnum`.

The structures query at line 1022:
```rust
structures: Query<(&GridPosition, &ObjectInstance), With<StructureInstance>>,
```
Has no access to `DomainEnum`. All structures get their tiles inserted into both `blocked_tiles` and `structure_tiles` (lines 1050-1058).

### The Fix

#### 1. Add `DomainEnum` to the structures query (`src/game/units/systems/core.rs:1022`)

Change:
```rust
structures: Query<(&GridPosition, &ObjectInstance), With<StructureInstance>>,
```
To:
```rust
structures: Query<(&GridPosition, &ObjectInstance, Option<&DomainEnum>), With<StructureInstance>>,
```

#### 2. Filter underground structures in the loop (`src/game/units/systems/core.rs:1049-1059`)

Change:
```rust
// Mark structures
for (grid_pos, obj_instance) in &structures {
    let (size_w, size_h) = obj_instance.object_type.object_type().size;
    for dx in 0..size_w as i32 {
        for dz in 0..size_h as i32 {
            let tile = (grid_pos.x + dx, grid_pos.z + dz);
            occupancy.blocked_tiles.insert(tile);
            occupancy.structure_tiles.insert(tile);
        }
    }
}
```
To:
```rust
// Mark structures (skip underground — they don't block surface movement)
for (grid_pos, obj_instance, domain_opt) in &structures {
    let is_underground = domain_opt.map_or(false, |d| *d == DomainEnum::Underground);
    if is_underground {
        continue;
    }
    let (size_w, size_h) = obj_instance.object_type.object_type().size;
    for dx in 0..size_w as i32 {
        for dz in 0..size_h as i32 {
            let tile = (grid_pos.x + dx, grid_pos.z + dz);
            occupancy.blocked_tiles.insert(tile);
            occupancy.structure_tiles.insert(tile);
        }
    }
}
```

### Why This Is Sufficient
- HQ is already tagged `DomainEnum::Underground` at spawn (`src/game/utils.rs:799`)
- `TunnelExpansionMarker` confirms the structure is an expansion, but `DomainEnum` is the correct filter — it's the domain concept that determines surface pathability
- The Tunnel itself (surface structure) has NO `DomainEnum` component, so `domain_opt` is `None` → `map_or(false, ...)` → not underground → still blocks. Correct behavior.
- Future underground expansions spawned with `DomainEnum::Underground` will automatically inherit this fix

### Other Systems — No Changes Needed
- **`find_path()`** at `src/game/units/pathfinding.rs:84` — uses `OccupancyMap` as input. Fix upstream (in `rebuild_occupancy_map`) means pathfinding automatically sees the corrected data.
- **`can_place_building()`** at `src/game/world/utils.rs:244` — GDO surface placement; doesn't apply to underground expansions.
- **`has_structure_overlap()`** at `src/game/world/faction.rs:1576` — used for underground expansion placement validation within the tunnel area. This should still check ALL structures (including other underground ones) to prevent expansions overlapping each other. No change needed.
- **`can_worker_place_structure()`** at `src/game/world/utils.rs:328` — worker placement on surface. No change needed.
- **Collision layer (`ground_bodies`)** — only populated for units, not structures. No change needed.

### Key Types
- `DomainEnum` at `src/shared/types.rs:377` — `Ground`, `Air`, `Underground`
- `OccupancyMap` at `src/game/units/types/types.rs:82` — `blocked_tiles: HashSet<(i32, i32)>`, `structure_tiles: HashSet<(i32, i32)>`
- `TunnelExpansionMarker` at `src/game/types/structures.rs:247` — links expansion to parent tunnel
- `StructureInstance` at `src/game/types/objects.rs` — marker component for all structures

### Test Strategy
Existing test file: `tests/qa/tunnel_expansions_and_starting_condition.rs`

Add a test that:
1. Spawns a Tunnel + HQ (underground expansion) using the test harness
2. Runs `rebuild_occupancy_map` (via `step()`)
3. Asserts that HQ footprint tiles are NOT in `occupancy.blocked_tiles` and NOT in `occupancy.structure_tiles`
4. Asserts that Tunnel footprint tiles ARE in both sets
5. Optionally: verifies `find_path()` can route through HQ footprint tiles

Use `spawn_headquarters_at_grid()` from the test harness at `src/shared/testing/harness.rs:179`.

### Import Required
`DomainEnum` is already imported in `src/game/units/systems/core.rs` — it's used at line 1011 for unit air/ground checks. No new imports needed.

## Dependencies
None. This is a standalone bug fix. No other developer tasks need to be completed first.

## QA Steps
1. [human] Start a game as Syndicate. Locate the starting Tunnel and its underground HQ. Verify that the tiles above the HQ's 2x2 footprint are walkable by selecting any surface unit and right-clicking on those tiles.
2. [human] Produce an Agent from the HQ with a surface rally point. Verify the Agent ejects from Side A and successfully moves to the rally point without getting stuck.
3. [human] Order a unit to pathfind across the HQ's underground footprint (walk from one side to the other). Verify the unit paths through without detour or blockage.
4. [human] Build a second underground expansion (e.g., Barracks) in the Tunnel Area. Verify its footprint tiles are also walkable on the surface.
5. [human] Verify that the Tunnel structure itself (4x4 surface building) still correctly blocks surface movement — only the underground expansions should be walkable.

## Expected Experience
- Step 1: Clicking on tiles above the underground HQ shows them as valid move targets. The unit moves to those tiles without issue.
- Step 2: After production completes, the Agent emerges from Side A of the Tunnel and pathfinds smoothly to the rally point. No stuck/frozen units.
- Step 3: The unit walks in a straight (or reasonable) path across the HQ's underground footprint, treating those tiles as normal terrain.
- Step 4: After the new expansion is built, surface tiles above it remain traversable. No new blocked tiles appear on the surface.
- Step 5: Attempting to move a unit through the Tunnel's own 4x4 footprint causes the unit to path around it, as expected for a surface structure.
