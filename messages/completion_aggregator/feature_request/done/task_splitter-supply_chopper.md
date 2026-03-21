# supply-chopper

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

## Content

Implement the SupplyChopper unit as defined in `artifacts/designer/design/gdo_objects.md`.

GDO supply transport air unit. An unarmed HoverCraft that collects supplies from Supply Delivery Stations and delivers them to Supply Towers.

**Entity Type:** Unit
**Faction:** GlobalDefenseOrdinance
**UnitBase:** HoverCraft
**AttackAttributes:** None (unarmed)
**TurretAttributes:** None

**Stats:**
- Silhouette: 60x60 space units (square)
- MaxHP: 150
- PointArmor: 1
- FullArmor: 1
- SightRange: 5 grid units
- Destructible: true
- Groupable: true

**UnitBaseAttributes[HoverCraft]:**
- ForwardAcceleration: 0.9 space units/frame^2
- OmniDirectionalAcceleration: 0.1 space units/frame^2
- DragRatio: 0.1 per frame
- TurnRate: 10 degrees/frame

**SupplyChopperInstanceState:**
- CarriedSupplies: number (supplies currently being transported)
- AttachedTower: ObjectInstance | None (the Supply Tower this chopper is attached to)

**Supply Pickup:**
When a Supply Chopper lands on a SupplyDeliveryStation, it automatically picks up all CurrentSupplies from the station.

**ObjectInterfaceState[SupplyChopper]:**

DefaultState right-click resolution:
- Ground: Move to location
- SupplyDeliveryStation: PickUpSupplies (fly to SDS, land, pick up all available supplies)
- Own SupplyTower: AttachToTower (fly to tower, land, attach). Only if chopper is not carrying units.
- Other friendly/neutral object: Move to object location

Immediate commands:
- Stop: issues Stop command
- HoldPosition: issues HoldPosition command

Target commands:
- Move: enters AwaitingTarget[Move]
- Pick Up Supplies: enters AwaitingTarget[PickUpSupplies]
- Attach to Tower: enters AwaitingTarget[AttachToTower]

AwaitingTarget[PickUpSupplies]: Left-click SDS -> issues PickUpSupplies command
AwaitingTarget[AttachToTower]: Left-click own SupplyTower -> issues AttachToTower command
AwaitingTarget[Move]: Left-click ground/object -> issues Move command

**Production:** Built at SupplyTower for 100 Space Crystals, 160 frames (10 seconds). One free chopper spawns with each newly placed SupplyTower.

## QA Instructions

1. Place a SupplyTower — verify a free Supply Chopper spawns on it, automatically attached.
2. Build an additional Supply Chopper from the SupplyTower (Q) — verify 100 crystals deducted, 10-second build.
3. Select the Supply Chopper — verify it has Move, Stop, HoldPosition, PickUpSupplies, AttachToTower commands but NO attack commands.
4. Right-click a SupplyDeliveryStation — verify chopper flies to it, lands, and picks up all CurrentSupplies.
5. Right-click own SupplyTower — verify chopper flies to tower, lands, and attaches.
6. Verify an attached chopper drops off carried supplies when landing on its tower.
7. Verify attached chopper is gradually repaired at no cost while landed.
8. Issue a command to an attached chopper — verify attachment breaks.
9. Verify chopper uses DragMovement (momentum-based, slides when changing direction).
10. Verify the chopper is unarmed (no attack capability).
11. Verify silhouette is 60x60 space units and MaxHP is 150.
