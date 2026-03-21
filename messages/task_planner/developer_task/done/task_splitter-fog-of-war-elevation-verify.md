# fog-of-war-elevation-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-fog-of-war-elevation.md

## Task

Verify that the fog of war vision system and elevation modifier are fully implemented and match the design spec. This feature appears to already be fully implemented in the codebase. The developer should verify all components are present and working:

1. **FogOfWarMap resource** (game/world/types.rs): Per-player visibility map with VisibilityStateEnum (Unexplored/Explored/Visible), get/set methods, tiles_in_sight_range with Euclidean distance.

2. **update_fog_of_war system** (game/world/map.rs): Recalculates visibility each tick from SightRange+Owner entities, transitions Visible→Explored with LastKnownStructures snapshots, properly handles multi-tile structure vision centers.

3. **apply_fog_rendering system** (game/world/map.rs): Hides enemy units on non-Visible tiles, adjusts tile colors (Unexplored=0.1, Explored=0.5, Visible=1.0).

4. **apply_structure_fog_rendering system** (game/world/map.rs): Hides structures on Unexplored tiles, shows on Explored (last-known state), always shows own/neutral structures.

5. **LastKnownStructures resource** (game/world/types.rs): Tracks structure snapshots (object_type, hp_fraction) per (player_id, x, z).

6. **ElevationMap resource** (game/world/types.rs): Populated from tile placements in spawn_grid.

7. **elevation_modifier function** (game/world/types.rs): Returns +1/-1/0 based on relative elevation, air exempt, underground uses surface elevation, binary (any difference = modifier).

8. **Elevation integrated into combat**: Used in combat systems (core.rs, behaviors.rs) for attack range modification.

If all components are present and correctly implemented per the design doc, add a brief confirmation comment to the code and ensure tests pass. If any gaps are found, implement the missing pieces.
