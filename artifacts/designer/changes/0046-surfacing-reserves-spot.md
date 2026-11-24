# 0046 — Surfacing reserves the spot

- **Date**: 2026-11-23T19:43:01-05:00
- **Answers**: —
- **Docs**: `systems/units.md > UnitCollision > Underground Collision` (Reserved while surfacing)

## What changed and why
User decision: once an underground unit has **started** surfacing, its surface footprint is reserved. Ground units treat it as an obstacle and can't move onto it. Building placement there is also refused (my extension, the same reasoning). This lasts until surfacing completes. It prevents a unit stepping onto the spot mid-transition.

## Implementation impact
Later domain (not M1). The reservation acts as a temporary obstacle in collision and placement checks.

## Verification
(Later.) Start surfacing a drill unit, then order a Peacekeeper onto the spot: it paths around or stops short.
