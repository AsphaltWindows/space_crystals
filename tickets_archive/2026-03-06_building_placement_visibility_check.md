# Ticket: Add Visibility Check to Building Placement Validation

## Current State
The GDO building placement system (AwaitingPlacement flow) validates spatial constraints only: the building must overlap the GDO build area, must not overlap existing structures, and (for Extraction Plates) must be on a Space Crystal Patch. There is no check for the player's fog of war visibility state on the target tiles. Players can place buildings on Unexplored or Explored tiles they cannot currently see.

## Desired State
Building placement on the surface map requires **all tiles under the building footprint** to be in the **Visible** state for the placing player. If any tile is Unexplored or Explored (not currently visible), placement is rejected. The AwaitingPlacement ghost preview should show red tinting when the footprint overlaps non-Visible tiles, consistent with other invalid placement feedback. This applies to all GDO surface building placement (Power Plant, Barracks, Supply Tower, Extraction Plate, etc.). Syndicate underground placement (Tunnel Area) is exempt.

## Justification
Reported during QA testing. Building in fog of war is not standard RTS behavior and undermines the vision system's purpose. Designer confirmed intent in forum topic `forum/building_placement_in_fog_of_war.md`. Vision system spec (`features/vision_system.md`) defines three visibility states per tile; placement validation in `features/gdo_objects.md` (GDOBuildArea rules) should incorporate visibility as an additional constraint. Formal design doc update pending from designer.

## QA Steps
1. Start a new game as GDO. Observe the deployment center and its revealed vision area.
2. Note the GDO build area extends beyond the visible area in some directions (build radius 12 vs sight range 6).
3. Select the Deployment Center, enter Build Menu, select Power Plant.
4. Move the placement ghost to a tile within the build area but covered by fog of war (Unexplored or Explored).
5. Verify the ghost shows red tinting (invalid placement).
6. Attempt to left-click to place. Verify the placement is rejected (building is not placed).
7. Move the placement ghost to a tile within the build area that is Visible.
8. Verify the ghost shows green tinting (valid placement).
9. Left-click to place. Verify the building is placed successfully.
10. Repeat steps 3-9 with a different building type (e.g., Barracks) to confirm consistency.

## Expected Experience
- When hovering the building ghost over fogged/unexplored tiles within the build area, the ghost turns red, matching the visual feedback for other invalid placements (e.g., overlapping structures).
- Clicking on an invalid (non-Visible) location does nothing — the ghost remains active awaiting valid placement.
- When hovering over Visible tiles within the build area with no other conflicts, the ghost turns green and placement succeeds on click.
- The build area overlay is still shown (it does not change), but visibility adds a second layer of validation on top of spatial rules.
