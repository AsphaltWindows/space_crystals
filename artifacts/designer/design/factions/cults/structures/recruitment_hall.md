# RecruitmentHall

(Moved from factions/cults/economy.md, 2026-11-27; its heading had been lost in an earlier edit.)

The Cults' starting structure. Automatically produces Recruits from the surrounding population of Recruitable terrain. Each Recruitment Hall operates independently with its own local capacity and production rate.

## Faction - TheCults
## Size - 4x4 grid units
## SymmetryType - AAAB (user, 2026-11-26; Q0100: **one exit side, B**; side placeholder; Recruits come out of it one at a time through the Exit Doorway, core/entities.md)
## MaxHP - 1000
## PointArmor - 1
## FullArmor - 16
## SightRange - 6 grid units
## Destructible - true
## Groupable - true (user, 2026-11-27; was false: selected Halls form one group, and a rally point set with several selected applies to all of them)
## TechPrerequisite - None

The player starts with one RecruitmentHall, but additional ones are buildable by Recruits like any other Cults structure. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## RecruitmentArea
A 10x10 grid unit area centered on the 4x4 building. **The area may extend past the map edge** (user, 2026-11-27; replaces Q0099's "the whole area must lie on the map"): a Hall can be placed anywhere its 4x4 footprint fits on the map, including right at the edge. The part of its area that lies **off the map yields no Recruitable tiles**, so a Hall near the edge is **suboptimal but possible**. Effectiveness still divides by the full 100 tiles (Effectiveness). (Tunnel Areas keep their own on-map rule, 0242.)

## Recruitment Area display
(user, 2026-11-26; Q0107, option b; as the Tunnel Area display, factions/syndicate/tunnel_area_and_space.md > Tunnel Area display)
- **Placing a Hall** shows its 10x10 area: the tiles it would claim, the tiles already claimed by your own Halls, the non-Recruitable tiles, and its Effectiveness in %.
- **Your own Halls under construction and placed ghosts** (not yet started) also show the tiles they **would** claim (user, 2026-11-26), marked as taken by them, even though a Hall claims nothing until it is complete (Claiming details).
- **Enemy Halls you can see, or remember** (seen before and now in fog, shown in their last-known state; core/vision.md > Explored) count in the preview: the tiles they claim are shown as unclaimable (user, 2026-11-26). A remembered Hall is assumed to still be there, so the estimate can be wrong (it may be gone). Enemy Halls you have never seen are not counted (no information leak), so the preview may then be optimistic.
- **A selected own Hall** shows its claimed tiles: its **true** claims, matching its InfoPanel's Effectiveness and what it produces (user, 2026-11-27; Q0133). Unlike the placement preview, it may reveal where an unseen enemy Hall took tiles: the player's Recruits live on that land, so that is fair.
- Enemy Halls' areas are never drawn as areas of their own.

## TileClaiming
A Recruitment Hall claims Recruitable tiles within its Recruitment Area that are not already claimed by another active Recruitment Hall. Tiles are claimed on a first-built basis — the hall that was constructed first has priority. This priority also applies when tiles are freed by the destruction of a competing hall; if multiple halls contest a newly freed tile, the one that was built first claims it.

**Claiming details** (user, 2026-11-26; Q0099):
- **Priority** goes to the Hall **completed first**. A Hall under construction claims nothing, provides no capacity and produces nothing until it is complete.
- **Ties** (completed in the same frame, e.g. the starting Halls): the contested tiles are split **contiguously** by a simple rule: each contested tile goes to the Hall whose **centre is nearer**; at exactly equal distance, to the Hall with the lower object id (designer's naive rule, as the user asked for "some naive way").
- **Across players**: Halls of **different players compete** for tiles the same way as a player's own Halls.
- **Rounding**: LocalUnitControlCapacity is **rounded down** everywhere (production and HUD): 33 claimed tiles of 100 gives 6.6, so 6.

## Effectiveness
Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment Area). The total is always the full 10x10 = 100, **off-map tiles included** (they count as non-Recruitable; user, 2026-11-27).

Buildings do not affect recruitability. A Recruitable tile stays Recruitable whether or not a building stands on it — the Recruitment Hall's own 4x4 footprint, any other Cults building, or another faction's building. The only thing that prevents a hall from counting a Recruitable tile is that tile being claimed by another Recruitment Hall (see tile claiming above). A hall on fully Recruitable terrain with no competing halls therefore reaches 100% effectiveness.

**Map design guideline — Recruitable terrain:** balancing starts from the assumption that the whole map is Recruitable (every default tile preset except Water is Recruitable). Map makers then reduce Recruitable areas where needed for balance — e.g. to make some expansions or crystal fields worse for the Cults. Because Recruits lose efficiency at anything far from their hall (walking), the best Cults expansions are halls with local work: a crystal field plus a Storage, or a forward position.

Effectiveness scales both the hall's maximum Unit Control capacity and its Recruit production speed linearly.

## BaseUnitControlCapacity - 20
## BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit

At reduced effectiveness, capacity and production time scale linearly. Example: at 50% effectiveness, capacity = 10, production time = 24 seconds per Recruit.

Rationale: every Cults unit (miners, builders, staff and army) counts against its originating hall's capacity, so a single hall's 20 Unit Control caps the whole faction until more halls are built. At 12 s a starting hall (6 Recruits → 20) fills in 14 × 12 s = 168 s (2:48). 9 s was tried and judged too strong: in a pure-mining comparison on an 8-patch field it put the Cults ~25% ahead of GDO and the Colonists in crystals mined by 2:06. At 12 s the Cults stay roughly level with them until ~1:30 and pull slightly ahead by 2:48 (best case, all Recruits mining with ideal Storage placement). The interval barely affects expansion timing: the earliest second hall is ~0:55–1:15 at either 9 s or 12 s, because the 6 starting Recruits and saving the first 300 SC dominate. The Cults are expected to commit to a second hall early; that hall's Recruits lose efficiency at anything far from it (walking). Playtest tuning dial (range considered: 9–12 s).

## RecruitmentHallInstanceState:
- Effectiveness: number (0.0 to 1.0)
- ClaimedTiles: set of Coordinates
- LocalUnitControlUsage: number (current usage against this hall)
- LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)
- CurrentProductionTime: number (BaseRecruitProductionTime / Effectiveness)
- ProductionProgress: number (frames elapsed) | None
- RallyPoint: Coordinates | ObjectInstance | None

## Production Behavior
The Recruitment Hall automatically and continuously produces Recruits at no resource cost until LocalUnitControlUsage reaches LocalUnitControlCapacity. Produced Recruits come out of the **exit side (B)** and move to its rally point. **Production can't be cancelled** (user, 2026-11-26; Q0100). **Unit Control**: the Recruit in production **counts toward LocalUnitControlUsage from the start of its production** (as other factions, 0143). When the Hall is at capacity nothing is in production; when capacity frees up (a unit dies), the next Recruit **starts from 0**. Production logic is entirely local — global Unit Control state does not affect production decisions.

## Unit Control Tracking
Each Recruit tracks which Recruitment Hall produced it. **Recruits no Hall produced** (placed by a custom map or spawned by a debug command; user, 2026-11-26; Q0101) are **orphans** from the start: they count in the global display only, against no Hall. This lineage persists when a Recruit is trained into another unit. A trained unit's Unit Control cost equals the number of Recruits consumed to produce it, with each point of cost attributed to the originating hall of the respective Recruit. When a unit is destroyed, LocalUnitControlUsage decreases at the originating hall(s).

## Cults Unit Control HUD Display
Displays: (sum of all LocalUnitControlUsage across all active Recruitment Halls and orphaned units) / (sum of all LocalUnitControlCapacity across all active Recruitment Halls). This is a global aggregation for display purposes only.

## On Destruction:
- LocalUnitControlCapacity is removed from the global sum
- Surviving units produced by this hall remain on the field and continue to count toward the global usage sum
- Claimed tiles are freed and may be reclaimed by other active Recruitment Halls based on build-order priority

## ConstructionHP Rule - applies

## ObjectInterfaceState[RecruitmentHall]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

(No cancel: Recruit production can't be cancelled; user, 2026-11-26; Q0100.)

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).
