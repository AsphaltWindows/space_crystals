# rally_attack_approach_path

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-gdo_barracks_r1.md

## Task

Fix the `issue_rally_command` function in `artifacts/developer/src/game/world/faction.rs` (line ~425) so that when a produced unit is issued an AttackTarget command against an enemy rally target, it also receives a MoveTarget and Path for initial approach -- matching the pattern used in the right-click attack fix in `right_click_move_command` (core.rs, lines ~355-375).

Currently (line 470), the enemy branch only inserts `UnitCommand::AttackTarget(*target_entity)` without MoveTarget/Path, causing the unit to stop instead of approaching. The friendly branch (line 472-481) correctly computes and inserts a path.

The fix: in the `is_enemy` branch of `issue_rally_command`, after inserting `UnitCommand::AttackTarget`, also compute a path from the spawn position to the target and insert `MoveTarget(target_pos)` and `Path { waypoints: smoothed, current_waypoint: 0 }`.

This function is shared by Barracks (line 404), Headquarters (line 571), and Armory (line 2531), so fixing it once fixes rally-attack for all production structures.

Also add a test verifying that `issue_rally_command` with a RallyTarget::Object pointing to an enemy entity results in both `UnitCommand::AttackTarget` and `MoveTarget` being inserted on the unit.

## Technical Context

### File to change
- **`artifacts/developer/src/game/world/faction.rs`** -- the only file that needs modification.

### Exact fix location (lines 462-484)
The `issue_rally_command` function at line 425 has two branches inside `RallyTarget::Object`:
- **Friendly branch (lines 472-481)**: already correct -- computes path via `find_path_for_domain`, inserts `MoveTarget`, `Path`, and `UnitCommand::Move`.
- **Enemy branch (lines 468-470)**: broken -- only inserts `UnitCommand::AttackTarget(*target_entity)`. Needs the same path computation added.

### Pattern to follow (right-click attack fix, core.rs lines 355-375)
The fix in `right_click_move_command` shows the exact pattern:
```rust
// After inserting UnitCommand::AttackTarget:
let start_grid = world_to_grid(transform.translation);  // For rally: use spawn_grid (already available)
let target_grid = world_to_grid(target_world_pos);       // For rally: world_to_grid(target_pos)
if let Some(path) = find_path_for_domain(
    start_grid, target_grid, &tiles, unit_base,
    grid.width as i32, grid.height as i32,
    &occupancy, (start_grid.x, start_grid.z),
) {
    let smoothed = smooth_path(path);
    entity_cmds.insert((
        MoveTarget(target_world_pos),
        Path { waypoints: smoothed, current_waypoint: 0 },
    ));
}
```

### Concrete fix for the enemy branch
Replace lines 468-470:
```rust
if is_enemy {
    // Attack enemy rally target
    commands.entity(unit_entity).insert(UnitCommand::AttackTarget(*target_entity));
}
```
With:
```rust
if is_enemy {
    // Attack enemy rally target with approach path
    let target_grid = world_to_grid(target_pos);
    if let Some(path) = find_path_for_domain(spawn_grid, target_grid, tiles, unit_base, grid.width as i32, grid.height as i32, occupancy, (spawn_grid.x, spawn_grid.z)) {
        let smoothed = smooth_path(path);
        commands.entity(unit_entity).insert((
            UnitCommand::AttackTarget(*target_entity),
            MoveTarget(target_pos),
            Path { waypoints: smoothed, current_waypoint: 0 },
        ));
    } else {
        // Pathfinding failed -- still issue the attack command, combat behavior will handle approach
        commands.entity(unit_entity).insert(UnitCommand::AttackTarget(*target_entity));
    }
}
```

Note: Unlike the right-click handler which inserts UnitCommand and MoveTarget/Path separately (because the unit already exists with other components), here we can bundle them in one `.insert()` call. The `else` branch preserves the existing fallback behavior.

### Key types and imports (already in scope at line 438-441)
- `world_to_grid` -- from `crate::game::units::utils`
- `smooth_path` -- from `crate::game::units::utils`
- `find_path_for_domain` -- from `crate::game::units::pathfinding`
- `MoveTarget`, `Path` -- from `crate::game::units::types::movement`
- `UnitCommand` -- from `crate::game::units::types::state`
- `GridPosition` -- already in scope via `super::*`
- `spawn_grid` variable -- already computed at line 448

### Test guidance
Add a test in the existing `mod tests` block (starts at line 2576). Use the `setup_production_app()` helper (line 3563) for the App.

The test cannot easily call `issue_rally_command` directly (it takes Query parameters). Instead, use the barracks production system as a vehicle:
1. Use `setup_production_app()` to create the App
2. Spawn tiles covering the path area (needed for `find_path_for_domain` to succeed) -- use `Tile` marker + `GridPosition` + `TilePreset { preset: TilePresetEnum::Terrain }`
3. Spawn a barracks entity with `BarracksState` that has:
   - `rally_point: Some(RallyTarget::Object(enemy_entity))`
   - `current_build: Some(BuildableUnitEnum::Peacekeeper)`
   - `current_build_progress: Some(0)` (set to production_time - 1 so next tick completes)
4. Spawn an enemy entity with `Transform`, `Owner::player(1)`, `ObjectInstance::destructible(...)` at a known position
5. Spawn `GdoPlayerResources` + `Player` for player 0
6. Run `barracks_production_tick_system` enough times to complete production
7. Call `app.update()` to flush deferred commands
8. Query for the produced unit and assert it has both `UnitCommand::AttackTarget(enemy_entity)` AND `MoveTarget`

Key constants for barracks production:
- `PEACEKEEPER_PRODUCTION_FRAMES = 90` (check `gdo_structure_stats` module in structures.rs)
- `BarracksState` fields: `current_build`, `current_build_progress`, `rally_point`, `power_ratio`

### Call sites (no changes needed)
- Line 404: Barracks calls `issue_rally_command` -- automatically gets the fix
- Line 571: HQ calls `issue_rally_command` -- automatically gets the fix
- Line 2531: Armory calls `issue_rally_command` -- automatically gets the fix

## Dependencies

None. This is a self-contained fix to a single function. The function is already called from all production structures, so the fix propagates automatically.
