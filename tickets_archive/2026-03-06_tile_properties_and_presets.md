# Ticket: Implement Tile Properties and TilePreset System

## Current State
No tile or terrain system exists. There are no types defining tile gameplay properties or terrain presets.

## Desired State
The following types and data are defined and available for use throughout the codebase:

1. **Tile Properties**: Five boolean properties that define gameplay behavior for a tile:
   - `Buildable`: whether structures can be placed on the tile
   - `Traversible`: whether ground units can cross the tile
   - `Rugged`: whether the tile affects infantry movement/defense
   - `Drillable`: whether drill units can traverse underground through the tile
   - `Recruitable`: whether Cults can recruit from the tile

2. **TilePresetEnum**: An enum identifying each tile preset by variant.

3. **TilePreset**: A named configuration struct containing:
   - `Value` (TilePresetEnum)
   - `Name` (string)
   - `Texture` (asset reference/placeholder)
   - All five boolean tile properties

4. **Default TilePresets**: Five built-in presets with the following property values:

   | Preset         | Buildable | Traversible | Rugged | Drillable | Recruitable |
   |----------------|-----------|-------------|--------|-----------|-------------|
   | Plane          | true      | true        | false  | true      | true        |
   | Rugged Terrain | false     | true        | true   | true      | true        |
   | Cliff          | false     | false       | false  | true      | true        |
   | Mountain       | false     | false       | false  | false     | true        |
   | Water          | false     | false       | false  | false     | false       |

5. The Tile itself is a **Visible Entity** that is **not selectable** (Selectable = false). It should integrate with the entity hierarchy from the entity system. Any combination of properties is valid for custom presets; the defaults above represent common terrain types.

These should be organized in a tile system module (e.g., `src/tile/` or similar).

## Justification
The tile system is the foundation for the game map. Vision, movement, building placement, and combat all reference tile properties. See `features/tile_system.md` and `design/entities.md`.

## QA Steps
1. Verify that a tile system module exists containing TilePresetEnum, TilePreset, and tile property definitions.
2. Verify that TilePresetEnum has exactly 5 variants: Plane, RuggedTerrain, Cliff, Mountain, Water.
3. Verify that TilePreset contains fields for Value, Name, Texture (or placeholder), Buildable, Traversible, Rugged, Drillable, and Recruitable.
4. Verify that each default preset's property values match the table above (e.g., Plane is Buildable=true, Traversible=true, Rugged=false, Drillable=true, Recruitable=true).
5. Verify that Tile integrates with the entity hierarchy as a Visible, non-Selectable entity.
6. Run `cargo build` and confirm the project compiles without errors.
7. Run `cargo test` and confirm all tests pass, including unit tests for default preset property values.

## Expected Experience
- The project compiles cleanly with the new tile module.
- A test instantiating each default preset and checking its boolean properties passes.
- TilePresetEnum variants are importable from other modules.
- The Tile type correctly marks itself as Visible and not Selectable within the entity hierarchy.
