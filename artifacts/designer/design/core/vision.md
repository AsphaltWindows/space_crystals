# Vision

## Vision
The fog of war system that controls what each player can see on the map. Every tile on the map has one of three visibility states per player. Vision is provided by owned Object Instances (units and structures) based on their SightRange.

## VisibilityState
The visibility of a tile from the perspective of a specific player.

### Value - VisibilityStateEnum (Unexplored | Explored | Visible)

## Unexplored
The tile has never been within the SightRange of any of the player's units or structures. The tile is fully black — terrain, structures, and units are all hidden.

## Explored
The tile was previously within the SightRange of one of the player's units or structures, but is not currently. Terrain is shown. Structures and resources are shown in their last-known state (may have been destroyed or changed since). Enemy units are not shown.

### Explored Selectability
Objects in Explored areas are **not selectable**, with one exception:
- **SupplyDeliveryStation**: Selectable by GDO players only. The GDO has a lore/strategic relationship with supply stations — supplies are intended for GDO forces, and the SupplyTower scheduled delivery mechanic requires planning around delivery timing even without active vision. Non-GDO players cannot select SupplyDeliveryStations in Explored areas.

## Visible
The tile is currently within the SightRange of one of the player's units or structures. Everything on the tile is shown in real time — terrain, structures, and units.

## ElevationModifier
Sight range and attack range are modified by relative elevation between the source and target. This modifier only applies when both the source and target are ground or underground units. Air units are exempt from elevation modifiers in both directions (neither gain nor suffer them).

### Higher ground: +1 to sight range and attack range against lower-elevation targets
### Lower ground: -1 to sight range and attack range against higher-elevation targets
### Equal elevation: no modifier
### Binary: any elevation difference triggers the modifier regardless of the size of the gap
### Air exempt: air units ignore elevation modifiers entirely, both as source and target
### Underground units: use the elevation of the terrain above them
