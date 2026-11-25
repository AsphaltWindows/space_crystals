# Q0101 — Cults Unit Control: the global rule, and Recruits that no Hall produced

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/overview.md > CultsResources > Unit Control`; `factions/cults/economy.md > Unit Control Tracking`, `> Cults Unit Control HUD Display`, `> Starting Conditions`

## Question
1. **The global rule.** "The player cannot build units if doing so would exceed the total available Unit Control." Halls produce by their local count, and training costs one per Recruit consumed, which already counted. So nothing the design has would ever be stopped by the global total. Is the global figure for display only?
2. **Recruits no Hall produced.** Recruits placed directly by a custom map, or spawned by a debug command, weren't produced by a Hall. Do they count as orphans (global usage only), or against some Hall?

## Context
"Production logic is entirely local — global Unit Control state does not affect production decisions." "A trained unit's Unit Control cost equals the number of Recruits consumed to produce it." The starting Recruits are "all belonging to the starting Recruitment Hall".

## Options
1. (a) Display only, for every Cults unit so far. Vehicles too, since they cost one per Recruit consumed. (b) A real check on something not yet designed. Say what.
2. (a) Orphans. (b) The owner's first Hall in the map's layout. (c) The nearest own Hall.

Proposed: (a) and (a).
