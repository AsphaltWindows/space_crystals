# 0102 — Defeated players in team games watch with team vision

- **Date**: 2026-11-24T05:15:00-05:00
- **Answers**: — (closes the TBD in 0101)
- **Docs**: `core/game_rules.md > End of Game` (Defeated before the end)

## What changed and why
In team games, a player defeated while their team is still playing keeps watching with **their team's vision only**, meaning what their allies see. There is never a full map reveal, so they can't pass on information their allies couldn't have. They can't command anything.

## Implementation impact
- Spectating mode for defeated players: their fog is the union of their allies' vision. There is no command input.

## Verification
In a 2v2, after one player is defeated, their screen shows exactly what their ally's units and buildings reveal. Enemy units outside the ally's sight stay hidden.
