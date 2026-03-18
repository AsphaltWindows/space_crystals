# Ticket: Fix Fog of War Centering on Deployment Center

## Current State
The fog of war reveal area at game start is not symmetrically centered on the deployment center. The visible area appears offset from the structure's position, resulting in uneven visibility around the player's starting building.

## Desired State
The fog of war initial reveal area is symmetrically centered on the deployment center's position. At game start, the tiles revealed by the deployment center's SightRange should extend equally in all directions from the structure's center point. The visibility calculation must use the correct center coordinates of the deployment center (accounting for any sprite offset, tile alignment, or anchor point differences).

## Justification
Bug identified during QA testing (see forum topic `fog_of_war_not_centered_on_deployment_center.md`). Per `features/vision_system.md`, vision is provided by owned Object Instances based on their SightRange. The deployment center is the initial vision source at game start, and its revealed area must be correctly centered for gameplay to function as designed.

## QA Steps
1. Start a new game. Do not spawn or move any units.
2. Observe the fog of war reveal area around the deployment center.
3. Count the number of visible tiles extending from the deployment center in each cardinal direction (north, south, east, west).
4. Verify that the visible tile count is equal in all four cardinal directions from the deployment center's center tile.
5. Verify that the reveal area also extends symmetrically in diagonal directions.
6. Repeat with a second player's deployment center to confirm both are correctly centered.

## Expected Experience
- At game start, the deployment center sits at the center of a symmetrical revealed area.
- The fog of war boundary is equidistant from the deployment center in all directions (within the constraints of the SightRange shape).
- No visible offset or bias in any direction — the deployment center is clearly the focal point of the initial revealed area.
