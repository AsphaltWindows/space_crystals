# Feature Update: Control System

**Date**: 2026-03-09
**Feature file**: `features/control_system.md`
**Design sources**: `design/control_system.md`

## Modifications

### Added: CommandPanel Grid Layout
Documented the 3x3 command grid with default hotkeys (Q/W/E, A/S/D, Z/X/C).

### Added: Standard Slot Assignments
Three bottom-row slots standardized across all object types that use them:
- **Z**: Back / Cancel menu (returns to previous state in multi-stage menus)
- **X**: Cancel Production / Cancel Upgrade (refunds cost)
- **C**: Set Rally Point (for unit-producing structures, enters AwaitingTarget[SetRallyPoint])

### Added: Production Building Default Right-Click
All unit-producing structures now use right-click from DefaultState to set rally point (ground -> location, object -> object).
