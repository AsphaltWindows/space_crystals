# Feature: Simulation Core

## Overview
The fundamental simulation parameters that govern time and space in the game.

## Design Sources
- `design/scale.md`

## Specifications

### SimulationFrame
- The game simulation advances in discrete frames at a fixed rate.
- **FramesPerSecond**: 16

### GridUnit
- Strategic-scale spatial measurement for structure placement, range, and sight.
- Structures snap to a grid measured in grid units.
- Range, SightRange, MinRange, and other strategic distances use grid units.

### SpaceUnit
- Fine-grained spatial measurement for unit silhouettes, movement speeds, acceleration, and physical positioning.
- Movement speeds are measured in space units per frame.
- **SpaceUnitsPerGridUnit**: 64

## Dependencies
None. This is the foundational measurement system.

## Open Questions
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (partially resolved by 64 su/gu ratio)
