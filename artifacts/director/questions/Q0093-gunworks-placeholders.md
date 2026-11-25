# Q0093 — Placeholder stats for the Gunworks

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: the stats in option (a), with no research and no staffing.
- **Docs**: `factions/cults/structures/gunworks.md`; `factions/cults/units/recruit.md > Constructs`; `core/entities.md > Building armor rule`

## Question
If the Gunworks is in the playtest (Q0092), what are its placeholder stats? And does it hold Recruits in this build, which has no research?

## Context
"Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit staffing capacity, research queue size, and ObjectInterfaceState: TBD." Its cost and time are TBD in the Recruit's Constructs list. Its research ("Weapon upgrades (list, costs, BaseResearchTimes TBD)") isn't designed, so staffing would do nothing. The nearest designed buildings are the Armory (2x3, HP 300, 1/6, sight 4, 200 SC, 480 frames) and the Storage (2x2, HP 250, 100 SC, 320 frames).

## Options
- **(a)** 2x2, AAAA, HP 300, armour 1/4 (the building rule with n = 1), sight 4, Groupable; 150 SC and 480 frames with one Recruit; no panel commands; no staffing until research is designed. A right-click on it with Recruits moves them there.
- **(b)** The Syndicate's Optics Lab's numbers (2x2, HP 400, 150 SC, 400 frames), with a surface armour of 1/4.
- **(c)** Option (a) with staffing: Recruits can enter, as the Recruit's right-click list says ("Own Gunworks / Machine Shop → Enter (staff research)"), but nothing happens inside. That is confusing without research.

Proposed: (a).
