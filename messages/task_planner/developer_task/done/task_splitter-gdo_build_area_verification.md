# gdo-build-area-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-gdo-build-area.md

## Task

**Verification task — the GDO Build Area system is already fully implemented.**

Verify the following existing implementation matches the design spec:

1. **GdoBuildArea resource** (game/world/types.rs): HashSet<(i32, i32)> storing all valid build cells. Methods: contains(), overlaps_footprint().

2. **expand_build_area()** (game/world/utils.rs): Expands build area using Chebyshev distance from each cell of a placed building's footprint.

3. **can_place_building()** (game/world/utils.rs): Validates placement — at least 1 cell in build area, all tiles Visible, all tiles Buildable, no structure overlap, ExtractionPlate special case (must be on patch without existing plate).

4. **DC seeding** (game/world/faction.rs spawn_deployment_center_system): Seeds build area with extension=12 from DC position.

5. **Build area expansion on placement** (game/world/faction.rs handle_placement_click):
   - PowerPlant: extension=1
   - Barracks: extension=2
   - ExtractionFacility: extension=2
   - SupplyTower: extension=1
   - ExtractionPlate: extension=0

6. **Visual overlay** (game/world/faction.rs manage_build_area_overlay): Semi-transparent green overlay (0.2, 0.8, 0.2, 0.3) spawned during AwaitingPlacement modes, despawned on exit.

7. **Ghost tinting**: PlacementState.is_valid drives green/red ghost coloring.

Run `cargo test` to confirm all existing tests pass. If any spec mismatch is found, fix it. If everything matches, this task is complete with no code changes needed.
