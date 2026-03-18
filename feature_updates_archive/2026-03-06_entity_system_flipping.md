# Feature Update: Entity System - Structure Flipping

**Date**: 2026-03-06
**Feature file**: `features/entity_system.md`
**Design sources**: `design/entities.md`
**Design update**: `design_updates/2026-03-06_tunnel_stats_and_mechanics.md`

## Modifications

### Structure Type
- Added: placement now supports rotation AND flipping (horizontal/vertical axis).
- Up to 8 possible orientations for fully asymmetric buildings (ABCD), fewer for more symmetric types.

### Structure Instance
- Added: `FlipHorizontal` (boolean) and `FlipVertical` (boolean) properties.
- These complement the existing `Rotation` property to fully describe building orientation.
