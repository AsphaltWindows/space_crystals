# Cults: Economy

## Starting Conditions
Where each starting object is placed is defined by the map's Starting Layout for this faction (core/map_editor.md > Starting Layouts).

The Cults player starts with:
- One pre-built **Recruitment Hall**
- One pre-built **Storage**
- Six **Recruits**, all belonging to the starting Recruitment Hall (they count toward its LocalUnitControlUsage — see factions/cults/structures/recruitment_hall.md > Unit Control Tracking)
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
- **Blocked site**: other units on the footprint at that moment block it, and **the whole order fails** (refunded once in full; Payment and Order cancelled before construction starts above). Designer's note: a group-order special case; nothing retries.
- **The order's own Recruits** (user, 2026-11-27; Q0130; replaces "including Recruits of the same order"): the site **waits** for Recruits of the same order who stand on the footprint to step off, then starts. Only other units fail it. **A Recruit holding this Build queued** behind another order (a Shift chain) counts as one of the order's own Recruits (user, 2026-11-27; Q0131): the site waits until it steps off, e.g. once it has finished the earlier building and walks over. (If (d) proves hard to build, the fallback is (a): the ghost is red while a selected Recruit stands on the footprint.)
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
- **Placement** (user, 2026-11-26; Q0106): the building is simply gone (its destruction is an animation; no debris affects play), and evacuated units **all appear at the same time inside its former footprint**, unharmed, spread over it. Units that overlap ignore each other until separated, and idle ones push apart (systems/units.md > Ground Collision).
- **A unit that had finished training** and was waiting for the doorway comes out **as the trained unit**, with no refund (its training was complete).
- **Afterwards** evacuated units **idle**.
- **Orders are cancelled with a full refund**: every in-progress and queued training, augmentation, and research order is cancelled and its full Space Crystals cost refunded (consistent with core/tech_and_research.md > Research shared rule 7).

## Economy check
(user, 2026-11-26; Q0103) The figures the build is measured against (the Director's economy oracle):
- **Distance** is measured **edge to edge**, patch to Storage, in whole tiles (as for Agents).
- **Tolerance ±15%**: a measured figure passes if it is within 15% of its target either way (e.g. 1.45 SC/s per Recruit passes anywhere from about 1.23 to 1.67).
- **Targets**: ≈ 1.45 SC/s per Recruit at a 1-tile gap (≈ 0.59 at 2, ≈ 0.37 at 3); full efficiency at about **2 Recruits per tile of face length** (designer's estimate for the 3x2 Storage, 2026-11-27: ~6 on a 3-tile face, ~4 on a 2-tile face; was "4–5 per 2-tile face" for the 2x2 Storage; re-measure); a starting Hall fills from 6 to 20 in 168 s (2:48); the earliest second Hall at ~0:55–1:15; roughly level with the GDO and the Colonists until ~1:30 and slightly ahead by 2:48.
- **Two scenarios, on an 8-patch field:**
  1. **Ideal mining**: one Storage per face, all Recruits mining (the "level until 1:30, ahead by 2:48" comparison was made this way: pure mining, best case).
  2. **Realistic opening** (user's build line): start with the one starting Storage; **save for a second Recruitment Hall without building any extra Storages**, spreading new Recruits to patches farther from the starting Storage; **build the second Hall with all the non-optimal Recruits** (those on the far patches); then **build Storages around the patches**, one per face. Measured against the GDO and the Colonists at 1:30 and 2:48 and reported.
     - **Target** (user, 2026-11-27; Q0135, option b): expanding early is an **investment**: it may fall behind early, but it should **overtake ideal mining (scenario 1) by about 4:00**.
     - **A longer timeline** (user, 2026-11-27): measure both Cults scenarios, the GDO and the Colonists out to **6:00**, at **1:30, 2:48, 4:00, 5:00 and 6:00**, and report how the economies compare. Further targets may follow from that.
     - **What to compare at each checkpoint** (user, 2026-11-27), for all four lines:
       1. **Income per second** at that point (measuring window: the 10 s, 160 frames, before the checkpoint; designer's choice);
       2. **Total mined**: Space Crystals delivered since 0:00 (the starting bank not included);
       3. **Bank**: unspent Space Crystals at the checkpoint;
       4. **Total spent on economy**: Space Crystals paid so far for economy items, refunds subtracted.
     - **Everything built in these runs is economy** (user, 2026-11-27): the scenarios build nothing but economy, so "spent on economy" is simply **everything spent**, Generators included (e.g. Cults Halls and Storages; GDO Extraction Facilities, Plates and Generators; Colonists Drones and Egress Nodes). If a scenario ever needs a non-economy building, report it separately.
     - First measurement (Q0135, 2x2 Storage): 0.51 of the GDO by 1:30 and 0.62 by 2:48. The 3x2 Storage (0380) is expected to help.

## Buildings
The Recruitment Hall and the Storage have their own files (moved from here, 2026-11-27): `factions/cults/structures/recruitment_hall.md`, `factions/cults/structures/storage.md`.
