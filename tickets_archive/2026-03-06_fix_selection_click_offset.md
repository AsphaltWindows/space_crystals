# Ticket: Fix Selection Click Offset — Must Click ~1.5 Tiles Above Unit

## Current State
Clicking on a unit's visual position does not select it. Players must click approximately 1.5 tiles above the unit's visible location to register a selection. The `click_to_select_system` in `src/game/world/resources.rs` casts a ray from the camera through the cursor and checks against entity `Transform.translation` positions using `SelectionBounds`. The most likely cause is that `Transform.translation` is at the unit's base (ground level) while the visual model extends upward — with an angled camera, this projects the clickable point above the visual center.

## Desired State
Clicking directly on a unit's visual center selects it. The selection raycast hit detection aligns with where units visually appear on screen, regardless of camera angle or zoom level. The clickable volume should encompass the unit's visible area.

## Justification
The control system feature (`features/control_system.md`) defines Selection as the core player interaction. If click-to-select is offset from the visual, the game is functionally unplayable — every action that begins with "select a unit" is unreliable. This bug also affects QA confidence in all downstream test results. Forum topic: `forum/selection_click_offset.md`.

## QA Steps
1. Spawn multiple units of different types on the map.
2. Click directly on a unit's visual center — verify the unit is selected (green highlight or selection indicator appears).
3. Click approximately 1.5 tiles above a unit (the old broken behavior) — verify this does NOT select the unit unless another unit is visually at that position.
4. Zoom in to maximum zoom. Click on a unit's visual center — verify selection registers correctly.
5. Zoom out to minimum zoom. Click on a unit's visual center — verify selection registers correctly.
6. Test with at least two different unit types (e.g., Peacekeeper and Agent if available) — verify offset is fixed for all types, not just one.
7. Test box/drag selection — verify the drag rectangle correctly captures units whose visuals are inside the rectangle.
8. Click near the edge of a unit's visual boundary — verify selection still registers (bounds are not too tight).
9. Click on empty ground away from any unit — verify nothing is selected (no false positives from overly large bounds).

## Expected Experience
Clicking anywhere on a unit's visible sprite/model immediately selects it. The selection highlight appears on the clicked unit. There is no need to aim above, below, or offset from the visual position. This behavior is consistent across all zoom levels and unit types. Box-select also correctly captures units whose visuals are within the drag rectangle.
