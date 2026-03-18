# Ticket: GDO Supply Tower and Supply Chopper

## Current State
No GDO supply collection or transport system exists. There is no mechanism to collect supplies from Supply Delivery Stations.

## Desired State
Implement the Supply Tower and Supply Chopper:

**Supply Tower** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 3x3, Symmetry: AAAA, MaxHP: 400, PointArmor: 1, FullArmor: 9, SightRange: 4
- BuildRadiusExtension: 1, Power: -15
- TechPrerequisite: player owns >= 1 Power Plant
- SupplyTowerInstanceState: AttachedChopper (ObjectInstance|None), LandedChopper (ObjectInstance|None), ScheduledSDS (ObjectInstance|None), BuildQueue (array, max 5), CurrentBuild (ObjectEnum|None), CurrentBuildProgress (frames|None)
- Produces: Supply Chopper (100 SC, 160 frames / 10s). Build queue max 5.
- On placement: spawns one free Supply Chopper, automatically attached

**Landing and Attachment**:
- Attached chopper landing on its tower: drops off all supplies + gradual free repair
- When a chopper attaches, it evicts any non-attached landed chopper
- When tower has an attached chopper, no other chopper may land on it
- Any command to the attached chopper breaks its attachment
- Chopper must not carry units to attach

**Scheduled Deliveries**:
- Tower with attached chopper may schedule from a specific SDS
- Chopper auto-departs timed to arrive when delivery lands at SDS
- After pickup, returns to tower, drops off, waits for next departure
- If distance too long: departs immediately after drop-off
- Multiple towers same SDS: closest ready tower gets priority, one chopper in flight per SDS at a time

**Supply Tower Interface**:
- Build Supply Chopper (CommandIssuingTransition): deducts 100 SC, adds to queue. Available if queue < 5, sufficient SC.
- Cancel Production (CommandIssuingTransition): removes last queue entry, full refund. Available if queue not empty.
- Schedule Deliveries (StateOnlyTransition -> AwaitingTarget[ScheduleDeliveries]): available only if tower has attached chopper
- AwaitingTarget[ScheduleDeliveries]: left-click SDS -> sets ScheduledSDS, begins delivery loop (CommandIssuingTransition -> DefaultState). Left-click anything else -> no action. Escape/right-click -> DefaultState.

**Supply Chopper** (Unit - HoverCraft):
- Faction: GlobalDefenseOrdinance
- Silhouette: 60x60 space units, MaxHP: 150, PointArmor: 1, FullArmor: 1, SightRange: 5
- Unarmed (no AttackAttributes, no TurretAttributes)
- UnitBase: HoverCraft
- UnitBaseAttributes: ForwardAccel 0.9 su/f^2, OmniAccel 0.1 su/f^2, DragRatio 0.1/f, TurnRate 10 deg/f
- SupplyChopperInstanceState: CarriedSupplies (number), AttachedTower (ObjectInstance|None)
- Supply Pickup: landing on SDS automatically picks up all CurrentSupplies

**Supply Chopper Interface**:
- Right-click Ground -> Move. Right-click SDS -> PickUpSupplies. Right-click own SupplyTower -> AttachToTower. Right-click other -> Move.
- Immediate: Stop, HoldPosition
- Target: Move -> AwaitingTarget[Move], PickUpSupplies -> AwaitingTarget[PickUpSupplies], AttachToTower -> AwaitingTarget[AttachToTower]
- AwaitingTarget[Move]: same as BasicCombatUnit Move resolution (ground -> Move, object -> Move to object)
- AwaitingTarget[PickUpSupplies]: left-click SDS -> PickUpSupplies. Else no action.
- AwaitingTarget[AttachToTower]: left-click own Supply Tower (no other attached chopper) -> AttachToTower. Else no action.

## Justification
Defined in `features/gdo_objects.md` (SupplyTower and SupplyChopper sections). The supply chain is a core GDO economy mechanic. Supply Delivery Stations provide periodic supplies that must be actively collected by choppers — this creates a logistics gameplay loop distinct from passive Space Crystal mining.

## QA Steps
1. Build a Supply Tower via Deployment Center (requires owning a Power Plant). Verify 200 SC deducted.
2. Verify a free Supply Chopper spawns on the tower, automatically attached.
3. Select the Supply Tower. Verify "Build Supply Chopper", "Cancel Production", and "Schedule Deliveries" are available.
4. Build a Supply Chopper (100 SC). Verify it enters the queue and SC is deducted.
5. Cancel production. Verify last entry removed and 100 SC refunded.
6. Queue 5 choppers. Verify build button becomes unavailable.
7. Click "Schedule Deliveries". Verify AwaitingTarget mode. Left-click a Supply Delivery Station. Verify ScheduledSDS is set.
8. Verify the attached chopper departs timed to arrive at the SDS when the next delivery lands.
9. Verify the chopper lands at SDS and picks up all CurrentSupplies automatically.
10. Verify the chopper returns to tower, drops off supplies, and waits for next departure.
11. Select the Supply Chopper. Verify commands: Move, Stop, HoldPosition, PickUpSupplies, AttachToTower. Verify no attack commands (unarmed).
12. Right-click empty ground with chopper selected. Verify Move command issued.
13. Right-click an SDS with chopper selected. Verify PickUpSupplies command issued.
14. Right-click own Supply Tower. Verify AttachToTower command issued.
15. Issue any command to the attached chopper. Verify attachment breaks.
16. Manually fly the chopper back and use AttachToTower. Verify it re-attaches.
17. Land a second (non-attached) chopper on a tower that has an attached chopper. Verify it is evicted.
18. Set up two Supply Towers with attached choppers scheduled to the same SDS. Verify only one chopper flies at a time (closest ready tower gets priority).
19. Verify the chopper departs immediately after drop-off if the SDS delivery frequency is too fast for the travel distance.
20. Destroy the attached chopper's target tower. Verify chopper becomes unattached.

## Expected Experience
The Supply Tower should feel like a logistics hub. The free chopper on placement provides immediate utility. Scheduled deliveries should feel automated — the player sets up the route and the chopper handles timing. The chopper itself should feel like a utility unit: responsive movement with HoverCraft physics (momentum-based, drag-affected), no combat capability. Right-click context sensitivity (SDS = pick up, tower = attach, ground = move) should feel natural. The one-chopper-per-SDS constraint should prevent multiple choppers from racing to the same delivery.
