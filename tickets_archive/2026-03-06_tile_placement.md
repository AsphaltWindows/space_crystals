# Ticket: Implement TilePlacement

## Current State
Tile presets and properties are defined (or being defined), but there is no way to represent a specific tile placed on the map at a location with an elevation.

## Desired State
A **TilePlacement** struct is defined with the following fields:

1. `Type` (TilePresetEnum): which preset this placed tile uses.
2. `Location` (Coordinates): the map position of the tile. Uses the coordinate system from the simulation core / entity system.
3. `Elevation` (integer, range 0 to 16 inclusive): the height of this specific tile placement. Elevation is per-placement, not per-preset — two tiles of the same preset can have different elevations.

TilePlacement represents a concrete instance of a TilePreset on the game map. It should be placed in the same tile system module as TilePreset.

## Justification
TilePlacement is how the map is composed from tile presets. Elevation affects vision and combat range via the ElevationModifier system. Per-placement elevation is a core map design tool. See `features/tile_system.md`.

## QA Steps
1. Verify that TilePlacement exists with fields for Type (TilePresetEnum), Location (Coordinates), and Elevation (integer).
2. Verify that Elevation is constrained to the range 0-16 (either at the type level or via validation).
3. Verify that a TilePlacement can be instantiated with any valid TilePresetEnum variant, a coordinate, and an elevation value.
4. Run `cargo build` and confirm the project compiles without errors.
5. Run `cargo test` and confirm all tests pass, including a test that creates a TilePlacement with elevation 0, one with elevation 16, and verifies both are valid.

## Expected Experience
- The project compiles cleanly.
- A test creates a Plane tile at coordinates (0,0) with elevation 0 and a Mountain tile at (5,3) with elevation 16 — both succeed.
- A test attempting to create a tile with elevation 17 either fails to compile (type-level constraint) or returns an error/panic (runtime validation).
- TilePlacement is importable from the tile system module.
