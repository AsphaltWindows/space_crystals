# Ticket: Air Unit Soft Separation

## Current State
Air units have no collision or separation behavior. They freely overlap with each other, with ground units, and with structures. Multiple air units can stack on the same position.

## Desired State
Air units (Domain=Air) use a soft separation model:
- Air units do **not** collide with ground units or structures — they pass freely over/through them
- Air units apply a gentle repulsion force against other air units to prevent stacking
- Each air unit type defines a **SeparationRadius** (circular, per unit type) — the distance at which air-to-air repulsion activates
- SeparationRadius must be larger than the unit's Silhouette
- The repulsion is soft — it discourages overlap but does not hard-block movement. Units can temporarily overlap when moving through each other but will drift apart when stationary or moving slowly
- Repulsion force should scale with proximity (stronger when closer, zero at SeparationRadius edge)

Note: Underground units (DrillUnit) are explicitly out of scope.

## Justification
Per `features/unit_system.md` (UnitCollision > Air Collision section): air units do not collide with ground units or structures, and use soft separation with other air units via a circular SeparationRadius. This prevents air unit stacking while maintaining the fluid, unrestricted feel appropriate for aerial movement.

## QA Steps
1. Spawn a HoverCraft (air unit) and move it over a structure
2. Verify the air unit passes freely over the structure with no collision
3. Spawn a HoverCraft and move it through a group of ground units
4. Verify the air unit passes freely through ground units with no collision
5. Spawn two HoverCraft units and move them to the same location
6. Verify they do not perfectly stack — a gentle drift should push them apart
7. Move one air unit directly through another air unit's position
8. Verify the units can temporarily overlap during movement but separate when both stop
9. Spawn 5+ air units and group-move them to one spot
10. Verify they spread out around the target rather than stacking, forming a loose cluster
11. Verify the separation force does not cause jittering or oscillation — units should settle smoothly

## Expected Experience
- Air units glide freely over terrain, structures, and ground units without any collision response
- When multiple air units are near each other, they gently spread out rather than piling up
- The separation is subtle and smooth — no sudden jolts, bouncing, or visible snapping
- Air units can be ordered to the same location and will distribute themselves around it naturally
- The overall feel is fluid and organic, distinct from the rigid collision of ground units
