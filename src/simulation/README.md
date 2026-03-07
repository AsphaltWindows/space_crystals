# simulation/

Simulation core module for Space Crystals RTS.

## Contents

- **mod.rs** — `SimulationCorePlugin`, simulation constants (`FRAMES_PER_SECOND`, `SPACE_UNITS_PER_GRID_UNIT`)
- **types.rs** — Spatial measurement newtypes: `GridUnit`, `SpaceUnit`
- **utils.rs** — Shared simulation helper functions
- **diagnostics/** — Per-system performance diagnostics plugin (behind `diagnostics` feature flag)
