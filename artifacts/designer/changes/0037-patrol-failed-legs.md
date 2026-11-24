# 0037 — Patrol: a failed leg counts as arrived; two in a row end the Patrol

- **Date**: 2026-11-23T19:20:44-05:00
- **Answers**: Q0010 (part 3; Q0010 now fully answered by 0034, 0036, 0037)
- **Docs**: `systems/controls/behaviors.md > Patrolling > Failed legs` (new; ConsecutiveFailedLegs state)

## What changed and why
Option A. A leg that fails (give-up rule, 0024) counts as arrived and the unit turns around. Two failed legs in a row end the Patrol, and the unit goes idle. Unreachable or crowded endpoints (0025, 0026) and fighting aren't failures.

## Implementation impact
Matches the Director's default.

## Verification
- Block one end of a patrol with Hold Position units: the unit gives up that leg, turns around, and keeps patrolling the reachable part.
- Box the unit in on both sides: after two failed legs it stops patrolling and is idle.
