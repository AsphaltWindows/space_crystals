# Chop Shop

Tier 2 major building of the Tech/Weaponry branch. A scavenger's garage where Recruits are turned into vehicles.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Armory

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Production
Produces Cults vehicles: the **Jackal** (light vehicle, factions/cults/units/jackal.md) and the **Hand of Heaven** (missile platform; 3 Recruits; factions/cults/units/hand_of_heaven.md). Each vehicle consumes multiple Recruits (number set per vehicle type), and its Unit Control cost equals the number of Recruits consumed. **Production works like the Armory** (user, 2026-11-26; factions/cults/structures/armory.md > Training): Recruits enter and join an internal pool; orders are queueable and serial; queuing an order deducts its Space Crystals cost and **reserves the vehicle's number of Recruits** from the pool (an order can only be queued if enough unreserved Recruits are inside); when it finishes, they are consumed and the vehicle exits to the RallyPoint. Eject All and Cancel work as at the Armory (cancel returns the reserved Recruits to the pool).Capacity, queue size and sides: see Placeholder stats.

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 3x3; SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory); MaxHP 450; PointArmor 1, FullArmor 9; SightRange 4 grid units; Groupable **false** (user, 2026-11-26; Q0105: Cults production buildings aren't groupable).
- Cost and time: Recruit > Constructs (200 SC, 640 frames).
- InternalRecruitCapacity 10, counting reserved Recruits; so at most 5 Jackals queued (bounded by capacity: reserved units count toward it; Q0104).
- **Jackal**: 2 Recruits, 75 Space Crystals, 320 frames (20 seconds).
- Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Build Jackal**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production** (refunds the crystals and returns the reserved Recruits).

**Full HP** (user, 2026-11-26; Q0121): a vehicle or aircraft comes out at **full HP**, whatever damage its Recruits had.
