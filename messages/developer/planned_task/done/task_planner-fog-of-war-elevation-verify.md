# fog-of-war-elevation-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Verified Components (all present and functional)

**FogOfWarMap** — `artifacts/developer/src/game/world/types.rs` lines 303-375
- `FogOfWarMap` resource with `width`, `height`, `player_maps: HashMap<u8, Vec<VisibilityStateEnum>>`
- Methods: `new()`, `ensure_player()`, `get()`, `set()`, `tiles_in_sight_range()`
- `tiles_in_sight_range()` uses Euclidean distance (dx*dx + dz*dz <= range*range), clamped to grid bounds
- 12+ unit tests covering bounds, clamping, circular shape, state transitions (lines 877-1017)

**update_fog_of_war** — `artifacts/developer/src/game/world/map.rs` lines 285-355
- Queries: `vision_sources: Query<(&SightRange, &GridPosition, &Owner, Option<&ObjectInstance>)>`
- Uses `vision_center()` from `world/utils.rs` to offset multi-tile structures (filters out unit sizes correctly)
- Collects visible tile sets per player, transitions Visible→Explored with LastKnownStructures snapshots
- Clears last-known entries when tiles become Visible again
- 8 unit tests for vision center filtering (lines 454-551)

**apply_fog_rendering** — `artifacts/developer/src/game/world/map.rs` lines 361-412
- Hides enemy units on non-Visible tiles (`Visibility::Hidden`)
- Adjusts tile material colors: Unexplored=0.1, Explored=0.5, Visible=1.0 multiplier

**apply_structure_fog_rendering** — `artifacts/developer/src/game/world/map.rs` lines 417-444
- Hides enemy structures on Unexplored tiles
- Shows enemy structures on Explored tiles (last-known state via `Visibility::Inherited`)
- Skips own structures and neutral structures (always visible)

**LastKnownStructures** — `artifacts/developer/src/game/world/types.rs` lines 376-389
- `LastKnownStructure` struct: `object_type: ObjectEnum`, `hp_fraction: f32`
- `LastKnownStructures` resource: `entries: HashMap<(u8, i32, i32), LastKnownStructure>`
- 3 unit tests (lines 1019-1054)

**ElevationMap** — `artifacts/developer/src/game/world/types.rs` lines 229-254
- `elevations: HashMap<(i32, i32), u8>`, `get()` returns 0 for missing
- Populated in `spawn_grid()` (map.rs line 124): all tiles currently elevation 0
- 3 unit tests (lines 743-762)

**elevation_modifier** — `artifacts/developer/src/game/world/types.rs` lines 261-278
- Returns +1 (higher), -1 (lower), 0 (equal or air exempt)
- Air domain check: returns 0 if either source or target is `DomainEnum::Air`
- Underground uses surface elevation (caller passes surface elevation via ElevationMap lookup)
- Binary comparison via `cmp()`
- 9 unit tests covering all cases (lines 764-817)

**Elevation in combat** — `artifacts/developer/src/game/combat/systems/core.rs` lines 7, 76-81, 297-302, 400-405
- Imported and used in 3 combat systems: melee check (`is_melee() => 0`), ranged turret fire, projectile systems
- Pattern: `let elev_mod = if attack_cap.is_melee() { 0 } else { elevation_modifier(...) };`
- `effective_range = attack_cap.range + elev_mod as f32`
- Also used in `artifacts/developer/src/game/combat/systems/behaviors.rs` lines 62, 145, 402

### POTENTIAL GAP: Elevation modifier not applied to sight range

The design spec (`artifacts/designer/design/entities.md` lines 132-140) says:
> "Higher ground: +1 to **sight range** and attack range against lower-elevation targets"

However, `update_fog_of_war` (map.rs line 306) calls `fog_map.tiles_in_sight_range(cx, cz, sight_range.0)` with the raw sight range value — it does NOT apply elevation modifiers to vision. The elevation modifier is only applied in combat systems.

**NOTE**: Applying elevation to sight range is conceptually different from combat — in fog of war, there is no "target" to compare elevation against. The design may intend that units on higher ground simply see farther in all directions. If so, the implementation would need to:
1. Look up the source unit's elevation from `ElevationMap`
2. For each candidate tile, compare elevations
3. Include/exclude the tile based on whether `sight_range + elevation_modifier` reaches it

This is complex and may be working-as-intended if the design considers elevation-modified sight range out of scope for the fog system. The developer should:
- **First**: Verify all 8 checklist items pass as-is (they likely do)
- **Second**: Note the sight range gap as a finding — if the current behavior is intentional, document it with a comment; if not, implement per-tile elevation-aware vision

### Running tests
- `cargo test -p space_crystals` — runs all tests including the 30+ fog/elevation tests
- Key test files: `game/world/types.rs` (ElevationMap, FogOfWarMap, LastKnownStructures, elevation_modifier tests), `game/world/map.rs` (vision center tests)

## Dependencies

None — this is a standalone verification task. All fog-of-war and elevation systems are already implemented.
