# Design Update: Agent Unit & Melee Subtype

**Date**: 2026-03-06
**Files modified**: `design/syndicate_objects.md`, `design/combat.md`

## Changes

### FullyConnected Melee Subtype (combat.md)

- FullyConnected attack type now has two subtypes: **Ranged** and **Melee**
- Ranged: standard behavior, benefits from ElevationModifier on range
- Melee: fixed short range (adjacent contact), ElevationModifier does NOT apply to range
- Peacekeeper's existing attack is FullyConnected Ranged; Agent's attack is FullyConnected Melee

### Agent Unit (syndicate_objects.md)

New unit: the Syndicate's cyborg worker.

- **UnitBase**: Heavy Infantry, 36x36 silhouette
- **Stats**: 75 HP, 1/1 armor, sight 5, speed 6 su/frame, infinite accel/decel, 180 deg/frame turn
- **Tunnel Space cost**: 2
- **Production**: Headquarters, 100 SC, 160 frames (10 seconds)
- **Attack**: FullyConnected Melee, Ground, SingleTarget, 6 damage, Aim 2 / Firing 4 / Cooldown 1 / Reload 9
- **Crystal gathering**: 50 SC per load, 48f mine, 48f drop-off at Tunnel Side B
- **Supply gathering**: 1 Supply per trip, 48f pick up, 48f drop-off at Tunnel Side C
- **Building**: Constructs Tunnels and defensive structures on surface, must be present for duration

### Headquarters Update (syndicate_objects.md)

- Removed Guard from Headquarters production — HQ now only produces Agents
- Guard will be produced by a separate T1 underground expansion (TBD)
