# 0354 — Sparrow flight: edge overshoot, takeoff, circling spot, strafing, Stop and Hold; 1 GU idle circle

- **Date**: 2026-11-26T23:20:00-05:00
- **Answers**: Q0120 (points 2–7; point 1 in 0328)
- **Docs**: `factions/cults/units/sparrow.md` (Placeholder stats: Idle circle; Flight rules)

## What changed and why
User accepted the defaults:
- **Point 2:** no extra cap on overshooting the map edge.
- **Point 3:** takes off at IdleSpeed and speeds up along the runway.
- **Point 4:** with no rally point, circles 2 GU past the C end.
- **Point 6:** flies on until out of range, then loops back at full speed.
- **Point 7:** Stop circles in place; Hold circles in place and doesn't chase.

**Point 5** (designer placeholder): the idle circle has a fixed **1 GU radius**, instead of the 16 su its full centripetal limit would allow.

## Implementation impact
- At idle, the Sparrow turns with a gentler centripetal acceleration: IdleSpeed² / 64 = 0.25 su/frame².

## Verification
- An idle Sparrow circles a tile-wide loop.
- Ordered to Hold, it circles there and engages only what enters range.
