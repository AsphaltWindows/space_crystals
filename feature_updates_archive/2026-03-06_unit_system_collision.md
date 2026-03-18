# Feature Update: Unit System — Unit Collision

**Date**: 2026-03-06
**Feature file**: `features/unit_system.md`
**Design sources**: `design/units.md` (UnitCollision section)
**Triggered by**: `design_updates/2026-03-06_pending_review_formalization.md`

## Summary

Added UnitCollision section defining two collision models:

1. **Ground Collision**: Hard collision using Silhouette rectangle. No overlap, no push. Idle units don't move aside. Moving units pathfind around. Ground units collide with ground units and structures.

2. **Air Collision**: No collision with ground units or structures. Soft separation with other air units via circular SeparationRadius (per unit type, must be larger than Silhouette).

Note: Underground collision (DrillUnit) is unspecified. Added to open questions.
