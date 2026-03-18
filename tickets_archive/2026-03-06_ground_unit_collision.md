# Ticket: Ground Unit Hard Collision

## Current State
Ground units have no collision with each other or with structures. Units freely overlap and pass through one another and through buildings. Pathfinding does not account for unit positions as obstacles.

## Desired State
Ground units (Domain=Ground) use hard collision based on their Silhouette rectangle:
- Ground units cannot overlap with other ground units or with structures
- Collision is hard — no overlap allowed, no push/displacement of idle units
- Idle units do not move aside for other units (they are static obstacles)
- Moving units must pathfind around occupied space (other units + structures)
- Collision detection uses the unit's Silhouette (2D rectangle in space units), oriented by the unit's Rotation

Note: Underground units (DrillUnit) are explicitly out of scope — their collision model is unspecified (see open questions in `features/unit_system.md`).

## Justification
Per `features/unit_system.md` (UnitCollision > Ground Collision section): ground units are solid obstacles defined by their Silhouette rectangle with hard collision. Moving units must pathfind around occupied space. This is a core movement system requirement — without it, units stack on top of each other and pass through buildings.

## QA Steps
1. Spawn two Peacekeeper units near each other
2. Order one unit to move through the position of the other idle unit
3. Verify the moving unit pathfinds around the idle unit rather than passing through it
4. Verify the idle unit does not move or get pushed aside
5. Order a unit to move through a structure (e.g., Power Plant)
6. Verify the unit pathfinds around the structure
7. Spawn multiple units in a tight group and issue a move command to one
8. Verify the unit finds a path out of the group without overlapping
9. Attempt to move a unit into a space too narrow for its Silhouette
10. Verify the unit stops or reroutes rather than squeezing through

## Expected Experience
- Units never visually overlap with other ground units or structures
- When ordered to move through an occupied area, units smoothly navigate around obstacles
- Idle units remain stationary when other units approach — they do not get bumped or displaced
- Movement feels natural and responsive despite collision constraints
- In tight formations, units may take longer paths to avoid each other but never clip through
