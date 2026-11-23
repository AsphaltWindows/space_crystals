# Map: Tiles and Resources

## Tile
Non-selectable visible entity which makes up the game's map. Tile properties are defined by TilePresets. Any combination of properties is valid — the default presets represent common terrain types, but custom presets with arbitrary property combinations and textures can be created via the map editor.

### Selectable - false

### Buildable - boolean
### Traversible - boolean
### Rugged - boolean
### Drillable - boolean
### Recruitable - boolean

## TilePreset
A named tile configuration with a texture and specific property values. The texture visually communicates the tile's gameplay properties. The default presets are the standard terrain types, but map designers can define custom presets with any combination of properties and their own textures.

### Value - TilePresetEnum
### Name - string
### Texture - asset
### Buildable - boolean
### Traversible - boolean
### Rugged - boolean
### Drillable - boolean
### Recruitable - boolean

## DefaultTilePresets

### Plane
- Buildable: true
- Traversible: true
- Rugged: false
- Drillable: true
- Recruitable: true

### Rugged Terrain
- Buildable: false
- Traversible: true
- Rugged: true
- Drillable: true
- Recruitable: true

### Cliff
- Buildable: false
- Traversible: false
- Rugged: false
- Drillable: true
- Recruitable: true

### Mountain
- Buildable: false
- Traversible: false
- Rugged: false
- Drillable: false
- Recruitable: true

### Water
- Buildable: false
- Traversible: false
- Rugged: false
- Drillable: false
- Recruitable: false

## TilePlacement
An instance of a TilePreset placed on the map at a specific location. Its height comes from the map's corner heights (see Elevation below), not from the placement itself.

### Type - TilePresetEnum
### Location - Coordinates

## Elevation
Terrain height is stored at the **corners** of tiles, not per tile (decided 2026-11-23; replaces the earlier per-tile Elevation). A map of W x H tiles has (W+1) x (H+1) corner heights. Adjacent tiles share their corners, so the terrain surface is continuous: hills and slopes form wherever corner heights differ.

### CornerHeight - integer (0 to 16)

### Flat and sloped tiles
- **Flat tile**: all four corners have the same height. That height is the tile's level.
- **Sloped tile**: its corners differ. Units can walk across sloped Traversible tiles. Whether slopes change movement speed is not decided yet (TBD; until then they don't).

### Maximum slope
- On a **Traversible** tile, the four corner heights may differ by **at most 1 level**. Every walkable slope is therefore a gentle hill.
- Larger height differences are only allowed on **non-Traversible** tiles (e.g. Cliff). Sharp drops are made with cliffs, and **chokepoints come from cliffs combined with elevation differences** (user's intent).
- The map editor enforces this rule.

### Terrain height at a point
The terrain height at any point is interpolated from the corners of the tile containing it. (The interpolation method is a technical choice; it must be continuous across tile edges.)

### Elevation level
An object's **elevation level** is the terrain height at its position, **rounded down** to an integer. This is the elevation used by core/vision.md > ElevationModifier. A unit walking up a slope changes level at the point where the terrain height crosses the next integer.

### Structures only on flat terrain
Structures can only be placed on **flat** tiles. Every tile of a surface structure's footprint must be flat, in addition to being Buildable (core/entities.md > Placement Validation). Adjacent flat tiles share corners, so a connected footprint of flat tiles is always all at one level. Hills stay open ground; bases sit on flat areas.

### Display
See core/art_direction.md > Elevation Display.

## Resource
An Object Type representing a map resource node. Resources are indestructible, unowned, and provide no vision. They exist as contested map features that factions interact with through their own faction-specific mechanics.

### Destructible - false
### Owner - None (always)
### SightRange - 0

## SpaceCrystalsPatch
A minable deposit of Space Crystals on the map. Factions harvest Space Crystals from patches using faction-specific collection mechanics. When a patch is fully depleted, it disappears from the map.

### Size - 1x1
### RemainingAmount - number (Space Crystals left in the patch)
### InfoPanel: when visible to the selecting player, displays RemainingAmount

## SupplyDeliveryStation
A location on the map that periodically receives supply deliveries. The delivery countdown begins only once the station's current supply content reaches 0. Factions collect supplies from the station using faction-specific collection mechanics.

### Size - 2x2
### DeliverySize - number (amount of supplies per delivery)
### DeliveryInterval - number (time between deliveries)
### CurrentSupplies - number (supplies currently on the station)
### NextDeliveryTime - number (frames until the next delivery arrives)

### InfoPanel:
- **Visible** (any player): displays DeliverySize, DeliveryInterval, NextDeliveryTime, CurrentSupplies
- **Explored** (GDO player only): displays DeliverySize, DeliveryInterval, NextDeliveryTime (CurrentSupplies is hidden)
- **Explored** (non-GDO player) / **Unexplored**: not selectable, no InfoPanel
