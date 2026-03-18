# Design Update: Syndicate Headquarters Stats & Guard Unit

**Date**: 2026-03-09
**Files modified**: `design/syndicate_objects.md`

## Changes

### Headquarters — Stats Added

The Headquarters tunnel expansion now has full stats defined:

- Size: 2x2
- Cost: 200 Space Crystals
- Build Time: 400 frames (25 seconds)
- HP: 400
- PointArmor: 1
- FullArmor: 4

### Headquarters — Now Produces Guards

The Headquarters produces both Agents and Guards (previously only Agents).

### Guard — New Unit

New Syndicate combat infantry unit added. The Guard is a Heavy Infantry unit produced by the Headquarters.

- UnitBase: HeavyInfantry
- Silhouette: 36x36
- HP: 80, Armor: 1/1
- MaxSpeed: 5 space units/frame
- SightRange: 5
- TunnelSpaceCost: 2
- Groupable: true
- Attack: FullyConnected, Ground, SingleTarget, Damage 6, Range 3 grid units
- Attack timing: Aim 2 → Fire 1 → Cooldown 1 → Reload 4
- Cost: 125 Space Crystals, 120 frames (7.5 seconds)

### Tunnel Expansion Rally Point Behavior — Documented

Clarified how produced units enter the game:

- Rally point set on surface: unit auto-ejects from parent Tunnel (Side A) and moves to rally point.
- No rally point, or rally point set on parent Tunnel: unit stays in Tunnel Network for manual ejection.

### Design Note: MaxSpeed Is Unit-Specific

MaxSpeed is a UnitBaseAttribute in schema (HeavyInfantry defines that it has one), but the value is set per unit type. Agent has MaxSpeed 6, Guard has MaxSpeed 5.
