# Cults: Economy

## Starting Conditions
Where each starting object is placed is defined by the map's Starting Layout for this faction (core/map_editor.md > Starting Layouts).

The Cults player starts with:
- One pre-built **Recruitment Hall**
- One pre-built **Storage**
- Six **Recruits**, all belonging to the starting Recruitment Hall (they count toward its LocalUnitControlUsage — see RecruitmentHall > Unit Control Tracking)
- **0 Space Crystals**

## Cults Building Mechanics

Cults buildings are constructed on-site by Recruits working from outside the structure. Assigned Recruits are not consumed — they remain on the field near the site for the duration of construction and are vulnerable to attack, same as the structure itself (ConstructionHP Rule). Construction speed scales proportionally with the number of Recruits currently assigned. Recruits can be individually reassigned to other orders at any time; removing a Recruit immediately reduces the assigned count and slows progress. If all Recruits are pulled off a site, progress stalls at its current value (no decay) until Recruits resume work or the building is cancelled. On completion, all Recruits that participated remain on the field as ordinary Recruits, free to be issued new orders.

Space Crystals cost and construction time for each buildable structure are defined in the Recruit's Constructs list (see Recruit > Building), the same way GDO's building costs/times are defined in the DeploymentCenter's Constructs list rather than on the building itself.

### BaseConstructionTime Scaling
Each buildable structure has a BaseConstructionTime: the time to complete construction with exactly 1 Recruit assigned. Effective construction time scales down linearly with the number of Recruits currently assigned — 2 Recruits halve it, 3 Recruits reduce it to a third, and so on (no cap on this scaling other than the practical limit of how many Recruits can be assigned to a site).

Rationale — Recruit-time as the Cults' core cost: every Cults activity (mining, building, staffing, training) reduces to Recruit-time, and walking distance is the variable that makes it expensive. Construction costs a fixed amount of Recruit-time on site (BaseConstructionTime), but each additional builder adds its own travel time, so building faster costs more total labor the farther the site is. Expansion cost is therefore crystals + build labor + travel, with the player choosing the speed/efficiency trade-off.

Rationale — why structures cost both crystals and build time: crystals are the generic, versatile cost component — Recruits can mine them at any time and the player can bank them and spend them later on whatever is needed. Build time is a commitment of Recruits at a specific place and moment. If a structure (e.g. a Recruitment Hall) were free but took much longer to build, building it would be a much bigger commitment that had to be made earlier. The split between crystal cost and build time is the dial that balances the Cults' flexibility: more crystal cost = more flexible (save now, decide later); more build time = more up-front commitment.

### Build Command
- Select one or more Recruits
- Select the Construct option
- Select the desired building
- Left Click on the ground to place it
- All selected Recruits walk to the location and begin construction from outside the site
- **Payment**: the building's Space Crystals cost is deducted as soon as the build order is given (the order requires enough Space Crystals at that moment), not when the Recruits arrive. Players who want to expand before they can afford it simply move Recruits toward the site ahead of time and issue the build order once the crystals are available.
- **Order cancelled before construction starts**: if the build order is cancelled before the building is placed (the Recruits are given another order before arriving, or placement fails validation on arrival), the full Space Crystals cost is refunded. Once construction has started, the order can no longer be cancelled through the Recruits — re-ordering them just removes them from the site (progress stalls, see above), and the building can only be cancelled through its own Cancel Construction command (75% refund, see Cancel Construction below).

### Group construction rules
(user, 2026-11-26; Q0097)
- **Where builders work**: each Recruit walks to the **part of the footprint closest to it**; if that part is taken (blocked), it **wall-hugs** along the footprint until it reaches a part where it can work, and builds from there (within its 8 su reach of the edge, as for mining).
- **Start**: the site starts **as soon as the first Recruit of the order arrives** (placement is checked then).
- **Blocked site**: units on the footprint at that moment, **including Recruits of the same order**, block it, and **the whole order fails** (refunded once in full; Payment and Order cancelled before construction starts above). Designer's note: a group-order special case; nothing retries.
- **Speed counts only Recruits actively building**: a building needs a fixed number of **Recruit-frames** (BaseConstructionTime with 1 Recruit). Every frame, each Recruit that is building during that frame adds 1. Recruits still walking, waiting for a spot or doing anything else add nothing. There is no cap other than room around the site.
- **Under attack**: builders **keep building** (as gatherers keep gathering).
- **Assist** targets only a building **already under construction**, not a ghost whose order hasn't started.

### Group construction refunds
(user, 2026-11-26; Q0098)
- **Partly re-ordered group**: the cost is refunded only once **no Recruit of the group still holds the order** (before construction starts).
- **All builders killed** before the start: **full refund** (as the Agent's order, 0289).
- **Shift chains**: each queued building is refunded in full when its **last holder drops it** before its construction starts.
- **After Cancel Construction**: Recruits with more orders queued (e.g. Shift-queued buildings) **go on to them**; with none, they go idle.

### Assist Construction Command
- Select one or more Recruits
- Select the Assist Construction option
- Left Click on an in-progress building
- Once the selected Recruits reach the building they join construction from outside, increasing build speed

### Cancel Construction
Available while a Cults building is under construction (not yet complete). Cancels construction, refunds 75% of the building's Space Crystals cost (rounded down), and destroys the partially-built structure. A rubble animation plays; this is purely cosmetic and has no gameplay effect. Any Recruits assigned to the site are unaffected by the cancellation — they remain on the field and go idle, available for new orders.

- **X: Cancel Construction**: refunds 75% of the building's Space Crystals cost (rounded down), destroys the partially-built structure. Only available while the building is under construction.

### Evacuation on Destruction
Applies to every Cults building that can hold units inside (Armory, Altar, Temple, Cathedral, Shrine, Gunworks, Machine Shop, Garrison, and any future such building — e.g. the Chop Shop and Airstrip if their production flow holds Recruits). When the building is destroyed:
- **Everyone inside is evacuated alive**: units in the internal pool, units reserved by queued orders, the unit currently being trained or augmented, Recruits staffing research, and Recruits powering the Shrine. This includes non-Recruit infantry held by the Altar (Militants, Gunners, Disciples).
- **Units come out unchanged**: a Recruit mid-training comes out as a Recruit; a unit mid-augmentation does not receive the Augmentation. Existing AugmentationState is kept.
- **Placement**: evacuated units appear around the building's footprint wherever there is open space, unharmed.
- **Orders are cancelled with a full refund**: every in-progress and queued training, augmentation, and research order is cancelled and its full Space Crystals cost refunded (consistent with core/tech_and_research.md > Research shared rule 7).

## RecruitmentHall

The Cults' starting structure. Automatically produces Recruits from the surrounding population of Recruitable terrain. Each Recruitment Hall operates independently with its own local capacity and production rate.

### Faction - TheCults
### Size - 4x4 grid units
### SymmetryType - AAAA
### MaxHP - 1000
### PointArmor - 1
### FullArmor - 16
### SightRange - 6 grid units
### Destructible - true
### Groupable - false
### TechPrerequisite - None

The player starts with one RecruitmentHall, but additional ones are buildable by Recruits like any other Cults structure. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

### RecruitmentArea
A 10x10 grid unit area centered on the 4x4 building. **The whole area must lie on the map** (user, 2026-11-26; Q0099): a Hall whose area would leave the map can't be placed (as Tunnels, 0242).

### TileClaiming
A Recruitment Hall claims Recruitable tiles within its Recruitment Area that are not already claimed by another active Recruitment Hall. Tiles are claimed on a first-built basis — the hall that was constructed first has priority. This priority also applies when tiles are freed by the destruction of a competing hall; if multiple halls contest a newly freed tile, the one that was built first claims it.

**Claiming details** (user, 2026-11-26; Q0099):
- **Priority** goes to the Hall **completed first**. A Hall under construction claims nothing, provides no capacity and produces nothing until it is complete.
- **Ties** (completed in the same frame, e.g. the starting Halls): the contested tiles are split **contiguously** by a simple rule: each contested tile goes to the Hall whose **centre is nearer**; at exactly equal distance, to the Hall with the lower object id (designer's naive rule, as the user asked for "some naive way").
- **Across players**: Halls of **different players compete** for tiles the same way as a player's own Halls.
- **Rounding**: LocalUnitControlCapacity is **rounded down** everywhere (production and HUD): 33 claimed tiles of 100 gives 6.6, so 6.

### Effectiveness
Effectiveness = (number of claimed Recruitable tiles in Recruitment Area) / (total number of tiles in Recruitment Area).

Buildings do not affect recruitability. A Recruitable tile stays Recruitable whether or not a building stands on it — the Recruitment Hall's own 4x4 footprint, any other Cults building, or another faction's building. The only thing that prevents a hall from counting a Recruitable tile is that tile being claimed by another Recruitment Hall (see tile claiming above). A hall on fully Recruitable terrain with no competing halls therefore reaches 100% effectiveness.

**Map design guideline — Recruitable terrain:** balancing starts from the assumption that the whole map is Recruitable (every default tile preset except Water is Recruitable). Map makers then reduce Recruitable areas where needed for balance — e.g. to make some expansions or crystal fields worse for the Cults. Because Recruits lose efficiency at anything far from their hall (walking), the best Cults expansions are halls with local work: a crystal field plus a Storage, or a forward position.

Effectiveness scales both the hall's maximum Unit Control capacity and its Recruit production speed linearly.

### BaseUnitControlCapacity - 20
### BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit

At reduced effectiveness, capacity and production time scale linearly. Example: at 50% effectiveness, capacity = 10, production time = 24 seconds per Recruit.

Rationale: every Cults unit (miners, builders, staff and army) counts against its originating hall's capacity, so a single hall's 20 Unit Control caps the whole faction until more halls are built. At 12 s a starting hall (6 Recruits → 20) fills in 14 × 12 s = 168 s (2:48). 9 s was tried and judged too strong: in a pure-mining comparison on an 8-patch field it put the Cults ~25% ahead of GDO and the Colonists in crystals mined by 2:06. At 12 s the Cults stay roughly level with them until ~1:30 and pull slightly ahead by 2:48 (best case, all Recruits mining with ideal Storage placement). The interval barely affects expansion timing: the earliest second hall is ~0:55–1:15 at either 9 s or 12 s, because the 6 starting Recruits and saving the first 300 SC dominate. The Cults are expected to commit to a second hall early; that hall's Recruits lose efficiency at anything far from it (walking). Playtest tuning dial (range considered: 9–12 s).

### RecruitmentHallInstanceState:
- Effectiveness: number (0.0 to 1.0)
- ClaimedTiles: set of Coordinates
- LocalUnitControlUsage: number (current usage against this hall)
- LocalUnitControlCapacity: number (BaseUnitControlCapacity * Effectiveness)
- CurrentProductionTime: number (BaseRecruitProductionTime / Effectiveness)
- ProductionProgress: number (frames elapsed) | None
- RallyPoint: Coordinates | ObjectInstance | None

### Production Behavior
The Recruitment Hall automatically and continuously produces Recruits at no resource cost until LocalUnitControlUsage reaches LocalUnitControlCapacity. Produced Recruits spawn at the hall and move to its rally point. Production logic is entirely local — global Unit Control state does not affect production decisions.

### Unit Control Tracking
Each Recruit tracks which Recruitment Hall produced it. This lineage persists when a Recruit is trained into another unit. A trained unit's Unit Control cost equals the number of Recruits consumed to produce it, with each point of cost attributed to the originating hall of the respective Recruit. When a unit is destroyed, LocalUnitControlUsage decreases at the originating hall(s).

### Cults Unit Control HUD Display
Displays: (sum of all LocalUnitControlUsage across all active Recruitment Halls and orphaned units) / (sum of all LocalUnitControlCapacity across all active Recruitment Halls). This is a global aggregation for display purposes only.

### On Destruction:
- LocalUnitControlCapacity is removed from the global sum
- Surviving units produced by this hall remain on the field and continue to count toward the global usage sum
- Claimed tiles are freed and may be reclaimed by other active Recruitment Halls based on build-order priority

### ConstructionHP Rule - applies

### ObjectInterfaceState[RecruitmentHall]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **X: Cancel Production**: cancels the current Recruit in production, resetting ProductionProgress. Only available if ProductionProgress is not None.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

## Storage

Drop-off point for Space Crystals collected by Recruits. Multiple Recruits can drop off Space Crystals simultaneously.

Rationale — size: Recruit income falls off steeply with distance (see Recruit > Gathering), so Storage placement is core Cults skill. The best layout leaves a 1-tile gap between patch and Storage; each 2-tile face supports roughly 4–5 Recruits at full efficiency. Moving a Storage one tile closer (2-tile gap → 1-tile gap) gains ~0.86 SC/s per Recruit, ~+4 SC/s per face, paying back the 100 SC in ~25 s plus build time. 2x2 (down from 3x2) was chosen to slow that payback and to make an efficient crystal field take more Storages (3–4 for an 8-patch field), raising the total cost of a well-run field while keeping each Storage at 100 SC / 20 s.

### Faction - TheCults
### Size - 2x2 grid units
### SymmetryType - AAAA
### MaxHP - 250
### PointArmor - 1
### FullArmor - 4 (building armor rule; was 6, a leftover from the old 3x2 size; Q0094)
### SightRange - 3 grid units
### Destructible - true
### Groupable - true
### TechPrerequisite - None

Space Crystals cost and construction time are listed under the Recruit's Constructs list, not here (see Recruit > Building).

### ConstructionHP Rule - applies

### ObjectInterfaceState: None (info display only)
