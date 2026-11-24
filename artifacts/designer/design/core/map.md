# Map: Tiles and Resources

## Tile
Non-selectable visible entity which makes up the game's map. Tile properties are defined by TilePresets. Any combination of properties is valid — the default presets represent common terrain types, but custom presets with arbitrary property combinations can be created via the map editor (custom textures: planned for the future; see core/map_editor.md > Custom Tile Presets).

### Selectable - false

### Buildable - boolean
### Traversible - boolean
### Rugged - boolean
### Drillable - boolean
### Recruitable - boolean

## TilePreset
A named tile configuration with a texture and specific property values. The texture visually communicates the tile's gameplay properties. The default presets are the standard terrain types, but map designers can define custom presets with any combination of properties (for now using existing textures; custom textures are planned, see core/map_editor.md > Custom Tile Presets).

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
- **Sloped tile**: its corners differ. Units can walk across sloped Traversible tiles. **Slopes don't change movement speed**, going up or down (user, 2026-11-23: no slowdowns for now; elevation matters through ElevationModifier and cliffs). May be revisited after playtesting.

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

**Exception: Syndicate underground expansions** (inside a Tunnel Area) may be placed on sloped tiles; the flatness rule doesn't apply to them (user, 2026-11-23). Their surface-related visuals on slopes are an art detail. Surface Syndicate structures (Tunnel, Laser Turret) follow the normal rule.

### Map resources only on flat terrain
**Space Crystals patches and Supply Delivery Stations** must also sit on **flat** tiles: every footprint tile flat (user, 2026-11-23, Director question Q0006). The map editor refuses them on slopes and map validation flags them. This keeps every patch usable by every faction (GDO Extraction Plates are structures and need flat ground) and keeps stations' building-like sprites level.

### Display
See core/art_direction.md > Elevation Display.

## Resource
An Object Type representing a map resource node. Resources are indestructible, unowned, and provide no vision. They exist as contested map features that factions interact with through their own faction-specific mechanics.

### Destructible - false
### Owner - None (always)
### SightRange - 0

## SpaceCrystalsPatch
A minable deposit of Space Crystals on the map. Factions harvest Space Crystals from patches using faction-specific collection mechanics. When a patch is fully depleted, it disappears from the map. Exception: a depleted patch under a GDO Extraction Plate stays until the plate is destroyed (factions/gdo/economy.md > ExtractionPlate).

### Size - 1x1
### RemainingAmount - number (Space Crystals left in the patch)
### StartingAmount - number, **set per patch in the map editor**; default for newly placed patches **1500** (placeholder, user 2026-11-23). RemainingAmount starts at this value.
### InfoPanel: when visible to the selecting player, displays RemainingAmount

## SupplyDeliveryStation
A location on the map that periodically receives supply deliveries. The delivery countdown begins only once the station's current supply content reaches 0. Factions collect supplies from the station using faction-specific collection mechanics.

### Size - 2x2
All three of the following are **set per station in the map editor** (user, 2026-11-23). The defaults for newly placed stations are placeholders:

### StartingSupplies - number (CurrentSupplies at game start); default **0**. A station starting at 0 begins its first countdown at game start, so the first delivery arrives one DeliveryInterval in. A stocked station starts counting down only once it is emptied (rule above).
### DeliverySize - number (amount of supplies per delivery); default **4**
### DeliveryInterval - number (time between deliveries); default **60 s (960 frames)**
### CurrentSupplies - number (supplies currently on the station)
### NextDeliveryTime - number (frames until the next delivery arrives)

### InfoPanel:
- **Visible** (any player): displays DeliverySize, DeliveryInterval, NextDeliveryTime, CurrentSupplies
- **Explored** (GDO player only): displays DeliverySize, DeliveryInterval and the **live, true** NextDeliveryTime (CurrentSupplies is hidden). NextDeliveryTime is not frozen at the last sighting and not extrapolated: the GDO always knows the real delivery schedule (core/vision.md > Explored Selectability). When the station is not empty, no countdown is running, and the panel shows that no delivery is scheduled. Intended side effect (user, 2026-11-23): a GDO player watching a fogged station sees its countdown start the moment someone empties it. That intel is deliberate.
- **Explored** (non-GDO player) / **Unexplored**: not selectable, no InfoPanel
