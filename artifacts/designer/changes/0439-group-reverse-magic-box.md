# 0439 — A group Reverse is placed by the magic box (from the docs)

- **Date**: 2026-11-26T20:20:00-05:00
- **Answers**: Q0172 part 2 (part 1 follows)
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)`

## What changed and why
Answered from the docs (option a): ReversingToLocation is "identical to MovingToLocation except the plan uses Reversing locomotion", so a Reverse to several units takes its destinations from the magic box like a Move. Reverse is now listed with Move, Attack Move and Patrol.

## Implementation impact
None: matches M5-V2.

## Verification
Select three Seekers in a row, Reverse to a point behind them: they back up keeping their row; Reverse to a point inside their spread: they converge.
