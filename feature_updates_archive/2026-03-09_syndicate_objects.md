# Feature Update: Syndicate Objects

**Date**: 2026-03-09
**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`, `design/control_system.md`

## Modifications

### Added: Tunnel Cancel Upgrade command
Added **X: Cancel Upgrade** to Tunnel DefaultState commands. Cancels in-progress tier upgrade with full Supplies refund. Only available while upgrading. Aligns with standard X slot assignment.

### Added: Headquarters full ObjectInterfaceState
Added HeadquartersInstanceState (RallyPoint, BuildQueue max 5, CurrentBuild, CurrentBuildProgress) and ObjectInterfaceState[Headquarters]:
- Q: Build Agent (100 SC)
- W: Build Guard (125 SC)
- X: Cancel Production
- C: Set Rally Point
- Right-click Ground/Object: SetRallyPoint
- Follows Barracks pattern for production building interface.
