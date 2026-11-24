# 0045 — Air units ignore terrain; Gliders may briefly leave the map edge

- **Date**: 2026-11-23T19:42:02-05:00
- **Answers**: Q0011 (part 4, air; Q0011 now fully answered by 0038–0045)
- **Docs**: `systems/units.md > UnitCollision > Air Collision`

## What changed and why
Air units fly over every tile regardless of terrain. HoverCraft stay inside the map. **Gliders may unavoidably go a little outside the map edge** while turning or circling, because they can't stop. Orders can only target points inside the map.

## Implementation impact
- Movement bounds: clamp HoverCraft to the map. Gliders may exceed it while maneuvering. Their turning logic should still steer back in.
- TBD: visibility and targetability of a Glider outside the map edge, and the maximum overshoot. Placeholder until decided: it stays visible/targetable under the same rules as at the nearest edge tile.

## Verification
- Order a Dragonfly to a point next to the map edge: it may swing slightly past the edge while turning, then come back.
- A Hornet ordered to the edge stops at the edge.
- Clicking outside the map issues no order beyond the edge.
