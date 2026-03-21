# fog-of-war-elevation

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

## Content

Implement the fog of war vision system and elevation modifier as defined in `artifacts/designer/design/entities.md` under Vision, VisibilityState, and ElevationModifier.

**Vision System:**
Every tile on the map has one of three visibility states per player. Vision is provided by owned Object Instances (units and structures) based on their SightRange.

**VisibilityState (per tile, per player):**
- **Unexplored**: Tile has never been in SightRange of any of the player's units/structures. Fully black — terrain, structures, and units all hidden.
- **Explored**: Previously in SightRange but not currently. Terrain is shown. Structures shown in last-known state (may have changed). Enemy units NOT shown.
- **Visible**: Currently in SightRange of the player's units/structures. Everything shown in real time.

**ElevationModifier:**
Sight range and attack range are modified by relative elevation between source and target. Only applies when both source and target are ground or underground units.
- **Higher ground**: +1 to sight range and attack range against lower-elevation targets
- **Lower ground**: -1 to sight range and attack range against higher-elevation targets
- **Equal elevation**: no modifier
- **Binary**: ANY elevation difference triggers the modifier regardless of gap size
- **Air exempt**: Air units ignore elevation modifiers entirely, both as source and target
- **Underground units**: Use the elevation of the terrain above them

## QA Instructions

1. Start a game — verify the entire map starts as Unexplored (black) except around the player's starting units/structures.
2. Move a unit across the map — verify tiles transition from Unexplored to Visible as the unit approaches (based on SightRange).
3. Move the unit away — verify previously-seen tiles transition to Explored (terrain visible, enemy units hidden, structures shown in last-known state).
4. Destroy an enemy structure while it's Explored (not Visible) — verify the structure's ghost remains visible in its last-known state until the tile becomes Visible again.
5. Place a unit on high ground and an enemy on low ground — verify the high-ground unit gains +1 sight range and +1 attack range.
6. Reverse the positions — verify the low-ground unit loses 1 sight range and 1 attack range against the high-ground target.
7. Test with an air unit — verify elevation modifiers do NOT apply (neither as attacker nor target).
8. Test with an underground unit — verify it uses the elevation of the terrain above it for modifier calculations.
