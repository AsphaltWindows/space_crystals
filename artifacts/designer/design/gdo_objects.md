# GDO Objects

## Peacekeeper
GDO basic infantry unit. A light infantry soldier armed with a fully connected ground attack.

### Faction - GlobalDefenseOrdinance
### Silhouette - 24x24 space units (square)
### MaxHP - 50
### PointArmor - 1
### FullArmor - 1
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### UnitBase - LightInfantry
### UnitControlCost - 1

### UnitBaseAttributes[LightInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

### TurretAttributes - None

### AttackAttributes:
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 2 frames
- ReloadDuration: 12 frames

### ObjectInterfaceState: BasicCombatUnitInterfaceState

## PowerPlant
GDO power generation structure. Provides power to the GDO global power grid.

### Faction - GlobalDefenseOrdinance
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

## Barracks
GDO infantry production structure. Produces infantry units from a build queue. Units exit from one of the short sides (B side).

### Faction - GlobalDefenseOrdinance
### Size - 3x2 grid units
### SymmetryType - ABAC
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
- BuildQueue: array of ObjectEnum (max 5)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### RallyPoint behavior:
When a unit finishes production and spawns, if RallyPoint is set, it is issued the default right-click command resolved against the RallyPoint target. Rally on ground -> Move to location, rally on enemy -> Attack, rally on friendly/neutral object -> Move to object. If RallyPoint is None, the unit spawns with no command. If the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None.

### Produces:
- Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)

### ObjectInterfaceState[Barracks]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Peacekeeper**: deducts 50 Space Crystals from player, adds Peacekeeper to BuildQueue. Only available if BuildQueue has fewer than 5 entries and player has sufficient Space Crystals and Unit Control.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

## DeploymentCenter
GDO primary construction structure. Constructs buildings one at a time, which must then be placed within the build radius. The build radius is the area around the Deployment Center and any placed GDO buildings, extended by each building's BuildRadiusExtension.

### Faction - GlobalDefenseOrdinance
### Size - 4x4 grid units
### SymmetryType - AAAA
### MaxHP - 1000
### PointArmor - 1
### FullArmor - 16
### SightRange - 6 grid units
### Destructible - true
### Groupable - false
### BuildRadiusExtension - 12 grid units
### Power - +20

### DeploymentCenterInstanceState:
- CurrentConstruction: ObjectEnum | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ObjectEnum | None

### Constructs:
- Power Plant: 150 Space Crystals, 160 frames (10 seconds)
- Barracks: 200 Space Crystals, 160 frames (10 seconds)
- Extraction Facility: 200 Space Crystals, 320 frames (20 seconds)
- Supply Tower: 200 Space Crystals, 240 frames (15 seconds). Requires: player owns at least one Power Plant.

### Cancellation:
- Cancel during construction: full refund of Space Crystals cost
- Cancel when ready to place: 75% refund of Space Crystals cost (rounded down)

### ObjectInterfaceState[DeploymentCenter]:

DefaultState commands:

Immediate commands (CommandIssuingTransition):
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition). Only visible when CurrentConstruction is set.
- **X: Cancel Ready Building**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition). Only visible when ReadyToPlace is set.

State commands (StateOnlyTransition):
- **Q: Build**: enters BuildMenu (StateOnlyTransition)

BuildMenu (instance idle — no CurrentConstruction, no ReadyToPlace):
- Power Plant: deducts 150 Space Crystals, sets CurrentConstruction = PowerPlant (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Barracks: deducts 200 Space Crystals, sets CurrentConstruction = Barracks (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Extraction Facility: deducts 200 Space Crystals, sets CurrentConstruction = ExtractionFacility (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals.
- Supply Tower: deducts 200 Space Crystals, sets CurrentConstruction = SupplyTower (CommandIssuingTransition, returns to DefaultState). Only available if player has sufficient Space Crystals and owns at least one Power Plant.
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
A single shared buildable area per GDO player. Seeded by each Deployment Center's BuildRadiusExtension and grown by every placed GDO building's BuildRadiusExtension. When placing a building, at least 1 grid cell of the new building must be within the current build area. After placement, the build area expands outward from the placed building by that building's BuildRadiusExtension in all directions.

## ExtractionFacility
GDO resource extraction structure. Constructs Extraction Plates onto Space Crystal Patches within the GDO player's build area. Can construct one plate at a time.

### Faction - GlobalDefenseOrdinance
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
- Extraction Plate: 75 Space Crystals, 96 frames (6 seconds)

### Cancellation:
- Cancel during construction: full refund of Space Crystals cost
- Cancel when ready to place: 75% refund of Space Crystals cost (rounded down)

### ObjectInterfaceState[ExtractionFacility]:

DefaultState (idle — no CurrentConstruction, no ReadyToPlace):
- **Q: Build Extraction Plate**: deducts 75 Space Crystals, starts construction (CommandIssuingTransition). Only available if player has sufficient Space Crystals.

DefaultState (constructing — CurrentConstruction is set):
- **X: Cancel Construction**: refunds full cost, clears CurrentConstruction (CommandIssuingTransition)

DefaultState (ready to place — ReadyToPlace is set):
- **Q: Place Plate**: enters AwaitingPlacement (StateOnlyTransition)
- **X: Cancel Ready Plate**: refunds 75% cost (rounded down), clears ReadyToPlace (CommandIssuingTransition)

AwaitingPlacement:
- A ghost preview of the Extraction Plate follows the cursor, snapped to the grid. The ghost is tinted green when over a valid Space Crystal Patch (within build area, no existing plate) and red otherwise.
- The player's current GDO build area is highlighted as a semi-transparent overlay on the ground.
- Left-click Space Crystal Patch within GDO build area that does not already have a plate: places Extraction Plate, clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to DefaultState (StateOnlyTransition), plate remains ready to place

## ExtractionPlate
GDO resource harvesting structure. Placed onto a Space Crystal Patch by an Extraction Facility. Mines Space Crystals from the patch at a fixed rate. When the patch is depleted, continues mining at a residual rate indefinitely.

### Faction - GlobalDefenseOrdinance
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

### InfoPanel: displays remaining Space Crystals in the underlying patch

### On destruction:
- If the underlying Space Crystal Patch still has resources: the patch becomes uncovered and accessible
- If the underlying Space Crystal Patch is depleted: the patch is removed from the map

### ObjectInterfaceState: None (info display only)

## SupplyTower
GDO supply collection structure. Serves as the drop-off point for supplies gathered by Supply Choppers. Produces Supply Choppers and comes with one free Supply Chopper when placed. A Supply Tower can have one Supply Chopper attached to it, enabling automated scheduled deliveries from a Supply Delivery Station.

### Faction - GlobalDefenseOrdinance
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
### TechPrerequisite - player owns at least one Power Plant

### SupplyTowerInstanceState:
- AttachedChopper: ObjectInstance | None (the Supply Chopper currently attached to this tower)
- LandedChopper: ObjectInstance | None (a Supply Chopper currently landed on this tower, may differ from AttachedChopper)
- ScheduledSDS: ObjectInstance | None (the Supply Delivery Station this tower's attached chopper is scheduled to collect from)
- BuildQueue: array of ObjectEnum (max 5 entries)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

### Produces:
- Supply Chopper: 100 Space Crystals, 160 frames (10 seconds)

### On Placement:
- One Supply Chopper is spawned on the tower at no additional cost (included in tower's construction cost). This chopper is automatically attached to the tower.

### Landing and Attachment:
- A Supply Chopper that lands on its attached Supply Tower drops off all carried supplies and is gradually repaired at no cost.
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
- **Q: Build Supply Chopper**: deducts 100 Space Crystals from player, adds SupplyChopper to BuildQueue. Only available if BuildQueue has fewer than 5 entries and player has sufficient Space Crystals.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).
- **S: Schedule Deliveries**: enters AwaitingTarget[ScheduleDeliveries]. Only available if tower has an attached chopper.

### AwaitingTarget[ScheduleDeliveries] resolution:
- Left-click SupplyDeliveryStation: sets ScheduledSDS to the clicked station, attached chopper begins automated delivery loop (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

## SupplyChopper
GDO supply transport air unit. An unarmed hovercraft that collects supplies from Supply Delivery Stations and delivers them to Supply Towers. Automatically picks up all available supplies when landing on a Supply Delivery Station.

### Faction - GlobalDefenseOrdinance
### Silhouette - 60x60 space units (square)
### MaxHP - 150
### PointArmor - 1
### FullArmor - 1
### SightRange - 5 grid units
### Destructible - true
### Groupable - true
### UnitBase - HoverCraft
### AttackAttributes - None
### TurretAttributes - None

### UnitBaseAttributes[HoverCraft]:
- ForwardAcceleration: 0.9 space units/frame squared
- OmniDirectionalAcceleration: 0.1 space units/frame squared
- DragRatio: 0.1 per frame
- TurnRate: 10 degrees/frame

### SupplyChopperInstanceState:
- CarriedSupplies: number (supplies currently being transported)
- AttachedTower: ObjectInstance | None (the Supply Tower this chopper is attached to)

### Supply Pickup:
- When a Supply Chopper lands on a Supply Delivery Station, it automatically picks up all CurrentSupplies from the station.

### ObjectInterfaceState[SupplyChopper]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues Move command to that location
- Right-click SupplyDeliveryStation: issues PickUpSupplies command (fly to SDS, land, pick up all available supplies). Only if chopper is not carrying units.
- Right-click own SupplyTower (when carrying supplies): issues DropOffSupplies command (fly to tower, land, drop off all carried supplies, immediately lift off — unless this is the chopper's attached tower)
- Right-click own SupplyTower (when not carrying supplies): issues AttachToTower command (fly to tower, land, attach). Only if chopper is not carrying units.
- Right-click other friendly/neutral Object: issues Move command to that object's location

Immediate commands (CommandIssuingTransition):
- Stop: issues Stop command
- Hold Position: issues HoldPosition command

Target commands (StateOnlyTransition):
- Move: enters AwaitingTarget[Move]
- Pick Up Supplies: enters AwaitingTarget[PickUpSupplies]. Only available if chopper is not carrying units.
- Attach to Tower: enters AwaitingTarget[AttachToTower]. Only available if chopper is not carrying units.
- Drop Off Supplies: enters AwaitingTarget[DropOffSupplies]. Only available if chopper is carrying supplies.

### AwaitingTarget[PickUpSupplies] resolution:
- Left-click SupplyDeliveryStation: issues PickUpSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[AttachToTower] resolution:
- Left-click own SupplyTower with no attached chopper: issues AttachToTower command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[DropOffSupplies] resolution:
- Left-click own SupplyTower with no attached chopper: issues DropOffSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

### AwaitingTarget[Move] resolution:
- Left-click ground: issues Move command to that location (CommandIssuingTransition, returns to DefaultState)
- Left-click object: issues Move command to that object's location (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to DefaultState (StateOnlyTransition)