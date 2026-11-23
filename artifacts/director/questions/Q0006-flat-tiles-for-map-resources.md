# Q0006 — Do map resources (patches, Supply Delivery Stations) need flat tiles?

- **Date**: 2026-11-23T13:31:00-05:00
- **Blocking**: no. Proceeding on the assumption that they don't: the editor lets map makers put crystal patches and Supply Delivery Stations on sloped tiles.
- **Docs**: `core/map.md > Elevation > Structures only on flat terrain`, `core/map.md > SpaceCrystalsPatch`, `core/map.md > SupplyDeliveryStation`, `core/map_editor.md`

## Question
Must a crystal patch or a Supply Delivery Station placed in the map editor sit on flat tiles, as structures must?

## Context
Change 0004 limits **structures** to flat tiles and lists the player placement methods it applies to (Direct Placement, Worker-Built, Warp In). Patches and stations are map objects placed by the map maker, not structures, so the rule doesn't obviously cover them. A station has a 2×2 footprint and will be a 3D-rendered building-like sprite, so on a slope it may look wrong. I'm reworking the map model and the editor for corner heights now, so the answer decides whether the editor's placement ghost goes red for them on a slope.

## Options
- **A (current assumption)**: no flatness rule for map resources. Both can go anywhere their footprint fits.
- **B**: stations need flat tiles, patches don't. The editor refuses a station on a slope and map validation flags it.
- **C**: both need flat tiles.
