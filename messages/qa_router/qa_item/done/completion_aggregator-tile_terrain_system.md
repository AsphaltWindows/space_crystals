# tile_terrain_system

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# tile-terrain-system

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the tile and terrain system as defined in `artifacts/designer/design/entities.md` under Tile, TilePreset, DefaultTilePresets, and TilePlacement.

**Tile:**
A non-selectable visible entity that makes up the game's map. Tile properties are defined by TilePresets. Properties:
- **Buildable** (boolean): whether structures can be placed on this tile
- **Traversible** (boolean): whether ground units can walk on this tile
- **Rugged** (boolean): whether this is rugged terrain (infantry-only traversal with defense bonus for LightInfantry)
- **Drillable** (boolean): whether underground units (DrillUnit) can travel through this tile
- **Recruitable** (boolean): whether Cults faction can recruit from this tile

**TilePreset:**
A named tile configuration with a texture and specific property values. Map designers can define custom presets via the map editor.

**DefaultTilePresets:**
| Preset | Buildable | Traversible | Rugged | Drillable | Recruitable |
|--------|-----------|-------------|--------|-----------|-------------|
| Plane | true | true | false | true | true |
| Rugged Terrain | false | true | true | true | true |
| Cliff | false | false | false | true | true |
| Mountain | false | false | false | false | true |
| Water | false | false | false | false | false |

**TilePlacement:**
An instance of a TilePreset placed on the map at a specific location and elevation.
- Type: TilePresetEnum
- Location: Coordinates
- Elevation: integer (0 to 16)

## QA Instructions

1. Create a map with all 5 default tile presets (Plane, Rugged Terrain, Cliff, Mountain, Water).
2. Verify each tile type has a distinct visual texture.
3. Attempt to place a structure on each tile type — verify it only succeeds on Plane (Buildable=true).
4. Order a ground unit to walk across each tile — verify it can traverse Plane and Rugged Terrain but not Cliff, Mountain, or Water.
5. Verify tiles can have elevation values from 0 to 16.
6. Place tiles at different elevations and verify they render at appropriate heights.
7. Verify that Rugged Terrain tiles are marked as Rugged (relevant for infantry defense bonuses and traversal rules).
