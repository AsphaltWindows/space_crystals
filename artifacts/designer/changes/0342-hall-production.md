# 0342 — Recruitment Hall: one exit side, no cancel, the Recruit in production counts, restart from 0

- **Date**: 2026-11-26T21:45:00-05:00
- **Answers**: Q0100
- **Docs**: `factions/cults/economy.md > RecruitmentHall` (SymmetryType; Production Behavior; ObjectInterfaceState)

## What changed and why
User:
1. **The Hall has one exit side.** It becomes AAAB with the exit on B (the side is a placeholder). Recruits come out of it one at a time through the Exit Doorway.
2. **Recruits can't be cancelled.** The X button is removed.
3. **The Recruit in production counts** toward LocalUnitControlUsage from the start of its production.
4. When capacity frees up, **production starts from 0**.

## Implementation impact
- Hall sides and placement rotation.
- Remove X.
- Unit Control counts the Recruit in production.

## Verification
- At 19/20 with a Recruit in production, the HUD shows 20/20.
- Recruits always appear on side B.
