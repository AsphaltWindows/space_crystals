# Feature: Tile System

## Overview
The map terrain system using configurable tile presets with gameplay-relevant properties.

## Design Sources
- `design/entities.md` (Tile, TilePreset, TilePlacement, DefaultTilePresets)

## Specifications

### Tile Properties
- Buildable: boolean (structures can be placed)
- Traversible: boolean (ground units can cross)
- Rugged: boolean (affects infantry movement/defense)
- Drillable: boolean (drill units can traverse underground)
- Recruitable: boolean (Cults can recruit from this tile)

### TilePreset
- Named configuration with a Texture and specific property values.
- Value (TilePresetEnum), Name, Texture (asset)
- Custom presets with arbitrary property combinations can be created via map editor.

### Default TilePresets
| Preset | Buildable | Traversible | Rugged | Drillable | Recruitable |
|--------|-----------|-------------|--------|-----------|-------------|
| Plane | true | true | false | true | true |
| Rugged Terrain | false | true | true | true | true |
| Cliff | false | false | false | true | true |
| Mountain | false | false | false | false | true |
| Water | false | false | false | false | false |

### TilePlacement
- Instance of a TilePreset placed on the map.
- Type (TilePresetEnum), Location (Coordinates), Elevation (integer 0-16)
- Elevation is per-placement, not per-preset.

## Dependencies
- `entity_system` (Tile is a Visible Entity)
