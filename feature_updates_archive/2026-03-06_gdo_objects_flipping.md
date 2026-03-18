# Feature Update: GDO Objects - Placement Flipping

**Date**: 2026-03-06
**Feature file**: `features/gdo_objects.md`
**Design sources**: `design/gdo_objects.md`
**Design update**: `design_updates/2026-03-06_tunnel_stats_and_mechanics.md`

## Modifications

### Deployment Center AwaitingPlacement
- Added: F flips ghost horizontally, Shift+F flips vertically (complements existing R/Shift+R rotation).
- Added: Side labels (A/B/C/D per SymmetryType) displayed on ghost preview, updating with rotation and flipping.

Note: This change also applies to Extraction Facility's AwaitingPlacement (which shares the same construct-then-place flow), though all current GDO structures with placement are symmetric (AAAA), making flipping a no-op for them. The feature becomes relevant when asymmetric structures are introduced.
