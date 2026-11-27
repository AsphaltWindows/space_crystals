# 0442 — Supply Chopper unit transport: lands to load and unload, instantly

- **Date**: 2026-11-26T20:50:00-05:00
- **Answers**: — (closes the last GDO design gap, the Chopper's transport TBD)
- **Docs**: `factions/gdo/units/supply_chopper.md > Cargo`, `> Unit Transport` (new), `> ObjectInterfaceState[SupplyChopper]`

## What changed and why
User: the Chopper "lands to load and unload and loads and unloads instantly like the Subterranean Transport". Confirmed proposal:
- **Capacity**: 4 infantry (one place each, Light or Heavy) or 1 vehicle of any kind, or Supplies; one kind at a time.
- **Load**: infantry/vehicle right-click the Chopper (Enter): it lands where it is, units walk in and board instantly. Or Chopper right-clicks an own unit (Load): flies over, lands beside it, the unit walks in. Units that don't fit stay out.
- **Unload**: **F: Unload All**, targeted (Shift-queues): fly to the spot, land, everyone out at once around it.
- **Landing**: on infantry-walkable ground free of buildings and units (else the nearest free spot); 16 frames to land and to take off, air target during both, ground target while landed.
- **Takes off by itself** after loading/unloading (user).
- Passengers can't fire out; **all die if it's destroyed** (user), landed or airborne.

## Implementation impact
New: Load/Enter, Unload All, the landed state outside Supply stations and Towers. The Chopper's existing supply commands still refuse while carrying units (and loading is refused while carrying Supplies).

## Verification
- Select 4 Peacekeepers, right-click a hovering Chopper: it lands, they walk in and vanish one by one, it takes off.
- A fifth Peacekeeper stays outside.
- F, click a spot across the map: it flies there, lands, all four appear around it, it takes off.
- Shoot it down loaded: the passengers die.
- An Air-only weapon can hit it while it's landing but not once it's down.
