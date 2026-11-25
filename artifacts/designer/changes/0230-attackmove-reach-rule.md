# 0230 — AttackMove takes only targets it can reach within leash and sight; no dropped-target memory

- **Date**: 2026-11-25T08:10:00-05:00
- **Answers**: Q0058 (and replaces 0204 point 1)
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (steps 2 and 3)

## What changed and why
User: no "left alone for the rest of the leg" memory. It can make units respond differently to the same situation, e.g. when an identical enemy is swapped in unseen. Instead there is a sensible, easily explained default: **an attack-moving unit fights what it can reach without leaving its route or losing sight of it.**
- An enemy within weapon range is always taken (unchanged).
- An enemy out of range is taken only if the path to a firing position, which the unit plans anyway, stays within AttackMoveLeashDistance of PathReference **and** keeps the target within SightRange. Otherwise the unit doesn't take it and walks on.
- A rejected enemy is re-judged only when something relevant changes: the enemy moves a meaningful distance, or the PathKnowledge the path used changes. This applies to the leash case and the fog case alike, so the loops of Q0054 and Q0058 can't start.
- **Performance is an explicit priority.** Expect large armies attack-moving past many unreachable targets. The re-check triggers and caching are the Director's call (goals: performance, predictability).
- Players are expected to correct their units. This is a default, not an optimiser (Core principle, 0223).

## Implementation impact
- **Replaces 0204 point 1** (dropped targets left alone for the leg) and the M3-L2/L3 behaviour for AttackMove (and Patrol, which uses AttackMove's scanning).
- Acquisition must validate the planned path against leash and sight before committing.
- Rejected enemies need a change-driven re-check.

## Verification
- A Guard attack-moves past an unarmed Drone behind a wall. If the only way to the Drone leaves the leash or loses sight of it, the Guard walks on without turning back, and arrives.
- If the Drone then moves into the open within reach, the Guard engages it.
- An enemy that comes within weapon range is always fought.
