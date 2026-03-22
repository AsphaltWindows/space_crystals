# camera_starting_position_map_override

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# camera_starting_position_map_override

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Updated `artifacts/designer/design/camera.md` — the Starting Position section now supports **map-defined starting camera positions per player slot**. Each player slot on a map can have an explicit starting camera position. At game start, the camera uses the map-defined position if set, otherwise falls back to centering on the player's primary structure (Deployment Center for GDO, starting Tunnel for Syndicate).

## QA Instructions

1. Load a map that has explicit starting camera positions defined for each player slot. Verify the camera starts at the map-defined position, not on the primary structure.
2. Load a map where no starting camera position is set for the local player's slot. Verify the camera falls back to centering on the primary structure (DC for GDO, Tunnel for Syndicate).
3. On a multiplayer map, verify that each player's camera independently starts at their own slot's defined position (or their own fallback structure).
