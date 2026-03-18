# Ticket: Fog of War Visibility States

## Current State
No fog of war system exists. There is no per-tile, per-player visibility tracking and no mechanism for Object Instances to provide vision via SightRange.

## Desired State
Every tile on the map has a VisibilityState per player, stored as a VisibilityStateEnum with three values:

- **Unexplored**: Tile has never been within SightRange of any of the player's owned Object Instances. Fully black — terrain, structures, and units are all hidden.
- **Explored**: Tile was previously visible but no longer is. Terrain is shown. Structures are shown in their last-known state. Enemy units are NOT shown.
- **Visible**: Tile is currently within SightRange of one of the player's owned Object Instances. Everything shown in real time.

Vision is provided by owned Object Instances (units and structures) based on their SightRange property (already defined in the entity system). The system must:
1. Track which tiles are within SightRange of each player's owned objects.
2. Transition tiles from Unexplored to Visible when first seen, from Visible to Explored when no longer in range, and back to Visible when re-entered.
3. For Explored tiles, remember the last-known state of structures on those tiles.

## Justification
Defined in `features/vision_system.md` (VisibilityState, Vision Source sections). Fog of war is a core RTS mechanic required for competitive play. SightRange is already specified on Object Types in `features/entity_system.md`.

## QA Steps
1. Place a player-owned unit on the map with a SightRange of 3.
2. Verify that tiles within 3 tiles of the unit are in Visible state.
3. Verify that tiles beyond 3 tiles that the unit has never been near are in Unexplored state and render as fully black (no terrain, structures, or units shown).
4. Move the unit away from its original position so previously visible tiles leave its SightRange.
5. Verify those tiles transition to Explored state: terrain is shown, any structures appear in their last-known state, but enemy units on those tiles are hidden.
6. Move the unit back so those tiles re-enter its SightRange.
7. Verify tiles transition back to Visible state with real-time information.
8. Place a second player's unit and verify that each player's visibility state is tracked independently (one player's vision does not affect the other's).

## Expected Experience
- Tiles the player has never scouted appear as solid black.
- When a unit moves into new territory, tiles light up revealing terrain, structures, and units in real time.
- When the unit moves away, the revealed tiles dim (indicating Explored) — terrain and structures remain visible but enemy units vanish.
- Returning to those tiles restores full real-time visibility.
- Each player sees only what their own units reveal; the other player's fog is independent.
