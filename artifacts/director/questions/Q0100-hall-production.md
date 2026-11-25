# Q0100 — Hall production: the exit side, X, and counting the Recruit in production

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > RecruitmentHall > Production Behavior`, `> ObjectInterfaceState[RecruitmentHall]`; `core/entities.md > Exit Doorway`; `factions/gdo/overview.md > Unit Control` (0143, "Reserved when production starts")

## Question
1. **The exit side.** "Produced Recruits spawn at the hall and move to its rally point." The Hall is AAAA, so every side is eligible. Which side do Recruits come out of?
2. **What X does.** "X: Cancel Production: cancels the current Recruit in production, resetting ProductionProgress." Production is automatic: does the next Recruit start at once, or does the Hall stop?
3. **Counting.** Does the Recruit in production count toward LocalUnitControlUsage, as other factions' units in production count toward used (0143)?
4. **At capacity.** When the Hall is at capacity and a unit dies, does production start from zero?

## Context
The exit doorway rule: "Buildings with **several exit-eligible sides** choose the side by their own rules; a side whose doorway is occupied counts as blocked". The Colonists' portals use an energy doorway at the centre instead.

## Options
1. (a) The side facing the rally point (the bottom when there is none), then the next free side clockwise. (b) Always the bottom side, so a blocked doorway pauses the Hall. (c) The centre, like a portal's energy doorway.
2. (a) Production restarts at once: X only delays it. (b) The Hall stops until some later event: none is designed, so it needs a resume command.
3. (a) It counts from when it appears. The HUD shows 19/20 while the 20th is in production. (b) It counts from the start of its production, as in 0143.
4. (a) Yes: nothing is in production while the Hall is at capacity. (b) The progress is kept, paused.

Proposed: (a) for each.
