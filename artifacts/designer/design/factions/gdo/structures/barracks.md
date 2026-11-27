# Barracks
GDO infantry production structure. Produces infantry units from a build queue. Units exit from one of the short sides (B side, the right end in the default orientation), through the exit doorway (core/entities.md > Structure Type > Exit Doorway).

## Faction - GlobalDefenseOrganization
## Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
## SymmetryType - ABAC (top A, right B, bottom A, left C; core/entities.md > Structure Type > Side lettering convention)
## MaxHP - 300
## PointArmor - 1
## FullArmor - 6
## SightRange - 4 grid units
## Destructible - true
## Groupable - true
## BuildRadiusExtension - 2 grid units
## Power - -30

## BarracksInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

## Destroyed while producing (user, 2026-11-24):
If a Barracks is destroyed, its whole BuildQueue (including the unit in production) is refunded **as if cancelled**: full Space Crystals cost, and Unit Control released for the unit in production. The same applies to every GDO producer and constructor (e.g. an Extraction Facility constructing a plate, an Airfield's queue). Matches research (core/tech_and_research.md > Research > Shared Rules). This includes a unit that is **finished but still fully inside** (production at 100%, waiting because the exit doorway is occupied; core/entities.md > Structure Type > Exit Doorway): it is refunded like the rest of the queue (user, 2026-11-24). A unit already standing in the doorway is a normal unit and is not affected.

## RallyPoint behavior:
When a unit finishes production and spawns, if RallyPoint is set, it is issued its own default right-click command resolved against the RallyPoint target (systems/controls/interface.md > CommandPanel > What a rally point makes a new unit do). E.g. rally on ground -> Move to location, rally on enemy -> Attack. If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None. Rally points on enemy objects turn into ground rally points when the enemy leaves sight (systems/controls/interface.md > CommandPanel > Rally points on enemy objects).

## Produces:
- Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)
- Rocketman: 100 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at least one Arsenal.
- Drone Operator: 100 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at least one Career Campus.
- Field Engineer: 75 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at least one Career Campus.

## ObjectInterfaceState[Barracks]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Peacekeeper**: deducts 50 Space Crystals from player, adds Peacekeeper to BuildQueue. Only available if BuildQueue has fewer than 20 entries and player has sufficient Space Crystals (Unit Control is checked when production starts: factions/gdo/overview.md > Unit Control).
- **W: Build Rocketman** (hotkey tentative): deducts the Rocketman's Space Crystals cost (100, placeholder) from player, adds Rocketman to BuildQueue. Only available if the player owns at least one Arsenal, BuildQueue has fewer than 20 entries, and player has sufficient Space Crystals (Unit Control is checked when production starts: factions/gdo/overview.md > Unit Control).
- **E: Build Field Engineer** (placeholder letter): deducts 75 Space Crystals and 1 Supply (Q0169), adds a Field Engineer to BuildQueue. Only available if the player owns at least one Career Campus, BuildQueue has fewer than 20 entries and the player has sufficient Space Crystals and Supplies (Unit Control is checked when production starts, as for Q and W: factions/gdo/overview.md > Unit Control).
- **R: Build Drone Operator** (placeholder letter, designer 2026-11-26; Q0155): deducts 100 Space Crystals and 1 Supply (Q0169; user, 2026-11-26), adds a Drone Operator to BuildQueue. Only available if the player owns at least one Career Campus, BuildQueue has fewer than 20 entries and the player has sufficient Space Crystals and Supplies (Unit Control is checked when production starts, as for Q, W and E).
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).
