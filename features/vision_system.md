# Feature: Vision System

## Overview
Fog of war controlling what each player can see, with three visibility states per tile per player, and elevation-based modifiers.

## Design Sources
- `design/entities.md` (Vision, VisibilityState, ElevationModifier)

## Specifications

### VisibilityState (per tile, per player)
- **Unexplored**: Tile never within SightRange of any player unit/structure. Fully black — terrain, structures, and units hidden.
- **Explored**: Tile was previously visible but is not currently. Terrain shown. Structures shown in last-known state. Enemy units NOT shown.
- **Visible**: Tile is currently within SightRange of a player's unit/structure. Everything shown in real time.

### Vision Source
- Vision is provided by owned Object Instances (units and structures) based on their SightRange.

### ElevationModifier
- Applies to sight range AND attack range.
- Only applies when BOTH source and target are ground or underground units.
- **Higher ground**: +1 to sight range and attack range against lower-elevation targets.
- **Lower ground**: -1 to sight range and attack range against higher-elevation targets.
- **Equal elevation**: no modifier.
- **Binary**: any elevation difference triggers the modifier regardless of gap size.
- **Air exempt**: air units ignore elevation modifiers entirely, both as source and target.
- **Underground units**: use the elevation of the terrain above them.

## Dependencies
- `entity_system` (Object Instances provide vision via SightRange)
- `tile_system` (tiles have elevation, visibility state is per-tile)
