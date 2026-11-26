# DeploymentCenter
GDO primary construction structure. Constructs buildings one at a time, which must then be placed within the build radius. The build radius is the area around the Deployment Center and any placed GDO buildings, extended by each building's BuildRadiusExtension.

## Faction - GlobalDefenseOrganization
## Size - 4x4 grid units
## SymmetryType - AAAA
## MaxHP - 1000
## PointArmor - 1
## FullArmor - 16
## SightRange - 6 grid units
## Destructible - true
## Groupable - false
## BuildRadiusExtension - 12 grid units
## Power - +30

## DeploymentCenterInstanceState:
- CurrentConstruction: ObjectEnum | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ObjectEnum | None

## Constructs:
"Requires" clauses follow core/tech_and_research.md > Tech Prerequisites: a prerequisite building only counts once it has been **placed** on the map (a building sitting in ReadyToPlace does not count), and is checked only when construction is started.
- Generator: 100 Space Crystals, 240 frames (15 seconds)
- Barracks: 200 Space Crystals, 160 frames (10 seconds)
- Extraction Facility: 200 Space Crystals, 320 frames (20 seconds)
- Supply Tower: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Radar Station.
- Arsenal: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Barracks.
- Career Campus: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Barracks.
- Factory: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at least one Barracks.
- Radar Station: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Generator.
- Advanced Mechanics: 200 Space Crystals, 400 frames (25 seconds) (placeholder). Requires: player owns at least one Factory.
- Airfield: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at least one Advanced Mechanics.
- Propulsion Labs: 200 Space Crystals, 400 frames (25 seconds) (placeholder). Requires: player owns at least one Airfield.
- Industrial Powerplant: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at least one Advanced Mechanics.
- Defense System: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Radar Station and at least one Arsenal.

## Cancellation:
- Cancel during construction: full refund of Space Crystals cost
- Cancel when ready to place: 75% refund of Space Crystals cost (rounded down)
- **Destroyed** while constructing or holding a ready building: refunded **as if cancelled** (full during construction, 75% rounded down when ready to place) (user, 2026-11-24)

## ObjectInterfaceState[DeploymentCenter]:

DefaultState commands:

Immediate commands (CommandIssuingTransition):
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition). Only visible when CurrentConstruction is set.
- **X: Cancel Ready Building**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition). Only visible when ReadyToPlace is set.

State commands (StateOnlyTransition):
- **Q: Basic Construction**: enters BuildMenu[Basic] (StateOnlyTransition)
- **W: Advanced Construction**: enters BuildMenu[Advanced] (StateOnlyTransition)
- **E: Place** (user, 2026-11-27; letter: designer placeholder): a shortcut so the player doesn't need to open the build menu to place. Shown whenever CurrentConstruction or ReadyToPlace is set, with the building's icon.
  - **While constructing**: greyed out, with a **running progress bar** on the button showing the construction's progress.
  - **When ready** (ReadyToPlace set): enabled; pressing it enters AwaitingPlacement for the ready building (StateOnlyTransition), exactly as the building's slot in the build menu does. Escape/right-click from there returns to DefaultState.
  - Hidden when the Deployment Center is idle.

**Two build menus** (user, 2026-11-24): the DC's buildings don't fit one 12-slot panel, so they are split. Names: "Basic Construction" / "Advanced Construction" ("Basic Structures" / "Advanced Structures" also acceptable to the user). Slot layout (fine for now, user):

BuildMenu[Basic]:
```
[Q] Generator        [W] Barracks   [E] Extraction Facility  [R] Supply Tower
[A] Radar Station    [S] Arsenal    [D] Career Campus        [F] Defense System
[Z] Back             [X] Cancel     [C] —                    [V] —
```
BuildMenu[Advanced] (the Factory and everything that follows it):
```
[Q] Factory                 [W] Advanced Mechanics  [E] Airfield  [R] Propulsion Labs
[A] Industrial Powerplant   [S] —                   [D] —         [F] —
[Z] Back                    [X] Cancel              [C] —         [V] —
```
Buildings whose prerequisites aren't met are shown greyed out in their slot. When a building is **ready to place**, its own slot (e.g. Q in BuildMenu[Basic] for a Generator) becomes "Select completed building" (enters AwaitingPlacement). The menu-state rules below (idle / constructing / ready) apply to both menus.

BuildMenu[Basic] / BuildMenu[Advanced] (instance idle — no CurrentConstruction, no ReadyToPlace), each entry in its slot above:
- Generator: deducts 100 Space Crystals, sets CurrentConstruction = Generator (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Barracks: deducts 200 Space Crystals, sets CurrentConstruction = Barracks (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Extraction Facility: deducts 200 Space Crystals, sets CurrentConstruction = ExtractionFacility (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Supply Tower: deducts 150 Space Crystals (placeholder), sets CurrentConstruction = SupplyTower (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals and owns at least one Radar Station.
- Arsenal, Career Campus, Factory, Radar Station, Advanced Mechanics, Airfield, Propulsion Labs, Industrial Powerplant, Defense System: deduct their cost and set CurrentConstruction the same way; costs as in Constructs (placeholders), availability per their Requires clause.
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
- The player's current GDO build area is highlighted as a semi-transparent overlay on the ground. The overlay shows the **build area only**; whether a given tile is valid right now (flat, Visible, unoccupied, not a resource) is shown per tile on the ghost (user, 2026-11-24).
- R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise. Non-square buildings swap footprint dimensions when rotated to 90 or 270. F flips the ghost horizontally, Shift+F flips vertically. Side labels (A/B/C/D per SymmetryType) are displayed on the ghost and update with rotation/flipping.
- Left-click valid location within build radius: places building at the ghost's position and rotation, clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to BuildMenu (StateOnlyTransition), building remains ready to place
