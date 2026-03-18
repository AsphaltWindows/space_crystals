# Design Update: Command Panel Grid Layout & Standard Slots

**Date**: 2026-03-09
**Files modified**: `design/control_system.md`, `design/gdo_objects.md`, `design/syndicate_objects.md`

## Changes

### CommandPanel Grid Layout (control_system.md)

Documented the 3x3 command grid with default hotkeys:
```
| Q | W | E |
| A | S | D |
| Z | X | C |
```

### Standard Slot Assignments (control_system.md)

Three bottom-row slots are standardized across all object types that use them:

- **Z (bottom-left)**: Back / Cancel menu — returns to previous state in any multi-stage menu. Equivalent to Escape or right-click cancel.
- **X (bottom-center)**: Cancel — cancels production (last queued item), in-progress upgrades, or in-progress construction. Refund amount depends on the building (full or 75%).
- **C (bottom-right)**: Set Rally Point — for unit-producing structures, enters AwaitingTarget[SetRallyPoint].

### Production Building Default Right-Click (control_system.md)

All unit-producing structures now use right-click from DefaultState to set rally point (right-click ground = rally to location, right-click object = rally to object).

### Tunnel — Cancel Upgrade Added (syndicate_objects.md)

New command: **X: Cancel Upgrade** — cancels in-progress Tunnel tier upgrade with full Supplies refund. Only available while upgrading.

### Tunnel Menus — Back Hotkeys Added (syndicate_objects.md)

Z mapped as back/cancel in all Tunnel sub-menus:
- EjectMenu: Z returns to DefaultState
- ExpandMenu: Z returns to DefaultState
- AwaitingPlacement: Z returns to ExpandMenu

### Headquarters — Interface State Added (syndicate_objects.md)

Full ObjectInterfaceState documented for Headquarters:
- Q: Build Agent (100 SC)
- W: Build Guard (125 SC)
- X: Cancel Production
- C: Set Rally Point
- Right-click: Set Rally Point
- BuildQueue max 5, instance state mirrors Barracks pattern

### GDO Barracks — Hotkeys Added (gdo_objects.md)

Existing commands mapped to standard slots:
- Q: Build Peacekeeper
- X: Cancel Production
- C: Set Rally Point (new — was only available via right-click)

### GDO Supply Tower — Hotkeys & Rally Added (gdo_objects.md)

Commands mapped to standard slots:
- Q: Build Supply Chopper
- X: Cancel Production
- C: Set Rally Point (new)
- S: Schedule Deliveries
- Right-click: Set Rally Point (new)

### GDO DeploymentCenter — Hotkeys Added (gdo_objects.md)

Standard slots applied to BuildMenu states:
- X: Cancel Construction (during construction) / Cancel Ready Building (when ready to place)
- Z: Back to DefaultState from BuildMenu

### GDO Extraction Facility — Hotkeys Added (gdo_objects.md)

Standard slots applied:
- Q: Build Extraction Plate / Place Plate
- X: Cancel Construction / Cancel Ready Plate
