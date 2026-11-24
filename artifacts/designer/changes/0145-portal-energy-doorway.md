# 0145 — Portal exits: energy doorway at the centre

- **Date**: 2026-11-24T18:20:00-05:00
- **Answers**: Q0041 (points 5 and 6; with 0143, Q0041 done)
- **Docs**: `factions/colonists/structures.md > Portals > Portal Exits (energy doorway)` (new); `> Robo Portal` (exit text), `> Bio Portal > Exit Behavior`; `core/entities.md > Structure Type > Exit Doorway` (portal pointer)

## What changed and why
**Replaces 0144** (closest exit *side*). User's image: an energy doorway at each portal's centre. Units:
1. materialize at the centre and move straight out (through their own portal's footprint) until clear;
2. may leave in any direction not impeded by the structure: through an open (A) side. Robo Portal 360°; Bio Portal (AABB = neighbouring sides, top+right by default — Q0041 pt 5: AABB as written) a half-circle from the top-left corner direction to the bottom-right corner direction;
3. head straight toward the RallyPoint if open; if not open or blocked, go around via the nearest open direction that gets out; no rally → random open unblocked direction (deterministic);
4. if no way out, stay half out at the footprint edge as a normal unit;
5. the next unit's production continues to 100% and then waits fully inside until the previous units have vacated (same as Exit Doorway rule 3; refunded if the portal is destroyed).
Q0041 pt 6 ("materializes"): answered by 1.

## Implementation impact
Portals don't use side-midpoint doorways. Exit is a ray from the centre within the open arc. Heavy/Star Portal: same model, open sides TBD.

## Verification
- Robo Portal rallied northeast: Drones emerge from the centre heading northeast.
- Bio Portal (default orientation) rallied to the southwest (closed side): units exit through the nearest open direction (top-left or bottom-right edge of the arc) and walk around.
- Surround a portal with units: the first unit stops half out; the next reaches 100% and waits; it appears after the first moves away.
