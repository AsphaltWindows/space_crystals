# src/

Root source directory for Space Crystals RTS.

## Structure

- **main.rs** — Application binary entry point: camera setup, lighting, GamePlugin
- **lib.rs** — Library crate root: re-exports shared, game, simulation, ui modules for use by integration tests and the binary
- **shared/** — Crate-wide types (Owner, GridPosition, Unit, FactionEnum, etc.) and shared utilities
- **simulation/** — Simulation core: fixed timestep plugin, spatial types (GridUnit, SpaceUnit), constants (FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT)
- **game/** — Core game logic (combat, units, world/map)
- **ui/** — User interface systems (HUD, minimap, command panel)
