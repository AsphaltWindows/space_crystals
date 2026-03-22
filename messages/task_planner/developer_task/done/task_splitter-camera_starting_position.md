# camera_starting_position

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-camera_starting_position.md

## Task

Add a startup system that centers the camera on the local player's primary structure at game start.

**Context:**
- Camera spawns at (0, 40, 25) looking at origin in `setup()` (main.rs ~line 71)
- GDO's Deployment Center spawns at grid (30, 30) in `setup_gdo_game_start` (faction.rs ~line 117-122)
- Syndicate's Tunnel spawns at grid (40, 40) in `setup_syndicate_game_start` (faction.rs ~line 145-152)
- `LocalPlayer` resource (types.rs) identifies which player is local (player 0)
- `SelectedFaction` resource determines which faction the local player chose
- Camera snap centering pattern already exists: set camera Transform x/z to target position, keep y=40, adjust z with offset formula (cam.y * 25.0 / 40.0) — see `selection_portrait_click_system` in hud.rs and control group recall in resources.rs

**Implementation:**
1. Create a new startup system `center_camera_on_start` that runs in Startup schedule AFTER all `setup_*_game_start` functions
2. Query the local player's owner ID from `LocalPlayer` resource
3. Query for the primary structure: ObjectEnum::DeploymentCenter or ObjectEnum::Tunnel owned by the local player (use Owner component match)
4. Read that entity's Transform position
5. Set the MainCamera Transform to center on that position using the same snap formula used elsewhere (x = target.x, z = target.z + z_offset, y = 40.0, looking_at target)
6. Register the system in mod.rs Startup schedule with appropriate ordering constraint (after faction game start systems)

**Key files:**
- `src/main.rs` — camera setup + MainCamera component spawn
- `src/game/world/faction.rs` — game start functions, LocalPlayer, SelectedFaction
- `src/game/world/mod.rs` — system registration
- `src/shared/types.rs` — MainCamera, LocalPlayer
- `src/ui/hud.rs` — reference for camera snap pattern (~line 1134)
- `src/game/world/resources.rs` — reference for camera snap pattern (~line 659)
