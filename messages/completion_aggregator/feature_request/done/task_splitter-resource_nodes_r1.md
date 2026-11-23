# resource_nodes_r1

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# resource_nodes_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Rework for resource_nodes. Most functionality works correctly (nodes appear, correct sizes, info panels show data, depletion works, no owner/vision). Two issues remain, both related to fog of war visibility:

**1. SpaceCrystalsPatch RemainingAmount visible in fog**: When a crystal patch is in an explored-but-not-visible area, selecting it still shows RemainingAmount. Per design, RemainingAmount should only display when the patch is currently visible to the selecting player.

**2. SupplyDeliveryStation CurrentSupplies visible in fog**: Same issue — CurrentSupplies is shown even when the SDS is only explored, not visible. Per design, CurrentSupplies should only display when the SDS is visible. DeliverySize and DeliveryInterval should always show (this part works correctly).

Both issues stem from the InfoPanel not checking visibility state before displaying resource-specific data. The selection system allowing clicks on non-visible objects is tracked separately as a forum topic.

Reference: artifacts/designer/design/entities.md for SpaceCrystalsPatch and SupplyDeliveryStation InfoPanel rules.

## QA Instructions

1. Find a SpaceCrystalsPatch in an explored-but-not-visible area (fog of war). Select it — verify RemainingAmount is NOT shown.
2. Move a unit to make the same patch visible again. Select it — verify RemainingAmount IS shown.
3. Find a SupplyDeliveryStation in an explored-but-not-visible area. Select it — verify CurrentSupplies is NOT shown, but DeliverySize and DeliveryInterval ARE shown.
4. Move a unit to make the SDS visible. Select it — verify CurrentSupplies IS now shown alongside DeliverySize and DeliveryInterval.
