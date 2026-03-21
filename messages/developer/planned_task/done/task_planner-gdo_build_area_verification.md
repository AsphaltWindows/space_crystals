# gdo-build-area-verification

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

This is a verification task. All code is already implemented. The developer should inspect the following files and confirm correctness:

### File 1: `artifacts/developer/src/game/world/types.rs` (lines 196-227)
- **GdoBuildArea** resource: `pub struct GdoBuildArea { pub cells: HashSet<(i32, i32)> }`
- Methods: `contains(x, z) -> bool`, `overlaps_footprint(pos_x, pos_z, size_x, size_z) -> bool`
- Registered as resource in `artifacts/developer/src/game/world/mod.rs:19` via `.insert_resource(types::GdoBuildArea::default())`
- CONFIRMED: matches spec item 1.

### File 2: `artifacts/developer/src/game/world/utils.rs` (lines 36-59)
- `expand_build_area(build_area, building_pos_x, building_pos_z, building_size_x, building_size_z, extension)`
- Uses nested loops: for each cell in building footprint, adds all cells within Chebyshev distance `extension`
- CONFIRMED: matches spec item 2.

### File 3: `artifacts/developer/src/game/world/utils.rs` (lines 244-321)
- `can_place_building(pos_x, pos_z, size_x, size_z, object_type, build_area, tiles, structures, patches, fog_map, player_id) -> Result<(), &'static str>`
- Checks: (1) overlaps_footprint with build_area, (2) footprint_is_visible via fog_map, (3) ExtractionPlate special case (on patch, no existing plate), (4) all tiles Buildable, (5) no structure overlap
- CONFIRMED: matches spec item 3.

### File 4: `artifacts/developer/src/game/world/faction.rs` (lines 66-92)
- `setup_gdo_game_start()` spawns DC at grid (30,30) and calls `expand_build_area(&mut build_area, dc_grid_x, dc_grid_z, 4, 4, 12)`
- NOTE: Task says "spawn_deployment_center_system" but actual function is `setup_gdo_game_start()`. The DC is 4x4, extension=12.
- CONFIRMED: matches spec item 4.

### File 5: `artifacts/developer/src/game/world/faction.rs` (lines 1378-1459)
- `placement_click_system()` handles placement for each building type:
  - PowerPlant (line 1394): `expand_build_area(..., rot_x, rot_z, 1)` — extension=1 CONFIRMED
  - Barracks (line 1401): `expand_build_area(..., rot_x, rot_z, 2)` — extension=2 CONFIRMED
  - ExtractionFacility (line 1408): `expand_build_area(..., rot_x, rot_z, 2)` — extension=2 CONFIRMED
  - SupplyTower (line 1424): `expand_build_area(..., rot_x, rot_z, 1)` — extension=1 CONFIRMED
  - ExtractionPlate (line 1453): `expand_build_area(..., 1, 1, 0)` — extension=0 CONFIRMED
- All use `rotated_building_size()` for rotation-aware footprints (except ExtractionPlate which is always 1x1).
- CONFIRMED: matches spec item 5.

### File 6: `artifacts/developer/src/game/world/faction.rs` (lines 1538-1615)
- `manage_build_area_overlay()`: spawns green overlay `Color::srgba(0.2, 0.8, 0.2, 0.3)` when entering AwaitingPlacement (GDO path at line 1594)
- Also handles tunnel overlay (purple, 0.6, 0.2, 0.8, 0.3) and agent placement (no overlay)
- Despawns all `BuildAreaOverlay` entities when exiting placement mode (line 1609-1613)
- CONFIRMED: matches spec item 6.

### File 7: `artifacts/developer/src/game/world/faction.rs` (lines 1295-1304)
- Ghost tinting in `update_ghost_position_system()`: `placement_state.is_valid` is set based on `can_place_building()` result
- Valid: green `Color::srgba(0.2, 0.8, 0.2, 0.5)`, Invalid: red `Color::srgba(0.8, 0.2, 0.2, 0.5)`
- `PlacementState` struct at `artifacts/developer/src/ui/types.rs:329` has `pub is_valid: bool`
- CONFIRMED: matches spec item 7.

### Test execution
- Run `cargo test` from `artifacts/developer/` to confirm all tests pass.
- Note: there are NO dedicated unit tests for `expand_build_area()`, `can_place_building()`, or `GdoBuildArea` methods. The only placement-related tests in utils.rs (line 921, 938) are for `can_worker_place_structure()` (Syndicate-specific). This is a gap but not a spec mismatch.
- Existing tests in utils.rs are for screen_space_hit_test, world_to_grid, grid_to_world, box-selection.

### Summary
All 7 spec items are confirmed implemented correctly. The developer should:
1. Read the files listed above to confirm the analysis
2. Run `cargo test` to verify all tests pass
3. If everything checks out, complete with no code changes

## Dependencies

None — this is a standalone verification task with no code changes expected.
