# 0165 — On defeat, unit modes stay as they were

- **Date**: 2026-11-24T20:40:00-05:00
- **Answers**: Q0036 (point 3; Q0036 done)
- **Docs**: `core/game_rules.md > On Defeat`

## What changed and why
Option A (current build). Becoming neutral clears orders but not modes: a raised Riot Shield stays raised; an attack already in Firing/Cooldown completes.

## Implementation impact
None.

## Verification
Defeat a player whose Peacekeepers are shielded: the neutral Peacekeepers keep shields up (half damage taken).
