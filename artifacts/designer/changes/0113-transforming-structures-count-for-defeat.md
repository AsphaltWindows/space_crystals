# 0113 — Defeat: structures under transformation count

- **Date**: 2026-11-24T13:10:00-05:00
- **Answers**: Q0039 (point 5)
- **Docs**: `core/game_rules.md > Defeat Condition`

## What changed and why
Option A (follows from 0109). A Beacon under deployment and a building under Warp In are structures and keep their owner undefeated. Cancelling a deployment that was the player's only building defeats them in the same tick; the cancel is never refused.

## Implementation impact
Defeat check counts transforming structures; it runs after a cancel in the same tick.

## Verification
- Colonists player loses every building while one Drone is mid-deploy: not defeated.
- Cancel that deploy: defeated immediately (units become neutral per On Defeat).
