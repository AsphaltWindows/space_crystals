# Entities

## Entity
A type or class of objects in the game

### Visible - boolean

## Invisible Entity
An Entity which has no direct visual representation on the map. An Abstract type class or category.

### Visible - false

## Faction
The Invisible Entity of in-game faction type has a faction-specific hud with displaying relevant information on the screen

### Value - FactionEnum
### Name - string
### DisplayHud[Value] - Display

## Player
The Invisible Entity of an in-game player. Each player has a number from 1 to n where n is the number of players in the game. Player has info and statistics relevant to their Faction, which are displayed in the Faction-specific DisplayHud.

### Name - string
### Faction - FactionEnum
### PlayerNumber - number
### DisplayHudInfo[Faction] - struct

## Visible Entity
An Entity which has an immediate visible representation on the screen.

### Visible - true

### Selectable - boolean

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

## Object Type
Selectable visible entity, instances of which exist in the game at some coordinates on the map. When selected by a player the InfoPanel is displayed for the object on that player's screen, depending on the player selecting and the player owning (if applicable) the object instance.

### Selectable - true

### Value - ObjectEnum
### Name - string
### Size - Height x Width
### InfoPanel - (owner: (PlayerNumber | None), seelctor: Player) -> Display
### Destructible - boolean
### SightRange - number (vision is provided to the owning player when the instance has an Owner)
### Groupable - boolean (if false, each instance is always in its own SelectionGroup even when selected with other instances of the same type)

## Vision
The fog of war system that controls what each player can see on the map. Every tile on the map has one of three visibility states per player. Vision is provided by owned Object Instances (units and structures) based on their SightRange.

## VisibilityState
The visibility of a tile from the perspective of a specific player.

### Value - VisibilityStateEnum (Unexplored | Explored | Visible)

## Unexplored
The tile has never been within the SightRange of any of the player's units or structures. The tile is fully black — terrain, structures, and units are all hidden.

## Explored
The tile was previously within the SightRange of one of the player's units or structures, but is not currently. Terrain is shown. Structures are shown in their last-known state (may have been destroyed or changed since). Enemy units are not shown.

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

## Structure Type
An Object Type of a building, an instance of which may appear on the game's map. During placement, the player can rotate the building in 90-degree increments and flip it across the horizontal or vertical axis. This gives up to 8 possible orientations for fully asymmetric buildings (ABCD), fewer for more symmetric types.

It has one of 7 symmetry types from the perspective of how relevant the building's orientation is. AAAA is the building is fully symmetrical on all 4 sides (requires for Height and Width to be equal) making the orientation irrelevant. Height and Width can be non-equal for symmetry types ABAB, ABAC and ABCD.

### SymmetryType - AAAA | AAAB | AABB | ABAB | AABC | ABAC | ABCD

### ConstructionHP Rule
Some structures are built on-site and start at 10% of their maximum HP, gaining HP linearly as construction progresses, reaching full HP when construction completes. HP during construction = MaxHP x (10% + 90% x construction_progress). A partially-built structure can be attacked and destroyed before completion. This rule applies only to structures that explicitly reference it.

## Placement Validation

Structure placement is validated differently depending on the placement method:

### Direct Placement (e.g., GDO buildings, Tunnel underground expansions)
- The building is placed immediately from an interface menu
- All tiles under the footprint must pass validation at the moment the player confirms placement
- For surface buildings: all footprint tiles must be in the **Visible** state for the placing player. If any tile is not Visible, the placement ghost is shown in red and placement cannot be confirmed
- For underground expansions: validated against the relevant underground spatial rules (e.g., Tunnel Area bounds, no overlap). Surface visibility is not relevant
- Standard spatial checks also apply (tiles must be Buildable, no existing structure overlap, faction-specific constraints like GDO Build Area)

### Worker-Built Structures (e.g., Agent building a Tunnel)
- The player queues a build command targeting a map location
- The command is accepted regardless of current visibility — no visibility check at command time
- The worker pathfinds to the location and validates on arrival: tiles must be Buildable, unoccupied, and meet faction-specific constraints
- If validation fails on arrival, the command is cancelled and the worker stops and idles
- No visibility requirement on arrival (the worker is physically present)

## Object Instance
An instance of an Object Type existing on the map.

### Type - ObjectEnum
### Location - Coordinates
### Owner - PlayerNumber | None
### HP - number (Destructible types only)

## Structure Instance
An Object Instance of a Structure Type.

### Rotation - 0 | 90 | 180 | 270
### FlipHorizontal - boolean
### FlipVertical - boolean
### StructureState - struct (to be defined with structure capabilities)

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
### InfoPanel: always displays DeliverySize and DeliveryInterval; when visible to the selecting player, also displays CurrentSupplies
