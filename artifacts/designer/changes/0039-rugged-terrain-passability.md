# 0039 — Rugged terrain: impassable for vehicles, except hover vehicles (double drag)

- **Date**: 2026-11-23T19:26:15-05:00
- **Answers**: Q0011 (part 2; unseen-structure pathing and later domains still open)
- **Docs**: `systems/units.md > UnitBase > RuggedTerrain` (meaning defined), `systems/units.md > HoverVehicle > RuggedTerrain` (now true, with double drag)

## What changed and why
- `RuggedTerrain = false` means **Rugged tiles are impassable** for that base: its Silhouette may never overlap one. This applies to Wheeled, Tracked and Drill vehicles (option A).
- **Exception:** **HoverVehicles can cross Rugged terrain**, but their **DragRatio is doubled** while over it, which halves effective MaxSpeed and makes them decelerate faster. They get no defense bonus.
- Light/Heavy Infantry and Mechs cross Rugged as before; Light Infantry keeps its defense bonus.

## Implementation impact
- Pathing: Rugged = blocked for Wheeled/Tracked/Drill; passable for infantry, Mechs and HoverVehicles.
- **Colonists Drones are HoverVehicles**, so they can reach crystal fields ringed by Rugged terrain, slowly.
- Which tile counts for the drag doubling: use the same rule as the Rugged defense bonus (the tile under the unit's centre, pending Q0014.3).

## Verification
- A Raptor or Artillery Rig paths around a Rugged patch; a Peacekeeper walks through.
- A Drone crosses a Rugged patch at roughly half speed and speeds back up on leaving it.
