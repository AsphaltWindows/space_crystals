# Ticket: Resource Types (SpaceCrystalsPatch and SupplyDeliveryStation)

## Current State
No resource object types are defined. There is no way to represent harvestable resource nodes on the map.

## Desired State
Define two concrete resource Object Types and their instance data:

**Resource base properties** (shared by all resource types):
- Destructible: false
- Owner: always None
- SightRange: 0

**SpaceCrystalsPatch**:
- Size: 1x1
- Instance data: `RemainingAmount` (number of Space Crystals left)
- Behavior: disappears from the map when `RemainingAmount` reaches 0
- InfoPanel: when visible to the selecting player, displays `RemainingAmount`

**SupplyDeliveryStation**:
- Size: 2x2
- Instance data: `DeliverySize` (supplies per delivery), `DeliveryInterval` (time between deliveries), `CurrentSupplies` (current supply count)
- Behavior: delivery countdown begins when `CurrentSupplies` reaches 0
- InfoPanel: always displays `DeliverySize` and `DeliveryInterval`; when visible to the selecting player, also displays `CurrentSupplies`

Register both as entries in `ObjectEnum`.

## Justification
Required by `features/entity_system.md` (Resource Types section). These are the two universal resource types shared across all factions, providing the economic foundation for gameplay.

## QA Steps
1. Verify `SpaceCrystalsPatch` is registered in `ObjectEnum`.
2. Verify `SupplyDeliveryStation` is registered in `ObjectEnum`.
3. Verify a SpaceCrystalsPatch instance can be spawned with `RemainingAmount`, and that it has Size 1x1, Destructible=false, Owner=None, SightRange=0.
4. Verify a SupplyDeliveryStation instance can be spawned with `DeliverySize`, `DeliveryInterval`, and `CurrentSupplies`, and that it has Size 2x2, Destructible=false, Owner=None, SightRange=0.
5. Write a unit test that spawns a SpaceCrystalsPatch with RemainingAmount=100, reduces it to 0, and verifies the patch is despawned (or marked for removal).
6. Write a unit test that spawns a SupplyDeliveryStation with CurrentSupplies=0 and verifies the delivery countdown state is active/initialized.
7. Write a unit test confirming neither resource type can have an Owner assigned.

## Expected Experience
All unit tests pass. Both resource types spawn correctly with their required data. SpaceCrystalsPatch removal triggers on depletion. SupplyDeliveryStation delivery countdown activates when supplies reach zero. Neither resource type accepts an owner.
