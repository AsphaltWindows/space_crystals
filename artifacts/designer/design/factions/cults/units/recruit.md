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
## Groupable - true (user, 2026-11-24: Recruits share a build, so Construct and Assist from the panel go to every selected Recruit; see systems/controls/selection.md > SelectionGroup > No smart-casting)

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
- Expected throughput: buildings snap to whole tiles, so a patch can't touch a Storage with room for Recruits to work; the best layout leaves a 1-tile (64 su) gap. With a 24 su Silhouette and 8 su reach on each side, a Recruit walks 24 su (6 frames) each way, so a round trip is 22 frames (6 + 8 mining + 6 + 2 drop-off) ≈ 1.45 SC/s per Recruit. A 64 su gap fits two Recruits side by side (one lane in, one lane out). The Storage is 3x2 (2026-11-27), so its long faces are 3 tiles and its short faces 2.
- **Which Storage** (user, 2026-11-26; Q0102): at **every return** the Recruit takes its load to the own completed Storage **nearest to the Recruit itself** (edge to edge) **when it finishes mining**, so a new Storage is used at once (user, 2026-11-27; Q0136; replaces "nearest from its patch"). **Tie**: the Storage with the **earlier id** (built first). If the Storage is lost **during** a return, the replacement is again the one nearest **the Recruit** (the shared drop-off rule, 0170). With **no** completed Storage, the Recruit **idles holding its load**.
- **Crowding** (user, 2026-11-26; Q0110): Recruits keep normal collision while gathering, and nobody gives way. If more Recruits work one face than it has lanes, they can block each other head-on for a long time. **The player's answer is more Storages** (and spreading Recruits over patches): add Storages when you want more Recruits mining at once. A Recruitment Hall's rally on a patch sends every new Recruit there, so it is the player's job to redistribute them.
- Patch sharing: any number of Recruits may mine the same Space Crystals patch at the same time (unlike Drones, see Colonists > Drone > Patch Occupancy).

## Repair
(user, 2026-11-26) Recruits can **repair** own **buildings and vehicles** (and a **Pilgrim landed on an Airstrip**, user, 2026-11-27). Repair **costs Space Crystals**. **Repair rules shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does. **Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, paid gradually as HP is restored (exceptions below). A vehicle's price for this counts its **Space Crystals only**, not the Recruits it consumed (Seeker: 37.5 SC from 0 to full; Q0124). Panel letter: **D** (Q0095). **When repair ends** (user, 2026-11-26; Q0124): repairers **stop once the target is at full HP**; if the player **runs out of Space Crystals** mid-repair, the order **ends** (they stop; a free Tunnel repair never runs out). **Gliders can't be repaired** (they can't stop; user, 2026-11-26).

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
(Letters: user, 2026-11-26; Q0095/Q0096; rearranged 2026-11-27 (user, 2026-11-27: A is Attack on every unit, systems/controls/interface.md > CommandPanel > Standard Slot Assignments): **Q** Move, **W** Basic Construct, **E** Advanced Construct, **A** Attack, **S** Assist, **D** Repair, **X** Stop. Building is on W on every worker.)
- **Q: Move** — as in BasicCombatUnitInterfaceState.
- **W: Basic Construct** — enters BuildMenu[Basic] (StateOnlyTransition). Multi-stage: select a building type, then place it on the ground (ghost preview, same placement flow as the Build Command).
- **E: Advanced Construct** — enters BuildMenu[Advanced] (StateOnlyTransition), same flow.

  Two build menus (user, 2026-11-26; Q0096, "basic and advanced", as the GDO's Deployment Center). Fixed slots: every building keeps its key and is greyed until its TechPrerequisite is met, so no hotkey ever moves. Rows follow the branches: top row shared base, **A S D F row Tech/Weaponry, X C V row Mystical**; **Z** returns to DefaultState. (Split and slots: designer placeholder, 2026-11-26.)
  - **BuildMenu[Basic]**: **Q** Recruitment Hall, **W** Storage; **A** Armory, **S** Gunworks, **D** Garrison; **X** Altar.
  - **BuildMenu[Advanced]**: **A** Chop Shop, **S** Machine Shop, **D** Airstrip, **F** Scrap Hangar; **X** Temple, **C** Cathedral, **V** Shrine.
- **S: Assist Construction** — targeted command (StateOnlyTransition). Left-click an in-progress own building to join construction.
- **D: Repair** — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair it (see Repair).
- **A: Attack** (user, 2026-11-27: "recruits should be able to attack (melee attack)"; on A like every unit) — as BasicCombatUnitInterfaceState's Attack: enters AwaitingTarget[Attack]; left-click an enemy to attack it in melee, or ground to Attack Move.
- **X: Stop** — as in BasicCombatUnitInterfaceState.

### Unit Commands:
- **Move** — right-click ground
- **Stop** — panel button X
- **Attack** — right-click enemy unit/building (melee), or panel button A (also Attack Move on ground)
- **Enter Armory** — right-click own Armory
- **Enter Altar** — right-click own Altar
- **Enter Cathedral** — right-click own Cathedral
- **Enter Temple** (staff research) — right-click own Temple
- **Enter Shrine** (join the ritual) — right-click own Shrine
- **Enter Garrison** — right-click own Garrison
- Gunworks / Machine Shop: **not enterable until their research is designed** (0319); a right-click on them is a Move
- **Gather** — right-click Space Crystal patch
- **Drop Off Resources** — right-click own Storage (when carrying Space Crystals)
- **Construct** — via panel buttons A (Basic) and W (Advanced)
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
- Own Garrison → Enter Garrison
- Own Gunworks / Machine Shop → Move (not enterable until their research is designed, 0319)
- In-progress own building → Assist Construction
- Damaged own building or vehicle (completed) → Repair, **unless the building can be entered** by the selection: then **Enter** wins (user, 2026-11-26). Repair those with **D: Repair**. **Except when it is full** (user, 2026-11-27; Q0140): a damaged enterable building that is **full at the moment of the click** resolves to **Repair** (the pointer shows Repair). This applies to every building that holds units.
- Enemy unit/building → Attack (melee)
- Ground → Move

Recruits are Groupable: all selected Recruits form one SelectionGroup, and both right-click and panel commands (Construct, Assist Construction, Repair) are issued to all of them. Construct sends every selected Recruit to build the **same** building together (factions/cults/economy.md > Build Command). To build with a single Recruit, select it alone first (e.g. click its portrait).
