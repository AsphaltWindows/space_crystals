# Airstrip

Tier 3 major building of the Tech/Weaponry branch. A makeshift runway where Recruits are turned into aircraft.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Chop Shop

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Production
Produces Cults aircraft: the **Sparrow** (small glider; factions/cults/units/sparrow.md), made from Recruits. Advanced aircraft — a large paratrooper plane and an airship/zeppelin — are also built here but require the player to own a Scrap Hangar. **Production works like the Armory** (user, 2026-11-26; factions/cults/structures/armory.md > Training): Recruits enter an internal pool, orders are serial and queueable and reserve the aircraft's number of Recruits, which are consumed on completion. A finished aircraft **takes off from the Airstrip** to its RallyPoint; Cults aircraft (so far) never land back. Capacity, queue size and sides: see Placeholder stats.

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 2x4 (a runway); SymmetryType ABAC with the short sides B and C; EntranceSide B (Recruits walk in); a finished aircraft is placed **directly on the runway** and takes off along it toward C (the exception to exit-opposite-entrance, Q0104); MaxHP 400; PointArmor 1, FullArmor 8; SightRange 4 grid units; Groupable true.
- Cost and time: Recruit > Constructs (250 SC, 640 frames).
- InternalRecruitCapacity 10, counting reserved Recruits; so at most 10 orders queued (bounded by capacity: reserved units count toward it; Q0104).
- **Sparrow**: 1 Recruit, 75 Space Crystals, 320 frames (20 seconds).
- Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build Sparrow**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**.

**Full HP** (user, 2026-11-26; Q0121): a vehicle or aircraft comes out at **full HP**, whatever damage its Recruits had.
