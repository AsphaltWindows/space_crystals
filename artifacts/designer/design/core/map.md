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
An instance of a TilePreset placed on the map at a specific location and elevation.

### Type - TilePresetEnum
### Location - Coordinates
### Elevation - integer (0 to 16)

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
