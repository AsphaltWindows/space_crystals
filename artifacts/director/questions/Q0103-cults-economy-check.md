# Q0103 — The Cults economy check: geometry, tolerance, targets and build line

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — the oracle is built anyway; only its pass or fail waits.
- **Docs**: `factions/cults/economy.md > RecruitmentHall` (rationale), `> Storage` (rationale); `factions/cults/units/recruit.md > Gathering`; 0259 and 0303 (the Syndicate's precedent)

## Question
Which Cults figures does the economy check hold the build to, measured how, and within what tolerance?

## Context
The figures in the docs:
- "≈ 1.45 SC/s per Recruit" at a 1-tile gap, which by the doc's own formula gives about 0.59 at 2 tiles and 0.37 at 3;
- "each 2-tile face supports roughly 4–5 Recruits at full efficiency";
- "a starting hall (6 Recruits → 20) fills in ... 168 s (2:48)";
- "the earliest second hall is ~0:55–1:15";
- "At 12 s the Cults stay roughly level with them [the GDO and the Colonists] until ~1:30 and pull slightly ahead by 2:48 (best case, all Recruits mining with ideal Storage placement)".

For the Syndicate, the user chose distances edge to edge, ±15%, and "The table is an ideal ... It's better with some variation" (0259, 0303).

## Options
- **Geometry:** (a) the gap from the patch's edge to the Storage's edge, in whole tiles. (b) Centre to centre.
- **Tolerance:** (a) ±15%, as for the Syndicate. (b) Something else.
- **Targets:** (a) all of the figures above. The comparison with the GDO and the Colonists is on an 8-patch field with one Storage per face, measured at 1:30 and 2:48. (b) Only the per-Recruit rates and the Hall's fill time.
- **The opening's build line:** (a) every Recruit mines; the first 300 SC go to a second Hall, built by four Recruits next to the field. (b) A line the user gives.

Proposed: (a) for each.
