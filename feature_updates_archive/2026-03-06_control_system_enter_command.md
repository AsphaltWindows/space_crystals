# Feature Update: Control System - Enter Command Right-Click

**Feature file**: `features/control_system.md`
**Design sources**: `design/control_system.md`

## Modifications

### BasicCombatUnitInterfaceState RightClick Updated
- Added Tunnel right-click resolution: right-clicking own Tunnel (Syndicate units only, tier sufficient for unit's base category) now issues Enter command targeting that Tunnel
- Resolution priority: Enemy -> Attack; Ground -> Move; Own Tunnel (Syndicate, tier sufficient) -> Enter; Friendly/Neutral -> Move
