# SupplyChopper
GDO supply transport air unit. An unarmed hovercraft that collects supplies from Supply Delivery Stations and delivers them to Supply Towers. Automatically picks up all available supplies when landing on a Supply Delivery Station.

## Cargo
A Supply Chopper carries one of the following at a time:
- Up to 4 infantry units
- A single vehicle
- Supplies

Unit transport mechanics (Load / Unload commands, landing to load, what counts as infantry vs vehicle for capacity, what happens to carried units if the chopper is destroyed) are TBD — deferred until unit design resumes.

## Faction - GlobalDefenseOrganization
## Silhouette - 60x60 space units (square)
## MaxHP - 150
## PointArmor - 1
## FullArmor - 1
## SightRange - 5 grid units
## Destructible - true
## Groupable - true
## UnitBase - HoverCraft
## AttackAttributes - None
## TurretAttributes - None

## UnitBaseAttributes[HoverCraft]:
- ForwardAcceleration: 0.9 space units/frame squared
- OmniDirectionalAcceleration: 0.1 space units/frame squared
- DragRatio: 0.1 per frame
- TurnRate: 10 degrees/frame

## SupplyChopperInstanceState:
- CarriedSupplies: number (supplies currently being transported)
- AttachedTower: ObjectInstance | None (the Supply Tower this chopper is attached to)

## Supply Pickup:
- When a Supply Chopper lands on a Supply Delivery Station, it automatically picks up all CurrentSupplies from the station.

## ObjectInterfaceState[SupplyChopper]:

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

## AwaitingTarget[PickUpSupplies] resolution:
- Left-click SupplyDeliveryStation: issues PickUpSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

## AwaitingTarget[AttachToTower] resolution:
- Left-click own SupplyTower with no attached chopper: issues AttachToTower command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

## AwaitingTarget[DropOffSupplies] resolution:
- Left-click own SupplyTower with no attached chopper: issues DropOffSupplies command (CommandIssuingTransition, returns to DefaultState)
- Left-click anything else: no action
- Escape/right-click: returns to DefaultState (StateOnlyTransition)

## AwaitingTarget[Move] resolution:
- Left-click ground: issues Move command to that location (CommandIssuingTransition, returns to DefaultState)
- Left-click object: issues Move command to that object's location (CommandIssuingTransition, returns to DefaultState)
- Escape/right-click: returns to DefaultState (StateOnlyTransition)
