# cults_game_start_fixes

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-add_cults_armory_r1.md

## Task

Fix the Cults faction game start so that playing as Cults is functional. Three bugs must be addressed in `src/game/world/faction/setup.rs`:

### 1. Camera not centering on Cults start (BLOCKER)

In `center_camera_on_start` (~line 263), the match arm for the primary structure type has `_ => return` which skips Cults and Colonists entirely. The camera stays at its default position while the Recruitment Center is at grid (50, 50), so the player sees an empty map.

**Fix**: Add `FactionEnum::TheCults => ObjectEnum::RecruitmentCenter` to the match. Optionally add Colonists too if they have a primary structure (currently they don't — stub is fine).

### 2. Enemy test units overlap Cults starting position (BLOCKER)

In `setup_enemy_test_units` (~line 293), enemy Peacekeepers are hardcoded at grid positions around (50, 50) — the exact same location as the Cults Recruitment Center. When playing as Cults, the enemy spawns right on top of your starting building.

**Fix**: Make enemy positions faction-aware. When Cults is selected (local player at grid ~50,50), spawn enemy test units near the GDO start area (~30,30) instead. When GDO/Syndicate is selected, the current (50,50) positions are fine since they're away from those starts.

### 3. No Space Crystal Patches near Cults start

In `src/game/world/resources.rs` `spawn_space_crystal_patches` (~line 18), all crystal patches near a faction start are clustered around the GDO position (30,30). None are within easy reach of grid (50,50).

**Fix**: Add 2-3 crystal patches near the Cults starting position, e.g., ((48, 52), 2500), ((52, 48), 2000), ((54, 50), 2500). Follow the existing pattern in the `scp_data` array.

### Testing

- Start a game as Cults — camera should center on the Recruitment Center
- No enemy units should overlap the Recruitment Center
- Crystal patches should be visible near the Cults start
- Existing GDO and Syndicate starts should still work correctly
- Run `cargo test` — existing tests in setup.rs should still pass (update test_center_camera_on_start_no_structure or add a new Cults camera test)
