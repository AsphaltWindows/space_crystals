# Feature: GDO Objects

## Overview
All concrete Global Defense Ordinance structures and units with their stats, production chains, and interface states.

## Design Sources
- `design/gdo_objects.md`
- `design/factions.md` (GDO resource system)

## Specifications

### GDOBuildArea
- Single shared buildable area per GDO player.
- Seeded by Deployment Center's BuildRadiusExtension (12).
- Grown by every placed building's BuildRadiusExtension.
- Placement rule: at least 1 grid cell of new building must be within current build area.
- After placement: area expands outward from placed building by its BuildRadiusExtension.

---

### Peacekeeper (Unit - LightInfantry)
- Silhouette: 24x24 space units
- MaxHP: 50, PointArmor: 1, FullArmor: 1, SightRange: 5
- UnitControlCost: 1
- Movement: TurnRate 180 deg/frame, Accel/Decel infinite, MaxSpeed 4 su/frame
- RuggedTerrainDefenseBonus: 50%
- Attack: FullyConnected (Ranged), Ground, SingleTarget, Damage 10, Range 4, MinRange 0
- Attack timing: Aim 2f, Fire 1f, Cooldown 2f, Reload 12f
- Interface: BasicCombatUnitInterfaceState

---

### Power Plant (Structure)
- Size: 2x2, Symmetry: AAAA, MaxHP: 350, Armor: 1/4, Sight: 3
- BuildRadiusExtension: 1, Power: +20
- Interface: None (info display only)
- Built by: Deployment Center (150 SC, 160 frames)

### Barracks (Structure)
- Size: 3x2, Symmetry: ABAC, MaxHP: 300, Armor: 1/6, Sight: 4
- BuildRadiusExtension: 2, Power: -30
- Units exit from B side
- **Produces**: Peacekeeper (50 SC, 80 frames / 5 seconds)
- Build queue: max 5. Cancel: refunds full cost of last entry.
- RallyPoint: uses right-click resolution (Ground->Move, Enemy->Attack, Friendly/Neutral->Move to object). Resets to None if target object destroyed.
- Interface: Build Peacekeeper, Cancel Production, right-click for rally

### Deployment Center (Structure)
- Size: 4x4, Symmetry: AAAA, MaxHP: 1000, Armor: 1/16, Sight: 6
- BuildRadiusExtension: 12, Power: +20, **Ungroupable**
- **Constructs** (one at a time):
  - Power Plant: 150 SC, 160 frames (10s)
  - Barracks: 200 SC, 160 frames (10s)
  - Supply Tower: 200 SC, 240 frames (15s). Requires: player owns >= 1 Power Plant.
- Cancellation: full refund during construction, 75% (rounded down) when ready to place.
- Interface flow: DefaultState -> BuildMenu -> AwaitingPlacement
  - BuildMenu options depend on instance state (idle/constructing/ready)
  - AwaitingPlacement: ghost preview, green/red tint, build area overlay, R/Shift+R rotation, F/Shift+F flipping (horizontal/vertical). Side labels (A/B/C/D per SymmetryType) displayed on ghost, updating with rotation/flipping.

### Extraction Facility (Structure)
- Size: 3x3, Symmetry: AAAA, MaxHP: 500, Armor: 1/9, Sight: 3
- BuildRadiusExtension: 2, Power: -15, **Ungroupable**
- **Constructs**: Extraction Plate (75 SC, 96 frames / 6 seconds)
- Same construct-then-place flow as Deployment Center (no sub-menu, only one product)
- Same cancellation rules (full refund during construction, 75% when ready)
- Plate placement: on Space Crystal Patch within GDO build area, no existing plate
- Interface: AwaitingPlacement with ghost on valid patches

### Extraction Plate (Structure)
- Size: 1x1, Symmetry: AAAA, MaxHP: 85, Armor: 2/2, Sight: 0
- BuildRadiusExtension: 0 (does NOT extend build area)
- MiningRate: 10 SC per 48 frames (3 seconds)
- ResidualMiningRate: 1 SC per 48 frames (when patch depleted)
- On destruction: patch uncovered if not depleted, patch removed if depleted
- Interface: None (info display only, shows remaining SC in underlying patch)

### Supply Tower (Structure)
- Size: 3x3, Symmetry: AAAA, MaxHP: 400, Armor: 1/9, Sight: 4
- BuildRadiusExtension: 1, Power: -15
- Tech prerequisite: player owns >= 1 Power Plant
- On placement: spawns one free Supply Chopper (attached automatically)
- **Produces**: Supply Chopper (100 SC, 160 frames / 10s). Build queue: max 5.
- **Attach mechanic**: One chopper attached at a time. Attached chopper drops off supplies + repairs when landed. Evicts non-attached landed choppers. Any command to attached chopper breaks attachment. Chopper must not carry units to attach.
- **Scheduled Deliveries**: Tower with attached chopper schedules from a specific SDS. Chopper auto-departs timed to arrive when delivery lands. If distance too long: departs immediately after drop-off. Multiple towers same SDS: closest ready tower goes first, one chopper in flight per SDS.
- Interface: Build Supply Chopper, Cancel Production, Schedule Deliveries (target SDS)

### Supply Chopper (Unit - HoverCraft)
- Silhouette: 60x60 space units
- MaxHP: 150, Armor: 1/1, Sight: 5
- **Unarmed** (no attack, no turret)
- Movement (DragMovement): ForwardAccel 0.9, OmniAccel 0.1, DragRatio 0.1, TurnRate 10 deg/frame
- Carries supplies. Automatically picks up all CurrentSupplies when landing on SDS.
- Interface: Move, Stop, Hold Position, Pick Up Supplies (target SDS), Attach to Tower (target own Supply Tower)
- Right-click: Ground->Move, SDS->PickUpSupplies, own SupplyTower->AttachToTower, other->Move

## Production Chain Summary
```
Deployment Center --> Power Plant, Barracks, Supply Tower
Barracks --> Peacekeeper
Extraction Facility --> Extraction Plate
Supply Tower --> Supply Chopper
```

## Dependencies
- `factions_and_resources` (GDO resource definitions)
- `unit_system` (LightInfantry, HoverCraft bases)
- `combat_system` (Peacekeeper's attack attributes)
- `control_system` (interface states, AwaitingPlacement, BasicCombatUnitInterfaceState)
- `entity_system` (SpaceCrystalsPatch, SupplyDeliveryStation interactions)
