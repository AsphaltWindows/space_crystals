# Feature Update: GDO Objects — Deployment Center & Extraction Facility Hotkeys

**Date**: 2026-03-09
**Feature file**: `features/gdo_objects.md`
**Design sources**: `design/gdo_objects.md`, `design/control_system.md`

## Modifications

### Updated: Deployment Center Interface
Expanded the brief interface flow summary into full ObjectInterfaceState[DeploymentCenter] with explicit hotkey assignments:
- BuildMenu (constructing): **X: Cancel Construction** (full refund), **Z**: back to DefaultState
- BuildMenu (ready to place): **X: Cancel Ready Building** (75% refund), **Z**: back to DefaultState
- Aligns with standard Z (back) and X (cancel) slot assignments from control_system.

### Updated: Extraction Facility Interface
Replaced brief "same flow as DC" summary with full ObjectInterfaceState[ExtractionFacility]:
- DefaultState (idle): **Q: Build Extraction Plate** (75 SC)
- DefaultState (constructing): **X: Cancel Construction** (full refund)
- DefaultState (ready): **Q: Place Plate** (enters AwaitingPlacement), **X: Cancel Ready Plate** (75% refund)
- AwaitingPlacement: ghost on valid patches, Escape/right-click returns to DefaultState
- Aligns with standard Q (primary action) and X (cancel) slot assignments.
