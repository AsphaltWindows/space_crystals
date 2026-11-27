# SupplyChopper
GDO supply transport air unit. An unarmed hovercraft that collects supplies from Supply Delivery Stations and delivers them to Supply Towers. Automatically picks up all available supplies when landing on a Supply Delivery Station.

## Cargo
A Supply Chopper carries one of the following at a time:
- Up to 4 infantry units
- A single vehicle
- Supplies

**Capacity** (user, 2026-11-26): each infantry unit (Light or Heavy) takes one of the 4 places; any one vehicle fits. One kind at a time: it can't load units while carrying Supplies, nor pick up Supplies while carrying units.

## Unit Transport
(user, 2026-11-26: "it lands to load and unload and loads and unloads instantly like the Subterranean Transport")
- **Loading**:
  - Right-click the Chopper with own infantry or a vehicle selected (**Enter**): the Chopper **lands where it is**, the units walk to it, and each **gets in instantly** on reaching it.
  - With the Chopper selected, right-click an own infantry unit or vehicle (**Load**): it flies over, lands beside it, and the unit walks in.
  - Units that don't fit stay outside (idle).
- **Unloading**: **F: Unload All** (targeted; Shift-queues like any command): left-click a spot; the Chopper flies there, lands, and **everyone comes out at once** around it.
- **Landing**: only on ground that infantry can walk on, with no building or unit in the way (if the spot is blocked it lands at the nearest free spot). **Landing and taking off take 16 frames (1 s) each** (as at the Airfield): it is an **air target** while landing and taking off, and a **ground target** while landed.
- **It takes off by itself** once loading or unloading is done (user, 2026-11-26).
- **Passengers can't fire out** (as the Subterranean Transport's).
- **Destroyed with passengers** (user, 2026-11-26): **everyone inside dies**, landed or in the air, as with the Subterranean Transport.

## Faction - GlobalDefenseOrganization
## Silhouette - 60x60 space units (square)
## SeparationRadius - 68 space units (designer placeholder, 2026-11-27; Q0181: Silhouette side + 8 su)
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
- Right-click own infantry unit or vehicle: issues **Load** (Unit Transport). Only if the chopper is not carrying Supplies and has room.
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
- **F: Unload All** (placeholder letter, as the Subterranean Transport's): enters AwaitingTarget[Unload]; left-click a spot to fly there, land and unload. Only available if the chopper is carrying units.

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
