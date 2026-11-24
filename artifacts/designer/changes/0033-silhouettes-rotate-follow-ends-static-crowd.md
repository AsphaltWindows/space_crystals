# 0033 — Silhouettes rotate with facing; 2-GU blocked rule for static targets; follow-end confirmed

- **Date**: 2026-11-23T19:11:01-05:00
- **Answers**: Q0009 (point 2; Q0009 now fully answered by 0031–0033)
- **Docs**: `systems/units.md > UnitCollision > Silhouette Rotation` (new), `systems/controls/behaviors.md > MovingToObject > Completion` ("Blocked near the target")

## What changed and why
- **Silhouettes rotate with the unit's facing** (user chose B over the Director's default A). Long vehicles fit through gaps only when pointed through them. Collision, crushing, range (0031) and proximity (0032) use the rotated shape.
- **Turning near obstacles:** hard collision means a unit can't rotate into an orientation that would overlap a structure, impassable terrain or another ground unit. It has to move to free space first, and pathfinding must account for room to turn. This is my resolution of the consequence, flagged to the user.
- **2-GU blocked rule also covers static targets** (a building or crystal patch surrounded by a crowd): stuck within 2 GU behind non-moving units = reached, so the order completes. Confirmed by the user.
- **The follow ends when the target dies or leaves the map** (0032): confirmed by the user.

## Implementation impact
- **Replaces** default A (axis-aligned). Collision shapes are oriented rectangles (OBB). Square silhouettes behave nearly the same, but their corners now sweep when turning.
- Rotation blocked by overlap needs handling in movement and pathfinding (turning-room checks).
- Related open TBD (systems/units.md > Crushing): whether a crusher turning in place crushes adjacent crushable enemies.

## Verification
- A long vehicle approaching a gap narrower than its length but wider than its width passes only when facing along the gap.
- A long vehicle parked lengthwise against a wall can't spin in place into the wall; it pulls away first.
- Right-click a crystal patch surrounded by idle enemy units: the unit stops within 2 tiles and its order completes immediately.
