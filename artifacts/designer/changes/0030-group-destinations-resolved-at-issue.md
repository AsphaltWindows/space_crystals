# 0030 — Group destinations are resolved at issue time and stored per unit

- **Date**: 2026-11-23T19:02:16-05:00
- **Answers**: —
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)` ("Destinations are resolved when the order is given"; queued orders: assumption note)

## What changed and why
A clarification of 0021/0029, worked through with the user. When a group order is given (queued or not), every unit's destination is computed **once, at the click**, for the whole group, and stored as that unit's own resolved destination in its command queue. Units later execute their queued legs individually, at different times, with **no recalculation**. This is what keeps the shape at each destination. For queued orders, the box assumes each unit reaches its planned spot. A unit that falls short still heads to its pre-computed spot on the next leg.

## Implementation impact
- Queue entries for group moves must hold a per-unit resolved location, not the raw click point.
- Must **not** recompute the magic box at dequeue time: each unit would be a group of one and all would converge.

## Verification
- Line of 4 mixed-speed units. Shift-queue far north, then back to the start: units turn around at different times but end in the original line at the start.
- Kill one unit mid-route: the others still end in their spots.
