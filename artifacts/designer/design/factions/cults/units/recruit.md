# Recruit

The Recruit is the Cults' basic worker unit, automatically produced by Recruitment Halls from the surrounding Recruitable population. Recruits gather Space Crystals, construct buildings from outside the site, assist in-progress construction, can enter an Armory to be trained into Militants or Gunners, can enter an Altar to be augmented, can enter a Cathedral to be trained into Disciples, staff research buildings such as the Temple, Gunworks, and Machine Shop (only Recruits can staff research), and power the Shrine by joining its ritual. Recruits have a melee attack for basic self-defense.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - LightInfantry

## Silhouette - 24x24 space units (square)
## MaxHP - 40
## PointArmor - 0
## FullArmor - 0
## SightRange - 5 grid units
## Destructible - true
## Groupable - true (user, 2026-11-24: Recruits share a build, so Construct and Assist from the panel go to every selected Recruit; see systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting)

## UnitBaseAttributes[LightInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

## TurretAttributes - None

## AttackAttributes:
- AttackType: FullyConnected (Melee)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 3
- Range: melee (adjacent contact)
- AimDuration: 2 frames
- FiringDuration: 8 frames
- CooldownDuration: 1 frame
- ReloadDuration: 1 frame

## Gathering

Recruits gather Space Crystals and deliver them to Storage.

- CarryCapacity: 2 Space Crystals per load
- MiningDuration: 8 frames
- DropOffDuration: 2 frames
- MiningReach: 8 space units — a Recruit can mine once its Silhouette is within 8 su of the patch's edge.
- DropOffReach: 8 space units — a Recruit can drop off once its Silhouette is within 8 su of the Storage's edge.
(Reach is a per-unit stat; 8 su is a placeholder currently shared by all workers.)
- Expected throughput: buildings snap to whole tiles, so a patch can't touch a Storage with room for Recruits to work; the best layout leaves a 1-tile (64 su) gap. With a 24 su Silhouette and 8 su reach on each side, a Recruit walks 24 su (6 frames) each way, so a round trip is 22 frames (6 + 8 mining + 6 + 2 drop-off) ≈ 1.45 SC/s per Recruit. A 64 su gap fits two Recruits side by side (one lane in, one lane out).
- **Crowding** (user, 2026-11-26; Q0110): Recruits keep normal collision while gathering, and nobody gives way. If more Recruits work one face than it has lanes, they can block each other head-on for a long time. **The player's answer is more Storages** (and spreading Recruits over patches): add Storages when you want more Recruits mining at once. A Recruitment Hall's rally on a patch sends every new Recruit there, so it is the player's job to redistribute them.
- Patch sharing: any number of Recruits may mine the same Space Crystals patch at the same time (unlike Drones, see Colonists > Drone > Patch Occupancy).

## Repair
(user, 2026-11-26) Recruits can **repair** own **buildings and vehicles**. Repair **costs Space Crystals**. **Repair rules shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, paid gradually as HP is restored (exceptions below). Panel letter: **D** (Q0095).

## Building

Recruits construct Cults buildings from outside the site and can assist in-progress construction (see Cults Building Mechanics). Recruits are not consumed by construction.

## Constructs:
- RecruitmentHall: 300 Space Crystals, 1440 frames (90 seconds) BaseConstructionTime (with 1 Recruit)
- Storage: 100 Space Crystals, 320 frames (20 seconds) BaseConstructionTime (with 1 Recruit)
- Armory: 200 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (with 1 Recruit). Requires: Storage.
- Altar: 150 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (placeholder). Requires: Storage.
- Temple: 200 Space Crystals, 640 frames (40 seconds) BaseConstructionTime (placeholder). Requires: Altar.
- Cathedral: 250 Space Crystals, 640 frames (40 seconds) BaseConstructionTime (placeholder). Requires: Temple.
- Shrine: 200 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (placeholder). Requires: Temple.
- Garrison: 150 Space Crystals, 400 frames (25 seconds) BaseConstructionTime (placeholder). Requires: Armory.
- Gunworks: 150 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (placeholder). Requires: Armory.
- Chop Shop: 200 Space Crystals, 640 frames (40 seconds) BaseConstructionTime (placeholder). Requires: Armory.
- Machine Shop: 150 Space Crystals, 480 frames (30 seconds) BaseConstructionTime (placeholder). Requires: Chop Shop.
- Airstrip: 250 Space Crystals, 640 frames (40 seconds) BaseConstructionTime (placeholder). Requires: Chop Shop.
- Scrap Hangar: 200 Space Crystals, 640 frames (40 seconds) BaseConstructionTime (placeholder). Requires: Airstrip.

## Armory Entry

Recruits can enter an Armory to join its internal StoredRecruits pool (up to InternalRecruitCapacity), making them available for training into Militants or Gunners.

## Recruit ObjectInterfaceState

### DefaultState commands:
(Letters: user, 2026-11-26; Q0095. Same layout as the Agent's panel.)
- **Q: Move** — as in BasicCombatUnitInterfaceState.
- **A: Construct** — enters BuildMenu (StateOnlyTransition). Multi-stage: select a building type, then place it on the ground (ghost preview, same placement flow as the Build Command).
- **S: Assist Construction** — targeted command (StateOnlyTransition). Left-click an in-progress own building to join construction.
- **D: Repair** — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair it (see Repair).
- **X: Stop** — as in BasicCombatUnitInterfaceState.

### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button X
- **Attack** — right-click enemy unit/building (melee)
- **Enter Armory** — right-click own Armory
- **Enter Altar** — right-click own Altar
- **Enter Cathedral** — right-click own Cathedral
- **Enter Temple** (staff research) — right-click own Temple
- **Enter Shrine** (join the ritual) — right-click own Shrine
- **Enter Gunworks** / **Enter Machine Shop** (staff research) — right-click own Gunworks / Machine Shop
- **Gather** — right-click Space Crystal patch
- **Drop Off Resources** — right-click own Storage (when carrying Space Crystals)
- **Construct** — via panel button A
- **Assist Construction** — via panel button S, or right-click
- **Repair** — via panel button D, or right-click

### Right-Click Resolution:
- Space Crystal patch → Gather
- Own Storage (carrying Space Crystals) → Drop off resources
- Own Armory → Enter Armory
- Own Altar → Enter Altar
- Own Cathedral → Enter Cathedral
- Own Temple → Enter Temple (staff research)
- Own Shrine → Enter Shrine (join the ritual)
- Own Gunworks / Machine Shop → Enter (staff research)
- In-progress own building → Assist Construction
- Damaged own building or vehicle (completed) → Repair (for a damaged building that can also be entered, see the open question in Repair)
- Enemy unit/building → Attack (melee)
- Ground → Move

Recruits are Groupable: all selected Recruits form one SelectionGroup, and both right-click and panel commands (Construct, Assist Construction, Repair) are issued to all of them. Construct sends every selected Recruit to build the **same** building together (factions/cults/economy.md > Build Command). To build with a single Recruit, select it alone first (e.g. click its portrait).
