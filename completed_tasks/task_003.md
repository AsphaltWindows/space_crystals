# Task 003: Implement Tile System Matching Design Specification

## Status
**Completed** - 2026-02-22

Renamed TileType -> TilePresetEnum, TileProperties -> TilePreset (added value and name fields). Added TilePlacement component with tile_type, location, and elevation. Updated all 10 referencing files. Preset property values match design exactly.

## Description
Refactor the tile system to match the formal design specification. Replace the current `TileType` enum and `TileProperties` component with a TilePreset-based system. Implement the 5 default presets with their exact property values and add the Elevation field to tile placements.

## Why Needed
The design formalizes tiles as preset-based configurations with 5 boolean properties (Buildable, Traversible, Rugged, Drillable, Recruitable) and separates the preset definition from the placement (which adds Location and Elevation). The current code has a similar but not identical system — it needs to match exactly.

## Acceptance Criteria
- `TilePresetEnum` enum with 5 default variants: `Plane`, `RuggedTerrain`, `Cliff`, `Mountain`, `Water`
- `TilePreset` struct/data with: `Value` (TilePresetEnum), `Name` (String), `Buildable`, `Traversible`, `Rugged`, `Drillable`, `Recruitable` (all bool)
- Default preset values match design exactly:
  - Plane: Buildable=true, Traversible=true, Rugged=false, Drillable=true, Recruitable=true
  - RuggedTerrain: Buildable=false, Traversible=true, Rugged=true, Drillable=true, Recruitable=true
  - Cliff: Buildable=false, Traversible=false, Rugged=false, Drillable=true, Recruitable=true
  - Mountain: Buildable=false, Traversible=false, Rugged=false, Drillable=false, Recruitable=true
  - Water: Buildable=false, Traversible=false, Rugged=false, Drillable=false, Recruitable=false
- `TilePlacement` component with: `tile_type` (TilePresetEnum), `location` (grid coordinates), `elevation` (u8, 0-16)
- A function or method to look up `TilePreset` properties from a `TilePresetEnum` value (e.g., `TilePresetEnum::properties() -> TilePreset`)
- Existing tile spawning system updated to use new types
- Existing pathfinding references to tile properties updated
- Project compiles and runs, map renders correctly

## Relevant Files/Components
- Current map module — has `TileType` enum, `TileProperties`, `GridPosition`, `Tile`, `GridMap`
- Current pathfinding module — references tile traversibility

## Technical Considerations
- The current `TileType` enum and `TileProperties` component can be replaced. `TileType` becomes `TilePresetEnum`, `TileProperties` fields now include `Recruitable` (new field).
- The current `determine_tile_type()` function generates a test pattern — keep a similar test pattern but use TilePresetEnum values.
- `GridPosition` can remain as-is — it maps to the design's `Location` in `TilePlacement`.
- Elevation defaults to 0 for existing tiles. The elevation system is defined but doesn't need active gameplay behavior yet — just store the value.
- Consider implementing `TilePresetEnum::properties()` as a match expression returning the preset data, since the 5 default presets have fixed values.

## Prerequisites
- [ ] `task_001.md` — Directory structure must be in place
- [ ] `task_002.md` — Core enums (TilePresetEnum could live in core types or map types)

## Complexity
Simple
