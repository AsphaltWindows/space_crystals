# 0057 — GDO sees a remembered Supply station's real delivery timer

- **Date**: 2026-11-23T20:23:15-05:00
- **Answers**: Q0019
- **Docs**: `core/map.md > SupplyDeliveryStation > InfoPanel` (Explored, GDO: wording clarified), `core/vision.md > Explored Selectability` (unchanged, the rationale)

## What changed and why
The docs already answer this. In Explored state, a GDO player's InfoPanel displays NextDeliveryTime, and the rationale is that GDO "requires planning around delivery timing even without active vision": supplies are meant for the GDO. So the panel shows the **live, true** NextDeliveryTime. It isn't frozen (B) or extrapolated from the last sighting (A).

Your option A doesn't work in all cases anyway: the countdown only runs while the station is **empty** (`SupplyDeliveryStation`). If it held supplies when last seen, nobody can extrapolate when it was emptied. CurrentSupplies stays hidden.

## Implementation impact
- **Replaces** assumption A. Read the station's real NextDeliveryTime for GDO players regardless of visibility.
- Not empty → no countdown → show "no delivery scheduled" (or equivalent).
- Side effect (intended per the GDO rationale, being confirmed with the user): a live timer that starts tells the GDO the station was just emptied.

## Verification
- As GDO, see a station, leave. While it's in fog, an enemy empties it: the GDO InfoPanel switches from "no delivery scheduled" to a running countdown, and the countdown matches the actual delivery.
- Non-GDO players can't select it in fog.
