# Q0099 — Recruitment Area: off-map tiles, rounding, priority, Halls under construction

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > RecruitmentHall` (RecruitmentArea, TileClaiming, Effectiveness, RecruitmentHallInstanceState, Cults Unit Control HUD Display)

## Question
1. **Off-map tiles.** Near the map's edge, part of the 10x10 area is off the map. Do those tiles count in "the total number of tiles in Recruitment Area"? If they do, a Hall at the edge is weaker.
2. **Rounding.** At 33 claimed tiles, the capacity is 6.6. Does production stop at 6 or at 7, and what does the HUD show?
3. **"Constructed first."** Does it mean completed first? How are ties broken, for two Halls completed in the same frame, or two starting Halls?
4. **Halls under construction.** Does a Hall under construction claim tiles, provide capacity or produce?
5. **Claims across players.** Do claims compete across players? The letter ("another active Recruitment Hall") says yes.

## Context
"Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment Area)"; "LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)"; production runs "until LocalUnitControlUsage reaches LocalUnitControlCapacity". The Syndicate's areas must lie wholly on the map (0242); nothing says so for Halls.

## Options
1. (a) Off-map tiles count, and can't be claimed: edge Halls are weaker. (b) Only on-map tiles count: a Hall at the edge can reach 100%. (c) Halls must keep their area on the map, as Tunnels must.
2. (a) Production continues while usage is below capacity, so 7 at 6.6, and the HUD shows the capacity rounded down (6). (b) The capacity is rounded down everywhere (6). (c) Rounded to the nearest everywhere (7).
3. (a) Completed first, with ties by the order the Halls were placed (object id). (b) Ordered first.
4. (a) No: it claims and produces from completion, like every other function of a building. (b) It claims from placement, reserving its tiles.
5. (a) Yes, by the letter. (b) Only own Halls compete.

Proposed: (a) for each. 2(a) keeps the exact rule: production runs while 100 × usage < 20 × claimed tiles.
