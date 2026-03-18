# Feature Update: Control System — Box Selection Priority

**Date**: 2026-03-06
**Feature file**: `features/control_system.md`
**Design sources**: `design/control_system.md` (BoxSelection section)
**Triggered by**: `design_updates/2026-03-06_pending_review_formalization.md`

## Summary

Added BoxSelection section defining 5-tier priority system for drag-box selection:

1. Own units (multi-select all in box)
2. Own buildings (single-select closest to center)
3. Enemy units (single-select closest to center)
4. Enemy buildings (single-select closest to center)
5. Neutral objects (single-select closest to center)

Only own units produce multi-selection. Highest-priority tier with any objects wins; lower tiers excluded.
