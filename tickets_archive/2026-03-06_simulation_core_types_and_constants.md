# Ticket: Implement Simulation Core Types and Constants

## Current State
No simulation foundation exists. There are no types or constants defining the game's time step or spatial measurement systems.

## Desired State
The following types and constants are defined and available for use throughout the codebase:

1. **SimulationFrame**: A fixed-rate simulation tick at 16 frames per second. This should configure Bevy's `FixedTime` (or equivalent fixed timestep) so that game logic runs at exactly 16 FPS, independent of rendering frame rate.

2. **GridUnit**: A type representing strategic-scale spatial measurement. Used for structure placement (grid snapping), range, sight range, min range, and other strategic distances.

3. **SpaceUnit**: A type representing fine-grained spatial measurement. Used for unit silhouettes, movement speeds (space units per frame), acceleration, and physical positioning.

4. **SPACE_UNITS_PER_GRID_UNIT**: A constant with value `64`, defining the conversion ratio between SpaceUnit and GridUnit.

These should be organized in a dedicated simulation core module (e.g., `src/simulation/` or `src/core/`).

## Justification
This is the foundational measurement system for the entire game. All subsequent features (entity system, tile system, unit system, combat, vision) depend on these core definitions. See `features/simulation_core.md`.

## QA Steps
1. Verify that a simulation core module exists in the source tree containing the type and constant definitions.
2. Verify that `SPACE_UNITS_PER_GRID_UNIT` equals `64`.
3. Verify that the fixed timestep is configured to 16 frames per second (i.e., a period of 62.5ms per frame).
4. Run `cargo build` and confirm the project compiles without errors.
5. Run `cargo test` and confirm all tests pass, including any unit tests for the conversion ratio and frame rate constant.

## Expected Experience
- The project compiles cleanly with the new module.
- A test converting 1 GridUnit to SpaceUnits yields 64.
- A test confirming the simulation frame rate constant is 16 FPS passes.
- The new types are importable from other modules in the crate.
