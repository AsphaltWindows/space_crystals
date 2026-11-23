# resource_nodes

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# resource-nodes

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the map resource node types as defined in `artifacts/designer/design/entities.md` under Resource, SpaceCrystalsPatch, and SupplyDeliveryStation.

**Resource (base type):**
An Object Type representing a map resource node. Resources are:
- Indestructible (Destructible=false)
- Unowned (Owner=None always)
- Provide no vision (SightRange=0)
They exist as contested map features that factions interact with through faction-specific mechanics.

**SpaceCrystalsPatch:**
A minable deposit of Space Crystals on the map.
- Size: 1x1 grid unit
- RemainingAmount: number (Space Crystals left in the patch)
- InfoPanel: when visible to the selecting player, displays RemainingAmount
- When fully depleted, the patch disappears from the map

**SupplyDeliveryStation (SDS):**
A location on the map that periodically receives supply deliveries.
- Size: 2x2 grid units
- DeliverySize: number (amount of supplies per delivery)
- DeliveryInterval: number (time between deliveries)
- CurrentSupplies: number (supplies currently on the station)
- The delivery countdown begins only once CurrentSupplies reaches 0
- InfoPanel: always displays DeliverySize and DeliveryInterval; when visible to the selecting player, also displays CurrentSupplies

## QA Instructions

1. Place SpaceCrystalsPatch nodes on a map and verify they appear as 1x1 grid objects.
2. Select a SpaceCrystalsPatch while it is Visible — verify InfoPanel shows RemainingAmount.
3. Select a SpaceCrystalsPatch while it is only Explored — verify RemainingAmount is NOT shown.
4. Deplete a SpaceCrystalsPatch — verify it disappears from the map when RemainingAmount reaches 0.
5. Verify SpaceCrystalsPatches cannot be attacked or destroyed (Indestructible).
6. Place a SupplyDeliveryStation on the map — verify it occupies 2x2 grid units.
7. Select an SDS — verify InfoPanel always shows DeliverySize and DeliveryInterval.
8. Select a Visible SDS — verify CurrentSupplies is also shown.
9. Wait for CurrentSupplies to reach 0, then verify the delivery countdown begins and after DeliveryInterval frames, CurrentSupplies refills to DeliverySize.
10. Verify neither resource type provides vision (SightRange=0) and neither has an owner.
