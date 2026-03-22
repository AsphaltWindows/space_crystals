# Camera

## Zoom

Fixed zoom level. The camera displays exactly 28 GridUnits horizontally. No zoom in or zoom out controls are available.

The number of GridUnits visible vertically is determined by the screen's aspect ratio minus the vertical space occupied by the HUD (top resource bar + bottom panel).

## Panning

The camera pans via standard pan (currently implemented as instant snap to the target position). When the camera centers on a location — such as via Alt-click portrait in the SelectionPanel — it snaps instantly so the target is at the center of the viewport.

## Starting Position

Each player slot on a map can have an explicit starting camera position. At game start, the camera centers on:

1. The **map-defined starting camera position** for the local player's slot, if one is set.
2. Otherwise, the local player's **primary structure** — Deployment Center for GDO, starting Tunnel for Syndicate.

## HUD Layout

- **Top**: Resource bar
- **Bottom**: Info/Selection/Command panel
