# Faction Module Split

## Date: 2026-03-24
## Status: Architecture request pending
## Priority: 3 (parallel with behaviors split)
## Parent: codebase-refactoring-priorities.md

## Problem Statement

`src/game/world/faction.rs` is 4,387 lines. It combines faction setup, production tick systems for five structure types, construction systems, placement ghost management, build area overlays, rally point management, resource display, and power grid computation. The production systems are heavily duplicated -- five systems follow nearly identical tick-check-spawn-rally patterns with only the structure component type and resource type differing. Beyond duplication, the file mixes concerns that are logically independent: placement UI logic has no relationship to production ticks.

## Current Structure Analysis

### 1. Setup Systems (lines 19-285, ~265 lines)
- `setup_player_resources()` - creates player and resource entities
- `spawn_faction_and_player()` - helper
- `setup_gdo_game_start()` - spawns GDO starting structures
- `setup_syndicate_game_start()` - spawns Syndicate starting structures
- `setup_cults_game_start()` - spawns Cults starting structures
- `setup_colonists_game_start()` - spawns Colonists starting structures
- `center_camera_on_start()` - centers camera on player's starting position
- `setup_enemy_test_units()` - debug unit spawning

### 2. Power Grid (lines 286-318, ~35 lines)
- `compute_power_grid()` - computes power ratio per player from power plants

### 3. GDO Production and Construction (lines 319-627, ~310 lines)
- `dc_construction_tick_system()` - Deployment Center construction progress
- `barracks_production_tick_system()` - Barracks unit production (~75 lines)
- `issue_rally_command()` - shared helper for post-spawn rally (~80 lines)
- `headquarters_production_tick_system()` - HQ unit production (~120 lines)

### 4. Resource/Rally Systems (lines 628-944, ~315 lines)
- `extraction_plate_mining_system()` - extraction plate resource collection
- `depleted_patch_despawn_system()` - removes depleted crystal patches
- `rally_target_cleanup_system()` - cleans up rally targets for despawned entities
- `production_rally_point_system()` - processes rally point commands (~120 lines)
- `compute_rally_target_from_click()` - determines rally target from mouse click
- `spawn_or_update_rally_marker()` - creates/updates rally marker visuals
- `despawn_rally_marker_for()` - removes rally markers

### 5. EF and Construction (lines 945-1174, ~230 lines)
- `ef_construction_tick_system()` - Extraction Facility construction
- `construction_hp_tick_system()` - agent-built structures HP tick
- `debug_construction_input()` - debug keybinding (~140 lines)
- `get_power_ratio_for_owner()` - helper

### 6. Resource Display (lines 1175-1224, ~50 lines)
- `display_resources_system()` - updates resource UI text

### 7. Placement Systems (lines 1225-1828, ~600 lines)
- `manage_placement_ghost()` - creates/destroys placement ghost entity (~100 lines)
- `update_placement_ghost()` - moves ghost with cursor, validates placement (~150 lines)
- `placement_click_system()` - handles click to place structure (~270 lines)
- `manage_build_area_overlay()` - shows/hides GDO build area overlay (~80 lines)

### 8. Build Area Helpers (lines 1829-1960, ~130 lines)
- `has_structure_overlap()` - collision detection for placement
- `build_area_mesh()` - generates GDO build area mesh
- `tunnel_area_mesh()` - generates tunnel area mesh

### 9. Syndicate/Cults Production (lines 1961-2585, ~625 lines)
- `tunnel_construction_tick_system()` - tunnel construction progress (~90 lines)
- `ejection_tick_system()` - tunnel unit ejection (~70 lines)
- `supply_tower_production_tick_system()` - supply tower production (~85 lines)
- `recruitment_tile_claiming_system()` - cults tile claiming (~90 lines)
- `recruitment_center_production_system()` - recruitment center production (~80 lines)
- `cults_unit_control_aggregation_system()` - cults unit count tracking
- `cults_construction_tick_system()` - cults construction progress
- `cults_construction_cancel_system()` - cults construction cancellation
- `armory_exit_side_position()` - helper
- `armory_training_tick_system()` - armory unit training (~55 lines)
- `armory_eject_tick_system()` - armory unit ejection

### 10. Tests (lines 2586-4387, ~1,801 lines)

## Proposed Refactoring

Convert `faction.rs` into a `faction/` directory module with these submodules:

### New Directory Structure

```
src/game/world/faction/
    mod.rs              -- re-exports all public systems and helpers
    setup.rs            -- setup_player_resources, spawn_faction_and_player, all setup_*_game_start, center_camera_on_start, setup_enemy_test_units
    power.rs            -- compute_power_grid, get_power_ratio_for_owner
    production.rs       -- barracks_production_tick, headquarters_production_tick, supply_tower_production_tick, recruitment_center_production, armory_training_tick, issue_rally_command
    construction.rs     -- dc_construction_tick, ef_construction_tick, construction_hp_tick, tunnel_construction_tick, cults_construction_tick, cults_construction_cancel, debug_construction_input
    resources.rs        -- extraction_plate_mining, depleted_patch_despawn, display_resources
    rally.rs            -- rally_target_cleanup, production_rally_point, compute_rally_target_from_click, spawn_or_update_rally_marker, despawn_rally_marker_for
    placement.rs        -- manage_placement_ghost, update_placement_ghost, placement_click, manage_build_area_overlay, has_structure_overlap, build_area_mesh, tunnel_area_mesh
    ejection.rs         -- ejection_tick, armory_eject_tick, armory_exit_side_position, recruitment_tile_claiming, cults_unit_control_aggregation
```

### Key Decisions

1. **Production systems consolidated**: All five production tick systems (`barracks`, `headquarters`, `supply_tower`, `recruitment_center`, `armory_training`) go into `production.rs` with `issue_rally_command`. This puts the duplicated code in one file, making the duplication visible and setting up future deduplication via a trait or generic. The file will be ~450 lines of logic -- large but manageable, and the duplication will be obvious to address in a follow-up.

2. **Construction vs production separation**: Construction systems (DC, EF, tunnel, cults, HP tick) are distinct from production systems (they build structures, not units). They go in `construction.rs`.

3. **Placement gets its own file**: Placement ghost management, click handling, and build area overlays form a cohesive UI-adjacent subsystem that is ~730 lines including helpers. It interacts with cursor state and UI resources, making it logically distinct from production/construction.

4. **Rally point systems together**: Rally markers, cleanup, and the production rally point system are tightly coupled. They share helpers like `compute_rally_target_from_click` and `spawn_or_update_rally_marker`.

5. **Ejection and tile systems together**: Tunnel ejection, armory ejection, recruitment tile claiming, and cults unit control aggregation are all "unit lifecycle within structures" operations. They form a small cohesive group.

6. **Future production deduplication**: Once production systems are isolated in `production.rs`, a follow-up refactor can introduce a `ProductionStructure` trait and generic tick system. This split is a prerequisite for that work but does not attempt it -- keeping this refactor as a pure reorganization with no behavioral changes.

## Scope

### Files Created
- `src/game/world/faction/mod.rs`
- `src/game/world/faction/setup.rs`
- `src/game/world/faction/power.rs`
- `src/game/world/faction/production.rs`
- `src/game/world/faction/construction.rs`
- `src/game/world/faction/resources.rs`
- `src/game/world/faction/rally.rs`
- `src/game/world/faction/placement.rs`
- `src/game/world/faction/ejection.rs`

### Files Deleted
- `src/game/world/faction.rs` (replaced by directory module)

### Files Modified
- `src/game/world/mod.rs` -- `pub mod faction;` still works (resolves to faction/mod.rs)
- Any external references to `faction::spawn_or_update_rally_marker`, `faction::despawn_rally_marker_for` etc. continue to work via re-exports from `faction/mod.rs`

### Expected Line Counts (approximate)
- setup.rs: ~265 lines + ~200 lines tests = ~465
- power.rs: ~50 lines + ~50 lines tests = ~100
- production.rs: ~450 lines + ~400 lines tests = ~850
- construction.rs: ~370 lines + ~300 lines tests = ~670
- resources.rs: ~90 lines + ~100 lines tests = ~190
- rally.rs: ~315 lines + ~250 lines tests = ~565
- placement.rs: ~730 lines + ~300 lines tests = ~1,030
- ejection.rs: ~315 lines + ~200 lines tests = ~515
- mod.rs: ~50 lines

No file exceeds ~1,050 lines.

## Constraints

- Behavioral equivalence: identical gameplay before and after
- `spawn_or_update_rally_marker` and `despawn_rally_marker_for` are `pub` and imported by `core.rs` -- must remain accessible via `faction::spawn_or_update_rally_marker` path
- System registration in `world/mod.rs` must reference the same system functions via re-exports
- All unit and integration tests must pass
- No deduplication in this refactor -- pure file reorganization

## Success Metrics

- `faction.rs` replaced by 10 files, none exceeding ~1,050 lines
- Each submodule has a clear single responsibility
- Production system duplication is contained in one file, ready for future trait extraction
- Placement logic is isolated from game logic
- Adding a new faction's setup requires editing only `setup.rs`
