# camera-pan-snap

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# camera-pan-snap

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added camera panning behavior to `artifacts/designer/design/camera.md`.

The camera uses standard pan, currently implemented as instant snap. When the camera centers on a location (e.g., Alt-click portrait in SelectionPanel), it snaps instantly so the target is at the center of the viewport. This applies to all camera-centering actions.

**Change**: Added new "Panning" section to camera.md between Zoom and HUD Layout, defining snap behavior for camera centering.

## QA Instructions

1. Have multiple units spread across the map.
2. Select a group of units so the SelectionPanel shows portraits.
3. Alt-click a portrait of a unit that is off-screen.
4. **Expected**: The camera instantly snaps to center on that unit. No smooth scrolling animation — it should be immediate.
5. Verify the selection does not change (same units remain selected).
6. Verify the unit from the Alt-clicked portrait is now visible and centered in the viewport.
