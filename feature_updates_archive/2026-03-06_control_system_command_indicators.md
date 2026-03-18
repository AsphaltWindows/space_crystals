# Feature Update: Control System — Command Indicators

**Date**: 2026-03-06
**Feature file**: `features/control_system.md`
**Design sources**: `design/control_system.md` (CommandIndicators section)
**Triggered by**: `design_updates/2026-03-06_pending_review_formalization.md`

## Summary

Added CommandIndicators section defining visual markers at command targets for selected units. Two indicator types (Location on ground, Object around perimeter). Color-coded: Green (peaceful movement: Move, Reverse, Enter), Red (hostile: Attack, AttackGround), Orange (aggressive movement: AttackMove, Patrol). Indicators shown only when unit with that command is selected; removed on deselect or completion.
