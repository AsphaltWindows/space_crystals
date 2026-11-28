# 0463 — Null Chamber buff: extra sight is vision only; a finished turn always buffs

- **Date**: 2026-11-27T15:45:00-05:00
- **Answers**: Q0187
- **Docs**: `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff` (Effects) and `> Placeholder stats` (Throughput)

## What changed and why
1. **Option (a), as built.** The +2 GU SightRange widens **vision only**. Target scans keep the unbuffed SightRange: idle auto-targeting, AttackMove and Patrol. The user's reason: a buff shouldn't change unit behaviour in ways that may be undesirable. The user also restated that the buff grants **detection**, which was already listed and isn't designed yet. With the extra sight, the buff is about awareness.
2. **Option (a)**, which changes what was built. A unit whose 2 s turn is done **leaves buffed** even if no Null Chamber is left when it leaves. The turn was completed, so it has the buff.

## Implementation impact
- **Point 1:** none.
- **Point 2:** drop the "a working chamber must exist at leaving" check. A finished turn is kept, and buffs on leaving whatever happened to the chamber.

## Verification
1. A buffed unit on Attack Move reveals 2 GU more of the map but engages at the same distance as an unbuffed one.
2. Let a unit finish its turn, destroy the player's only Null Chamber, then eject the unit. It comes out buffed.
