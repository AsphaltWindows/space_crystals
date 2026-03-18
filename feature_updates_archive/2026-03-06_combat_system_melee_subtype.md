# Feature Update: Combat System - FullyConnected Melee Subtype

**Date**: 2026-03-06
**Feature file**: `features/combat_system.md`
**Design sources**: `design/combat.md`
**Triggered by**: `design_updates/2026-03-06_agent_unit_and_melee_subtype.md`

## Modifications

### Added: FullyConnected Subtypes (Ranged / Melee)
- FullyConnected attack type now has two subtypes: **Ranged** and **Melee**
- **Ranged**: Standard behavior with numeric Range. ElevationModifier applies to range.
- **Melee**: Fixed short range (adjacent contact). ElevationModifier does NOT apply to range. All other FullyConnected properties (CanMiss=false, UnitTarget only, unified animation+effect) still apply.

### Added: Open Questions
- Melee range: "adjacent contact" needs concrete implementation rule
- ElevationModifier on attack range: formula not yet specified in combat context
