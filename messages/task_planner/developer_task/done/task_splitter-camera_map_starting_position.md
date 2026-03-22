# camera_map_starting_position

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-camera_starting_position_map_override.md

## Task

Add a `MapStartingPositions` resource (or equivalent) that stores an optional starting camera position (grid coordinates) per player slot index. Add a startup system (run after game start systems that spawn structures) that positions the camera at game start:

1. If `MapStartingPositions` has an entry for the local player's slot, center the camera on that grid position.
2. Otherwise, fall back to centering on the local player's primary structure — Deployment Center for GDO, starting Tunnel for Syndicate.
3. If neither exists (e.g., stub factions with no structures), leave the camera at its default position.

Implementation notes:
- The resource should be inserted during map loading / game setup. For now, it can default to empty (no map-defined positions), which triggers the fallback path.
- Use the existing camera snap formula: given a target grid position (x, z), set camera Transform to (x, cam.y, z + cam.y * 25.0 / 40.0) — this matches the snap logic in hud.rs (Alt-click portrait) and resources.rs (control group double-tap recall).
- Query `LocalPlayer` to determine which player slot is local, then query for `ObjectEnum::DeploymentCenter` or `ObjectEnum::Tunnel` (non-underground check or just first match) with matching `Owner`.
- The system should run in `Startup` schedule, ordered after `setup_gdo_game_start` / `setup_syndicate_game_start` so structures exist to query.
- Player slot index maps to Owner::player(N) — slot 0 is player 0, etc.
- The `MapStartingPositions` resource should be defined in game/world/types.rs alongside other map resources (GridMap, ElevationMap, FogOfWarMap).
