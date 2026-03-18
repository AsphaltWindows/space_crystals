# Ticket: Elevation Modifier for Sight and Attack Range

## Current State
No elevation-based modifier exists for sight range or attack range. Tiles have an Elevation property (0-16) defined in the tile system, but it does not yet affect gameplay calculations.

## Desired State
An ElevationModifier system that adjusts sight range and attack range based on relative elevation between source and target:

- **Higher ground**: +1 to sight range and attack range when the source is at higher elevation than the target.
- **Lower ground**: -1 to sight range and attack range when the source is at lower elevation than the target.
- **Equal elevation**: No modifier applied.
- **Binary**: The modifier is always exactly +1 or -1 regardless of the size of the elevation difference (e.g., elevation 2 vs elevation 10 still yields only +1/-1).
- **Applicability**: Only applies when BOTH source and target are ground or underground units/structures.
- **Air exempt**: Air units are exempt from elevation modifiers entirely — they neither gain nor suffer them, whether as source or target.
- **Underground units**: Use the elevation of the terrain tile above them (not their underground position) for modifier calculations.

This modifier must integrate with:
1. The fog of war system (affects effective SightRange for visibility calculations).
2. The combat system (affects effective attack range — to be implemented later).

## Justification
Defined in `features/vision_system.md` (ElevationModifier section). Elevation advantage is a core tactical element that rewards map control and positioning. Depends on tile elevation from `features/tile_system.md`.

## QA Steps
1. Place a ground unit with SightRange 4 on a tile at elevation 3.
2. Place an enemy ground unit on a tile at elevation 1, exactly 5 tiles away from the first unit.
3. Verify the higher-elevation unit can see the lower tile (effective SightRange = 4 + 1 = 5).
4. From the lower unit's perspective, verify it cannot see the higher tile (effective SightRange = 4 - 1 = 3, insufficient for 5 tiles).
5. Place both units at the same elevation, 5 tiles apart, each with SightRange 4.
6. Verify neither can see the other (no modifier, SightRange 4 < distance 5).
7. Place one unit at elevation 1 and another at elevation 15, same SightRange. Verify the modifier is still only +1/-1 (binary, not proportional to the gap).
8. Place an air unit at low elevation and a ground unit at high elevation. Verify the air unit's SightRange is NOT reduced (air exempt as target and source).
9. Place an underground unit on a tile whose surface elevation is 5. Place a ground unit on a tile at elevation 3. Verify the underground unit gets the +1 modifier (uses terrain elevation above it, which is 5 > 3).

## Expected Experience
- Units on hills or elevated terrain can see slightly farther when looking down at lower ground.
- Units on low ground have slightly reduced vision when looking up at higher terrain.
- The effect is subtle (always exactly 1 tile) but tactically significant for positioning.
- Air units behave consistently regardless of terrain elevation beneath them.
- The modifier is intuitive: high ground = advantage, low ground = disadvantage.
