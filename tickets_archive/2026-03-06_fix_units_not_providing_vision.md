# Ticket: Fix Units Not Providing Vision / Fog of War Reveal

## Current State
Only structures (e.g., the Deployment Center) provide fog of war vision. Units such as Peacekeepers move across the map without revealing any tiles around them, even though they have a defined SightRange (Peacekeeper SightRange: 5).

## Desired State
All owned Object Instances — both units and structures — provide fog of war vision based on their SightRange. When a unit moves, the fog of war updates to reveal tiles within its SightRange at its current position. Tiles transition from Unexplored to Visible when first entering a unit's SightRange, and from Visible to Explored when the unit moves away and no other friendly vision source covers them.

## Justification
Bug identified during QA testing (see forum topic `units_not_providing_vision.md`). Per `features/vision_system.md`, "Vision is provided by owned Object Instances (units and structures) based on their SightRange." Units are explicitly included as vision sources. The current implementation only queries structures for vision calculation, omitting units entirely.

## QA Steps
1. Start a new game. Note the fog of war around the Deployment Center.
2. Spawn a Peacekeeper unit.
3. Issue a move command to send the Peacekeeper into an Unexplored (fully black) area of the map.
4. As the Peacekeeper moves, verify that tiles within its SightRange (5 tiles) are revealed in real time around it.
5. After the Peacekeeper has moved away from its starting position, verify that tiles it previously revealed but are no longer in any friendly unit's or structure's SightRange transition to Explored state (terrain visible, enemy units hidden).
6. Move the Peacekeeper back to a previously Explored area and verify tiles return to Visible state.
7. Spawn a second unit type (if available) and verify it also provides vision based on its own SightRange.

## Expected Experience
- When a Peacekeeper moves across the map, a circle of visibility moves with it, revealing terrain, structures, and enemy units in real time.
- The player can scout the map by sending units into unexplored territory.
- Previously scouted areas that are no longer covered by any friendly vision source appear dimmed (Explored state) with terrain and last-known structures visible but enemy units hidden.
- The vision radius around each unit is consistent with its SightRange stat.
