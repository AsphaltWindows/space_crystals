# 0035 — Collision response: blocked movement is dropped and velocity resets to 0 (gradual un-sticking); tracked pathfinding note

- **Date**: 2026-11-23T19:18:25-05:00
- **Answers**: —
- **Docs**: `systems/units.md > UnitCollision > Collision Response (Un-sticking)` (new), `> Silhouette Rotation > Turning near obstacles` (now points to it), `systems/units.md > SpeedTurnRadiusMovement` (pathfinding note)

## What changed and why
**Replaces** the rule in 0033 that a unit "can't rotate into an overlapping orientation and must move to free space first". The user pointed out that wheeled vehicles, which can't turn in place, would jam against walls under it. The new per-frame order for every ground unit, which the user describes as "running into a wall every frame":
1. Add acceleration to velocity; compute this frame's turn.
2. Compute displacement from velocity; apply displacement and turn.
3. If the movement hits an obstacle, **drop the blocked portion** (no overlap) and **set velocity to 0**.

A unit against an obstacle therefore restarts from 0 each frame and creeps at **single-frame-acceleration speed** while still turning. It scrapes around the obstacle and then accelerates normally once free. Overlap caused by rotation (corners sweeping into a wall) follows the same principle; the exact geometry is your choice, but the unit never ends up overlapping.

**Tracked units:** they can reach the same spot by different speed/turn trade-offs. The user suggests **pre-computed values** (maneuver tables) to help pathfinding find the optimal path.

## Implementation impact
- Movement integration order and collision response as above, for all ground units.
- Make sure creeping along an obstacle still counts as progress where it actually gets closer, so the stuck/give-up rule (0024) doesn't fire on a unit that is successfully scraping free.
- Tracked pathfinding: precomputed maneuver data is a suggested approach, not a hard requirement.

## Verification
- Drive a Raptor nose-first into a wall, then order it to a point behind it to the right: it creeps, turns while scraping along the wall, then drives off at normal acceleration.
- A long vehicle parked against a wall ordered to turn 90°: it turns gradually without overlapping the wall.
- Tracked vehicles choose sensible routes: slowing for sharp corners, keeping speed on wide arcs.
