# cults_game_start_fixes

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-add_cults_armory_r1.md

## Task

Fix the Cults faction game start so that playing as Cults is functional. Three bugs must be addressed:

### 1. Camera not centering on Cults start (BLOCKER)

In `center_camera_on_start`, the match arm for the primary structure type has `_ => return` which skips Cults and Colonists entirely. Add `FactionEnum::TheCults => ObjectEnum::RecruitmentCenter` to the match.

### 2. Enemy test units overlap Cults starting position (BLOCKER)

In `setup_enemy_test_units`, enemy Peacekeepers are hardcoded at grid positions around (50, 50) — the exact same location as the Cults Recruitment Center. Make enemy positions faction-aware.

### 3. No Space Crystal Patches near Cults start

In `spawn_space_crystal_patches`, all crystal patches near a faction start are clustered around the GDO position (30,30). Add 2-3 patches near (50,50).

### Testing

- Start a game as Cults — camera should center on the Recruitment Center
- No enemy units should overlap the Recruitment Center
- Crystal patches should be visible near the Cults start
- Existing GDO and Syndicate starts should still work correctly
- Run `cargo test` — existing tests should pass; add a Cults camera test

## Technical Context

### File 1: `artifacts/developer/src/game/world/faction/setup.rs`

**center_camera_on_start** (line 244-280):
- Line 263-267: The match on `selected_faction.0` maps GDO→DeploymentCenter, Syndicate→Tunnel, then `_ => return`. This is the bug — Cults is caught by the wildcard.
- Fix: Add `FactionEnum::TheCults => ObjectEnum::RecruitmentCenter` before the wildcard. Keep `_ => return` for Colonists (no structure yet).
- The system queries `(&ObjectInstance, &Owner, &Transform)` and matches on `obj.object_type == primary_type && *owner == local_owner`.
- The Recruitment Center IS spawned with `ObjectInstance` and correct `Owner` by `setup_cults_game_start` (line 189-223), so the query will find it.
- System ordering already correct: `center_camera_on_start` runs `.after(faction::setup_cults_game_start)` (mod.rs line 81).

**setup_enemy_test_units** (line 283-316):
- Lines 293-299: Hardcoded positions array at (50,50), (51,49), (49,51), (52,50), (50,52).
- Line 292: `selected` resource IS available but currently unused (`let _ = &selected`).
- Fix: Use `selected.0` to choose positions. When `FactionEnum::TheCults`, use positions near GDO start (~30,30) instead, e.g. (30,33), (31,32), (29,34), (32,33), (30,35). For all other factions, keep the current (50,50) positions.
- Pattern to follow: The same `grid_to_spawn_position` + `registry.spawn` + `insert_entity_visuals` pipeline used in the existing code.

**Tests** (line 318+):
- Existing tests: `test_center_camera_on_start_gdo` (line 451), `test_center_camera_on_start_syndicate` (line 479), `test_center_camera_on_start_no_structure` (line 507).
- Add a new `test_center_camera_on_start_cults` test following the same pattern as the GDO test:
  - Create TestApp, insert `SelectedFaction(FactionEnum::TheCults)`, `LocalPlayer(0)`, `MapStartingPositions::default()`
  - Spawn a RecruitmentCenter entity with `ObjectInstance::indestructible(ObjectEnum::RecruitmentCenter)`, `Owner::player(0)`, and a Transform at the expected Cults position
  - Run `center_camera_on_start` and verify camera moved to the RC position
- The no_structure test (line 507) uses `FactionEnum::TheCults` and expects camera unmoved — this test should STILL pass since no RC is spawned in it.

### File 2: `artifacts/developer/src/game/world/resources.rs`

**spawn_space_crystal_patches** (line 11-76):
- Line 18-31: `scp_data` array defines all patch positions and amounts.
- Add 2-3 entries near (50,50) with comments like `// Near Cults starting position (50,50)`:
  - `((48, 52), 2500)` — SW of RC
  - `((52, 48), 2000)` — NE of RC  
  - `((54, 50), 2500)` — East of RC
- Follow exactly the existing tuple format `((grid_x, grid_z), amount)`.
- No code changes needed beyond adding entries to the array — the spawning loop handles them automatically.

### Key Types Referenced
- `FactionEnum`: enum in `shared/types.rs` — `GlobalDefenseOrdinance`, `TheSyndicate`, `TheCults`, `Colonists`
- `ObjectEnum`: enum in `shared/types.rs` — includes `RecruitmentCenter`, `DeploymentCenter`, `Tunnel`, `Peacekeeper`
- `ObjectInstance`: component in `game/types.rs` — stores `object_type: ObjectEnum`, has `::indestructible()` constructor
- `Owner`: component in `shared/types.rs` — `Owner::player(n)`, `Owner::neutral()`
- `SelectedFaction`: resource wrapping `FactionEnum`
- `LocalPlayer`: resource wrapping player slot `u8`
- `MapStartingPositions`: resource in `game/world/types.rs` — `HashMap<u8, (u32, u32)>`

### Coordinate Reference
- GDO starts at grid (30, 30) — DC spawned there
- Syndicate starts near GDO area (Tunnel near 30,30 area)
- Cults starts at grid (50, 50) — RC spawned there
- Grid is 64x64, world coords: `world_x = (grid_x - 32) + 0.5`

## Dependencies

None — this is a standalone bugfix task. All three fixes are in existing files and don't require new systems or components.
