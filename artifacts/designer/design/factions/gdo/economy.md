# GDO: Economy

## Starting Conditions
Where each starting object is placed is defined by the map's Starting Layout for this faction (core/map_editor.md > Starting Layouts).

The GDO player starts with:
- One pre-built **Deployment Center**
- One pre-built **Extraction Facility**
- Two **Extraction Plates** already placed on Space Crystals patches near the starting base (which patches: set in the map's GDO Starting Layout)

Starting resources: **100 Space Crystals**. Starting Supplies: **0** (placeholder; revisit when GDO Supply costs are designed, user 2026-11-24). Starting power: +30 (Deployment Center) − 15 (Extraction Facility) − 6 (two plates) = **+9**. Exact placement of the starting structures and plates relative to the player's start location is defined by the map's start position.

### Opening Economy Baseline
GDO's early game is bounded by two build queues that should not sit idle: the Deployment Center (buildings) and the Extraction Facility (Extraction Plates, i.e. economy growth). Design goals for the opening:

- **No instant Barracks.** GDO starts with income (two plates) rather than a crystal bank, so a Barracks cannot be put down before the player has had a chance to scout. (An earlier draft with 1 plate and 450 Space Crystals allowed a Barracks at frame 0 while still building plates — rejected for that reason.)
- **Even start against the Colonists.** Two plates (≈6.7 Space Crystals per second) roughly match the Colonists' four starting Drones (≈6 per second at typical distances). Three starting plates (10 per second) were rejected because they put GDO ahead from the first frame — see Economy Targets (GDO vs Colonists) below.
- **The Generator is the cheapest building per second of Deployment Center time** (100 Space Crystals / 15 seconds ≈ 6.7 per second; compare Extraction Facility 10, Barracks 20). It is the "inert" filler for the Deployment Center.
- **Anything else forces a trade-off** between idling the Deployment Center (save up; the building comes later) and cutting an Extraction Plate (the building comes sooner, at the cost of that plate's income until the base is fully plated — the *acceleration* option). With 25-second plates this choice matters mainly in the first ~30–40 seconds; after that the Deployment Center queue is the main constraint.
- **Slow plates (25 seconds).** A plate takes 25 seconds to build, so a GDO main base fills up gradually instead of reaching its one-base ceiling within the first minute. GDO is the hardest faction to expand, so fast one-base scaling on top of one-base strength was rejected. See Economy Targets (GDO vs Colonists) below.
- **Extraction Plate payback** (75 Space Crystals at 10 per 48 frames ≈ 22.5 seconds) deliberately sits between the Agent's and the Drone's payback at typical mining distances.

**How the opening plays out** (reference base, simulated):
- **The tight phase is the first ~30–40 seconds.** The starting 100 Space Crystals afford either the first plate (75) or the first Generator (100) at frame 0, not both. Plates first is best: a plate at frame 0, the first Generator once 100 Space Crystals are banked. Opening with a Generator delays the first plate by ~11 seconds and costs ≈ 8% of income by 3:00.
- **Power is not a constraint in the opening.** Starting power +9 covers three more plates, and with 25-second plates the first Generator always finishes before power runs out.
- **After the opening, the Deployment Center is the bottleneck.** The Extraction Facility only spends 3 Space Crystals per second of its time, so from about 1:00 a Generators-only Deployment Center floats Space Crystals (~300 banked by 1:30, ~600 by 2:00). The surplus is meant to go into Barracks and tech buildings — the decision becomes *which building next*. (Tech building costs, still TBD, should be set with this spend rate in mind.)
- **Power gates the Barracks.** A Barracks draws −30, so building one without going negative requires two Generators first. Fastest clean Barracks (Plate, Generator, Generator, Barracks) starts at ~46 seconds and finishes at ~56 seconds. An all-in Barracks (halting everything else) can start at ~15 seconds, but leaves power at −21, so plates and Peacekeeper production run at ~59% speed until Generators catch up.
- **Fully plated at ~2:30** on the reference base (6 plates to build at 25 seconds each). Plating an 8-patch expansion with one Extraction Facility takes ~3:20.

**Reference starting base: 8 Space Crystals patches.** These numbers are tuned against a starting base with 8 patches in GDO build range (so the Extraction Facility has 6 plates to build). **The patches must also be within the starting Deployment Center's sight** (6 grid units), since plates can only be placed on Visible tiles (core/entities.md > Placement Validation > Direct Placement; user, 2026-11-24: 6 is enough to see a long row of patches if the starting DC is placed favourably). This is an assumption of the reference base, not an editor rule: map makers can place layouts however they like (no editor warning; user, 2026-11-24). Patch count is a map-maker dial: fewer patches make an early plate cut cheap (the Extraction Facility catches up sooner), more patches make it costly and extend the Extraction Facility's busy period.

**Second Extraction Facility.** Every Extraction Facility can place plates anywhere in the current build area, so a second one on the same base doubles plating speed. A second Extraction Facility costs ~275 Space Crystals' worth (200 plus about three-quarters of a Generator for its −15 power) and 20 seconds of Deployment Center time. Built around 1:00, it loses value on the 8-patch reference base, is marginal at ~9 patches, pays off from ~10 patches, and is close to standard on mains with 12 or more. This is intended: small mains reward one Extraction Facility, big mains reward two.

### Economy Targets (GDO vs Colonists)
**Closed** (user, 2026-11-27; Q0146): automated economy testing is wrapped up ("so much of it has to do with build orders and optimizations and that would be more effectively tested by me"). These targets stay as **reference for playtests**; the Director measures nothing more against them.

The Colonists are the apt one-base comparison for GDO (Cults must balance Recruits' other jobs; the Syndicate should have the lowest one-base saturation income and expand fast, gated by Supplies). Comparing one Extraction Facility against one Robo Portal on the same patches, the intended shape is:

1. **Early on, the Colonists lead**: from the start to about 1:40 (restated 2026-11-25, Q0055/Q0072: the Colonists start slightly higher, ≈6.9 vs 6.7, so "grows faster up to 2 Drones per patch" no longer measures cleanly).
2. **After that, the Colonists grow more slowly** (each extra Drone adds less, and every 6th Drone becomes a Beacon).
3. **GDO's full saturation yields more Space Crystals per second, and costs less, than the Colonists at 2 Drones per patch.**
4. **The Colonists' full saturation yields more than GDO's full saturation.**

Reference check (8 adjacent patches in a row, Egress Node 2 grid units from the row, Drone acceleration included): Colonists at 2 Drones per patch ≈ 23 Space Crystals per second for 18 Drones (900 Space Crystals, including 2 Beacons); GDO fully plated = 26.7 per second for 8 plates (600); Colonists fully saturated = 32 per second (24 working Drones). **The table below is the simulated measurement** (user, 2026-11-25, Q0055/Q0072: the measured economy is the design; Drone acceleration per 0205; one Robo Portal producing Drones nonstop, a Beacon whenever capacity runs short), not an estimate:

**Reference geometry** (user, 2026-11-24): the Egress Node sits on the row's **long side, centred along it**, with a **2-grid-unit gap of open ground between the node's edge and the edge of the row's middle patches** (edge to edge, not centre to centre).

**A benchmark, not a rule** (user, 2026-11-24): this layout is only the balancing test setup. Maps place patches however they like and players build their Egress Node wherever they want; incomes on other layouts differ, as intended.

**Matching the table in a simulated check** (user, 2026-11-24): on the reference layout, targets 1–4 must hold and each income figure must be within **±15%** of the table. A larger gap is brought back to the designer with the measured table; game numbers are not adjusted to force a match.

| Time | Colonists income | GDO income |
|---|---|---|
| 0:00 | 6.9 | 6.7 |
| 0:40 | 11.4 | 10.0 |
| 1:00 | 13.3 | 12.9 |
| 1:20 | 16.0 | 15.2 |
| 1:40 | 18.7 | 17.6 |
| 2:20 | 22.9 (2 Drones per patch) | 23.3 |
| 2:40 | 25.1 | 26.2 (fully plated) |
| 3:00 | 26.3 | 26.7 |
| 4:00 | 27.8 | 26.7 |

The Colonists lead from the start to about 1:40, GDO is ahead from about 2:20 to the mid 3:00s as its plates land, and the Colonists pass GDO's ceiling between 3:40 and 4:00, reaching 32 per second at full saturation. (The earlier estimated table, which ignored each new Drone's first trip and the longer trips to the row's end patches, is superseded.)

## Generator
GDO power generation structure. Provides power to the GDO global power grid.

### Faction - GlobalDefenseOrganization
### Size - 2x2 grid units
### SymmetryType - AAAA
### MaxHP - 350
### PointArmor - 1
### FullArmor - 4
### SightRange - 3 grid units
### Destructible - true
### Groupable - true
### BuildRadiusExtension - 1 grid unit
### Power - +20

### ObjectInterfaceState: None (info display only)

## ExtractionFacility
GDO resource extraction structure. Constructs Extraction Plates onto Space Crystal Patches within the GDO player's build area. Can construct one plate at a time.

### Faction - GlobalDefenseOrganization
### Size - 3x3 grid units
### SymmetryType - AAAA
### MaxHP - 500
### PointArmor - 1
### FullArmor - 9
### SightRange - 3 grid units
### Destructible - true
### Groupable - false
### BuildRadiusExtension - 2 grid units
### Power - -15

### ExtractionFacilityInstanceState:
- CurrentConstruction: ExtractionPlate | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ExtractionPlate | None

### Constructs:
- Extraction Plate: 75 Space Crystals, 400 frames (25 seconds)

### Cancellation:
- Cancel during construction: full refund of Space Crystals cost
- Cancel when ready to place: 75% refund of Space Crystals cost (rounded down)

### ObjectInterfaceState[ExtractionFacility]:

DefaultState (idle — no CurrentConstruction, no ReadyToPlace):
- **Q: Build Extraction Plate**: deducts 75 Space Crystals, starts construction (CommandIssuingTransition). Only available if player has sufficient Space Crystals.

DefaultState (constructing — CurrentConstruction is set):
- **Q: Place Plate**, greyed out, with a **running progress bar** on the button showing the plate's construction (user, 2026-11-27); it lights up when the plate is ready.
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition)

DefaultState (ready to place — ReadyToPlace is set):
- **Q: Place Plate**: enters AwaitingPlacement (StateOnlyTransition)
- **X: Cancel Ready Plate**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition)

AwaitingPlacement:
- A ghost preview of the Extraction Plate follows the cursor, snapped to the grid. The ghost is tinted green when over a valid Space Crystal Patch (within build area, no existing plate) and red otherwise.
- The player's current GDO build area is highlighted as a semi-transparent overlay on the ground (the build area only; per-tile validity shows on the ghost; user, 2026-11-24).
- Left-click Space Crystal Patch within GDO build area that does not already have a plate: places Extraction Plate, clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to DefaultState (StateOnlyTransition), plate remains ready to place

## ExtractionPlate
GDO resource harvesting structure. Placed onto a Space Crystal Patch by an Extraction Facility. Mines Space Crystals from the patch at a fixed rate. When the patch is depleted, continues mining at a residual rate indefinitely.

### Faction - GlobalDefenseOrganization
### Size - 1x1 grid unit
### SymmetryType - AAAA
### MaxHP - 85
### PointArmor - 2
### FullArmor - 2
### SightRange - 0
### Destructible - true
### Groupable - true
### BuildRadiusExtension - 0
### Power - -3

### MiningRate - 10 Space Crystals per 48 frames (3 seconds)
### ResidualMiningRate - 1 Space Crystal per 48 frames (3 seconds)

### PowerPenalty - When total GDO power is negative, MiningRate and ResidualMiningRate are reduced proportionally (same ratio as the global power penalty for buildings).

**Paid in lumps** (user, 2026-11-24): a plate delivers its whole amount at once at the end of each 48-frame cycle (+10, or +1 at the residual rate), not as a trickle. A new plate's first payment comes 48 frames after it is placed. Under the power penalty the cycle progresses more slowly (at the power ratio), so the lumps come less often. (The opening baseline's timings were computed as a trickle; with lumps they shift by about 2 seconds, e.g. the fastest clean Barracks starts at ≈48 s instead of 45.8 s.)

**Last partial load** (user, 2026-11-24): when the patch holds less than a full load (e.g. 5 left of a 1505 patch), the next payment takes only what is left; after that the plate mines at ResidualMiningRate.

**Depleted patch under a plate**: the patch stays under its plate with 0 crystals left, and the plate keeps mining at ResidualMiningRate indefinitely; the patch is removed only when the plate is destroyed (see On destruction). This is GDO's exception to the general rule that a depleted patch disappears (core/map.md > SpaceCrystalsPatch).

### InfoPanel: displays remaining Space Crystals in the underlying patch
**Only the plate's owner** sees the remaining crystals (user, 2026-11-24). Every other player, allies included, gets the reduced panel for other players' structures (name, owner, HP; core/entities.md > Default InfoPanel for structures).

**Selecting a covered patch** (user, 2026-11-24): a click on a plate's tile selects the **plate**; the covered patch can't be selected on its own while the plate is there.

### On destruction:
- If the underlying Space Crystal Patch still has resources: the patch becomes uncovered and accessible
- If the underlying Space Crystal Patch is depleted: the patch is removed from the map

### ObjectInterfaceState: None (info display only)

## SupplyTower
GDO supply collection structure. Serves as the drop-off point for supplies gathered by Supply Choppers. Produces Supply Choppers and comes with one free Supply Chopper when placed. A Supply Tower can have one Supply Chopper attached to it, enabling automated scheduled deliveries from a Supply Delivery Station.

### Faction - GlobalDefenseOrganization
### Size - 3x3 grid units
### SymmetryType - AAAA
### MaxHP - 400
### PointArmor - 1
### FullArmor - 9
### SightRange - 4 grid units
### Destructible - true
### Groupable - true
### BuildRadiusExtension - 1 grid unit
### Power - -15
### TechPrerequisite - player owns at least one Radar Station

### SupplyTowerInstanceState:
- AttachedChopper: ObjectInstance | None (the Supply Chopper currently attached to this tower)
- LandedChopper: ObjectInstance | None (a Supply Chopper currently landed on this tower, may differ from AttachedChopper)
- ScheduledSDS: ObjectInstance | None (the Supply Delivery Station this tower's attached chopper is scheduled to collect from)
- BuildQueue: array of ObjectEnum (max 20 entries, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### Produces:
- Supply Chopper: 100 Space Crystals, 160 frames (10 seconds)

### On Placement:
- One Supply Chopper is spawned on the tower at no additional cost (included in tower's construction cost). This chopper is automatically attached to the tower.

### Landing and Attachment:
- A Supply Chopper that lands on its attached Supply Tower drops off all carried supplies and is gradually repaired at no cost, at the **repairers' rate** (0 to full in half its build time: 80 frames, about 30 HP a second; designer placeholder, 2026-11-26; Q0169).
- When a Supply Chopper attaches to a Supply Tower, it immediately evicts any existing landed Supply Chopper that is not the attached chopper.
- When a Supply Tower has an attached Supply Chopper, no other Supply Chopper may land on it.
- Any command given to an attached Supply Chopper breaks its attachment to the tower.
- A Supply Chopper must not be carrying units to attach to a Supply Tower.

### Scheduled Deliveries:
- A Supply Tower with an attached Supply Chopper may schedule deliveries from a specific Supply Delivery Station.
- The attached chopper automatically departs at a time calculated so it arrives at the SDS simultaneously with the next expected supply delivery.
- After picking up supplies, the chopper returns to the tower, drops off supplies, and waits for the next departure.
- If the travel distance is too long or deliveries at the SDS are too frequent, the chopper departs immediately after dropping off supplies.
- If multiple Supply Towers have attached choppers with scheduled deliveries from the same SDS, only one chopper flies to that SDS at a time. Priority: closest tower with a chopper ready to depart goes first.

### ObjectInterfaceState[SupplyTower]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Supply Chopper**: deducts 100 Space Crystals from player, adds SupplyChopper to BuildQueue. Only available if BuildQueue has fewer than 20 entries and player has sufficient Space Crystals.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).
- **S: Schedule Deliveries**: enters AwaitingTarget[ScheduleDeliveries]. Only available if tower has an attached chopper.

### AwaitingTarget[ScheduleDeliveries] resolution:
- Left-click SupplyDeliveryStation: sets ScheduledSDS to the clicked station, attached chopper begins automated delivery loop (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)
