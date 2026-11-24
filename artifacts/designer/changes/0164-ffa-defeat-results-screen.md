# 0164 — Defeated with no allies left: results screen

- **Date**: 2026-11-24T20:35:00-05:00
- **Answers**: Q0036 (point 2)
- **Docs**: `core/game_rules.md > End of Game` (Defeated before the end (no team left))

## What changed and why
Option B. A player defeated mid-game with no undefeated allies (e.g. free-for-all) leaves to a results screen and stops watching; the game continues for the others. Own-memory spectating shows nothing live; full vision could leak information.

## Implementation impact
Replaces the "Defeated: spectating" own-memory view in that case. Team games unchanged (team vision, 0102). Results screen contents TBD.

## Verification
3-player FFA: the first defeated player is moved to a results screen; the other two keep playing.
