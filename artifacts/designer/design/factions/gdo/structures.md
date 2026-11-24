# GDO: Structures

## Barracks
GDO infantry production structure. Produces infantry units from a build queue. Units exit from one of the short sides (B side, the right end in the default orientation), through the exit doorway (core/entities.md > Structure Type > Exit Doorway).

### Faction - GlobalDefenseOrganization
### Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
### SymmetryType - ABAC (top A, right B, bottom A, left C; core/entities.md > Structure Type > Side lettering convention)
### MaxHP - 300
### PointArmor - 1
### FullArmor - 6
### SightRange - 4 grid units
### Destructible - true
### Groupable - true
### BuildRadiusExtension - 2 grid units
### Power - -30

### BarracksInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### Destroyed while producing (user, 2026-11-24):
If a Barracks is destroyed, its whole BuildQueue (including the unit in production) is refunded **as if cancelled**: full Space Crystals cost, and Unit Control released. The same applies to every GDO producer and constructor (e.g. an Extraction Facility constructing a plate, an Airfield's queue). Matches research (core/tech_and_research.md > Research > Shared Rules). This includes a unit that is **finished but still fully inside** (production at 100%, waiting because the exit doorway is occupied; core/entities.md > Structure Type > Exit Doorway): it is refunded like the rest of the queue (user, 2026-11-24). A unit already standing in the doorway is a normal unit and is not affected.

### RallyPoint behavior:
When a unit finishes production and spawns, if RallyPoint is set, it is issued the default right-click command resolved against the RallyPoint target. Rally on ground -> Move to location, rally on enemy -> Attack, rally on friendly/neutral object -> Move to object. If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/selection_and_hud.md > Rally points on enemy objects).

### Produces:
- Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)
- Rocketman: cost/time TBD. Requires: player owns at least one Arsenal.
- Drone Operator: cost/time TBD. Requires: player owns at least one Career Campus.
- Field Engineer: cost/time TBD. Requires: player owns at least one Career Campus.

### ObjectInterfaceState[Barracks]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Peacekeeper**: deducts 50 Space Crystals from player, adds Peacekeeper to BuildQueue. Only available if BuildQueue has fewer than 20 entries and player has sufficient Space Crystals and Unit Control.
- **W: Build Rocketman** (hotkey tentative): deducts the Rocketman's Space Crystals cost (TBD) from player, adds Rocketman to BuildQueue. Only available if the player owns at least one Arsenal, BuildQueue has fewer than 20 entries, and player has sufficient Space Crystals and Unit Control.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

## DeploymentCenter
GDO primary construction structure. Constructs buildings one at a time, which must then be placed within the build radius. The build radius is the area around the Deployment Center and any placed GDO buildings, extended by each building's BuildRadiusExtension.

### Faction - GlobalDefenseOrganization
### Size - 4x4 grid units
### SymmetryType - AAAA
### MaxHP - 1000
### PointArmor - 1
### FullArmor - 16
### SightRange - 6 grid units
### Destructible - true
### Groupable - false
### BuildRadiusExtension - 12 grid units
### Power - +30

### DeploymentCenterInstanceState:
- CurrentConstruction: ObjectEnum | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ObjectEnum | None

### Constructs:
"Requires" clauses follow core/tech_and_research.md > Tech Prerequisites: a prerequisite building only counts once it has been **placed** on the map (a building sitting in ReadyToPlace does not count), and is checked only when construction is started.
- Generator: 100 Space Crystals, 240 frames (15 seconds)
- Barracks: 200 Space Crystals, 160 frames (10 seconds)
- Extraction Facility: 200 Space Crystals, 320 frames (20 seconds)
- Supply Tower: cost/time TBD. Requires: player owns at least one Radar Station.
- Arsenal: cost/time TBD. Requires: player owns at least one Barracks.
- Career Campus: cost/time TBD. Requires: player owns at least one Barracks.
- Factory: cost/time TBD. Requires: player owns at least one Barracks.
- Radar Station: cost/time TBD. Requires: player owns at least one Generator.
- Advanced Mechanics: cost/time TBD. Requires: player owns at least one Factory.
- Airfield: cost/time TBD. Requires: player owns at least one Advanced Mechanics.
- Propulsion Labs: cost/time TBD. Requires: player owns at least one Airfield.
- Industrial Powerplant: cost/time TBD. Requires: player owns at least one Advanced Mechanics.
- Defense System: cost/time TBD. Requires: player owns at least one Radar Station and at least one Arsenal.

### Cancellation:
- Cancel during construction: full refund of Space Crystals cost
- Cancel when ready to place: 75% refund of Space Crystals cost (rounded down)
- **Destroyed** while constructing or holding a ready building: refunded **as if cancelled** (full during construction, 75% rounded down when ready to place) (user, 2026-11-24)

### ObjectInterfaceState[DeploymentCenter]:

DefaultState commands:

Immediate commands (CommandIssuingTransition):
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition). Only visible when CurrentConstruction is set.
- **X: Cancel Ready Building**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition). Only visible when ReadyToPlace is set.

State commands (StateOnlyTransition):
- **Q: Build**: enters BuildMenu (StateOnlyTransition)

BuildMenu (instance idle — no CurrentConstruction, no ReadyToPlace):
- Generator: deducts 100 Space Crystals, sets CurrentConstruction = Generator (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Barracks: deducts 200 Space Crystals, sets CurrentConstruction = Barracks (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Extraction Facility: deducts 200 Space Crystals, sets CurrentConstruction = ExtractionFacility (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Supply Tower: deducts Space Crystals (TBD), sets CurrentConstruction = SupplyTower (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals and owns at least one Radar Station.
- Arsenal, Career Campus, Factory, Radar Station, Advanced Mechanics, Airfield, Propulsion Labs, Industrial Powerplant, Defense System: build menu entries TBD — costs, availability gating, and panel layout will be defined alongside each building's full stat block.
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

BuildMenu (instance constructing — CurrentConstruction is set):
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition, returns to DefaultState)
- Other build options unavailable
- **Z**: returns to DefaultState (StateOnlyTransition)

BuildMenu (building ready — ReadyToPlace is set):
- Select completed building: enters AwaitingPlacement (StateOnlyTransition)
- **X: Cancel Ready Building**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)
- Other build options unavailable
- **Z**: returns to DefaultState (StateOnlyTransition)

AwaitingPlacement:
- A ghost preview of the building follows the cursor, snapped to the grid. The ghost is tinted green when the placement is valid and red when invalid.
- The player's current GDO build area is highlighted as a semi-transparent overlay on the ground, showing all valid build cells.
- R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise. Non-square buildings swap footprint dimensions when rotated to 90 or 270. F flips the ghost horizontally, Shift+F flips vertically. Side labels (A/B/C/D per SymmetryType) are displayed on the ghost and update with rotation/flipping.
- Left-click valid location within build radius: places building at the ghost's position and rotation, clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to BuildMenu (StateOnlyTransition), building remains ready to place

## GDOBuildArea
A single shared buildable area per GDO player. Seeded by each Deployment Center's and each Outpost's BuildRadiusExtension and grown by every placed GDO building's BuildRadiusExtension. Field Engineer structures (Bunker, First Aid Tent) have BuildRadiusExtension 0 and never extend it — the Prospector/Outpost is GDO's expansion tool. (Outposts can be deployed outside the current build area, so they seed new, possibly separate, parts of it.) When placing a building, at least 1 grid cell of the new building must be within the current build area. After placement, the build area expands outward from the placed building by that building's BuildRadiusExtension in all directions.

## Arsenal
GDO combat research structure. Allows research of attack and weapon upgrades. Also unlocks the **Rocketman** (missile infantry) at the Barracks.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Barracks

(Full stats TBD)

## Career Campus
GDO advanced personnel structure. Allows the use of more advanced personnel — unlocks new units and ability research. Unlocks the **Drone Operator** and **Field Engineer** at the Barracks.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Barracks

(Full stats TBD)

## Factory
GDO vehicle production structure. Allows the building of vehicles.

### Produces (so far):
- Raptor: cost/time TBD.
- Twister: cost/time TBD.
- Prospector: cost/time TBD. Requires: player owns at least one Advanced Mechanics.
- Artillery Rig: cost/time TBD. Requires: player owns at least one Advanced Mechanics.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Barracks

(Full stats TBD)

## Radar Station
GDO reconnaissance structure. Has an ability to scan and grant vision of an area on the map on a cooldown.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Generator

(Full stats TBD, including scan ability details — area size, cooldown, duration)

## Advanced Mechanics
GDO vehicle upgrade structure. Allows the building of more advanced vehicles. Has upgrades for vehicles and buildings. Unlocks the **Prospector** (expansion vehicle) and the **Artillery Rig** at the Factory.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Factory

(Full stats TBD)

## Airfield
GDO aircraft production structure. Constructs aircraft and provides a landing/reloading zone for them.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Advanced Mechanics

### Produces (so far):
- Hornet: cost/time TBD.
- Dragonfly: cost/time TBD. Requires: player owns at least one Propulsion Labs.

### Aircraft Support
- **Limited ammo**: GDO aircraft carry limited ammo and **rearm at an Airfield** (the Airfield is their landing and reloading zone). Ammo per unit is on each aircraft (factions/gdo/units.md); return/rearm flow below.
- **Capacity: each Airfield supports at most 4 aircraft.** (User, 2026-11-23.)

#### Assignment
Every GDO aircraft is **assigned** to one Airfield (rules: user, 2026-11-23):
- An aircraft is assigned to the Airfield that built it.
- An aircraft lands and reloads only at its **assigned** Airfield.
- An Airfield holds at most **4 assignments**.
- **Reassigning**: the player can move an aircraft's assignment to another Airfield, if that Airfield has room: its assignments plus the aircraft in its production queue must be **fewer than 4** (queued aircraft hold their slots, so reassigning into them is blocked). Input: select the aircraft and **right-click the other Airfield**. The aircraft is reassigned and flies there to land.
- **No manual unassigning**: the player cannot unassign an aircraft (so aircraft can't be freely shuffled around).
- **Airfield destroyed**: its aircraft become **unassigned**. An unassigned aircraft can't land to reload until the player reassigns it. **There is no automatic reassignment**, even if another Airfield has room.

#### Production limit
An Airfield **can't queue another aircraft** if its assignments plus the aircraft already in its production queue would exceed 4. (E.g. 3 assigned + 1 queued = full.)

#### Landing slots
An Airfield has **4 landing slots**, one per possible assignment, so all of its aircraft can be landed at the same time.

#### Returning and rearming
- **Automatic return**: when an aircraft runs out of ammo, it automatically flies back to its assigned Airfield, lands and rearms.
- The automatic return **can be interrupted** by giving the aircraft any other order (e.g. to fly elsewhere). It then stays out of ammo until it returns.
- **Return to Base** command: sends the aircraft to its assigned Airfield to land and rearm (hotkey TBD). Right-clicking its own Airfield does the same.
- After rearming, the aircraft **stays landed** at the Airfield. It doesn't go back to where it was unless ordered.
- An unassigned aircraft has no base: it doesn't return automatically, and Return to Base is unavailable until it is reassigned.
- **Rearm time is per unit of ammo**, and the value differs per aircraft type (RearmTimePerAmmo on each aircraft, factions/gdo/units.md). Rearming an aircraft takes RearmTimePerAmmo × missing ammo (user, 2026-11-23).
- **Landed aircraft are ground targets** (user, 2026-11-23): while parked on the Airfield, an aircraft can be hit only by anti-ground weapons (TargetDomain Ground or Universal, systems/combat.md). Air-only weapons (TargetDomain Air) can't hit it. Once it takes off it is an air target again.

TBD: RearmTimePerAmmo values; landing/take-off time; Return to Base hotkey/slot.

(Full stats TBD)

## Industrial Powerplant
GDO power generation structure. Provides power to the grid in larger quantities than the Generator.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Advanced Mechanics

(Full stats TBD)

## Propulsion Labs
GDO advanced aircraft structure. Allows the building of more advanced aircraft and air units. Unlocks the **Dragonfly** at the Airfield.

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Airfield

(Full stats TBD)

## Defense System
GDO main defense-related structure. Constructs defensive structure(s).

### Faction - GlobalDefenseOrganization
### Constructed by - DeploymentCenter
### TechPrerequisite - player owns at least one Radar Station and at least one Arsenal

### Constructs:
- Missile Turret: cost/time TBD

(Full stats TBD)

## Missile Turret
GDO defensive structure. Attacks both air and ground targets (TargetDomain: Universal — see systems/combat.md).

### Faction - GlobalDefenseOrganization
### Constructed by - DefenseSystem
### TechPrerequisite - None

(Full stats TBD, including AttackAttributes)

## Bunker
GDO field defensive structure built by a Field Engineer. Infantry garrison inside and fire out.

### Entity Type - Structure Type
### Faction - GlobalDefenseOrganization
### Constructed by - Field Engineer (Build Bunker ability)
### BuildRadiusExtension - 0 (does not extend the GDO build area, so Field Engineers cannot use Bunkers to expand)

### Placement
Can be placed on **any Buildable terrain** — not limited to the GDO build area. Follows the Worker-Built Structures rules (core/entities.md > Placement Validation > Worker-Built Structures).

### Garrison
Works like the Cults Garrison (factions/cults/structures.md > Garrison), except as noted:
- **Who can enter**: all GDO infantry, up to **4 units**, using the Enter command.
- Units inside **fire their own weapons** from the Bunker. (Unlike the Cults Garrison, the Bunker gives no units a substitute weapon — an unarmed unit stays unarmed.)
- **No bonus range** for units inside (the Cults Garrison gives Soldiers and Gunners +1).
- **Protection**: units inside cannot be targeted and take no damage while the Bunker stands.
- **Evacuation on destruction**: when the Bunker is destroyed, everyone inside comes out alive around its footprint.
- Units inside are not consumed and still count toward Unit Control.
- **No abilities from inside**: units inside the Bunker cannot use their abilities (e.g. a Drone Operator cannot launch drones, a Rocketman cannot use Target Lock, a Peacekeeper cannot raise its Riot Shield). They only fire their normal weapons.

TBD: SightRange effect, cost, build time, Size, SymmetryType, MaxHP, armor, SightRange, ObjectInterfaceState (incl. eject commands).

## First Aid Tent
GDO support structure set up by a Field Engineer. Heals nearby infantry.

### Entity Type - Structure Type
### Faction - GlobalDefenseOrganization
### Constructed by - Field Engineer (Set up First Aid Tent ability)
### BuildRadiusExtension - 0 (does not extend the GDO build area)

### Healing
Heals **nearby infantry** (no need to enter). **Permanent** — it stays until destroyed.

### Placement
Can be placed on **any Buildable terrain**, like the Bunker (Worker-Built Structures rules).

TBD: heal radius, heal rate, whether it heals only GDO infantry, cost, build time, Size, SymmetryType, MaxHP, armor, SightRange.

## Outpost
GDO expansion structure created when a Prospector deploys. Its only function is to provide a **build radius**: it extends the GDO build area around it so the Deployment Center can place buildings there (including an Extraction Facility for a remote crystal field).

### Entity Type - Structure Type
### Faction - GlobalDefenseOrganization
### Created by - Prospector (Deploy into Outpost); not built by the Deployment Center
### Placement - any Buildable terrain (where the Prospector deploys)
### Power - consumes power (amount TBD)
### Permanent - cannot pack back up into a Prospector

Provides **only** the build radius — it does not construct anything, produce units, or act as a drop-off.

TBD: BuildRadiusExtension, power consumption, Size, SymmetryType, MaxHP, armor, SightRange.
