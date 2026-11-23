# fog_of_war_selectability

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

## Fog of War Selectability Rules & SupplyDeliveryStation NextDeliveryTime

Updated `artifacts/designer/design/entities.md` with two changes:

### 1. Explored Selectability Rules (new section under Explored)
Objects in Explored fog-of-war areas are **not selectable** by default. The one exception is **SupplyDeliveryStation**, which is selectable by **GDO players only**. This is grounded in lore (supplies are meant for GDO forces) and mechanics (SupplyTower scheduled delivery requires planning around delivery timing without active vision). Non-GDO players cannot select anything in Explored areas.

Previously, the implementation allowed full selection with full info for all objects in Explored areas. This must be changed to:
- All objects in Explored areas: not selectable (clicking does nothing)
- SupplyDeliveryStation in Explored areas + selecting player is GDO: selectable, shows restricted InfoPanel
- SupplyDeliveryStation in Explored areas + selecting player is non-GDO: not selectable

### 2. SupplyDeliveryStation InfoPanel Rework + NextDeliveryTime field
Added a new `NextDeliveryTime` field (frames until next delivery) and restructured the InfoPanel to be visibility-context-aware:
- **Visible** (any player): DeliverySize, DeliveryInterval, NextDeliveryTime, CurrentSupplies
- **Explored** (GDO only): DeliverySize, DeliveryInterval, NextDeliveryTime (no CurrentSupplies)
- **Explored** (non-GDO) / Unexplored: not selectable

## QA Instructions

1. **Setup**: Start a game as GDO. Scout an area containing a SupplyDeliveryStation and at least one enemy structure or resource patch, then withdraw your units so the area becomes Explored (fog of war returns).

2. **Verify non-selectability of structures/resources in Explored areas**: Click on an enemy structure shown in the Explored fog. Verify that nothing happens — no selection highlight, no InfoPanel displayed. Do the same for a SpaceCrystalsPatch in Explored fog. Confirm it is also not selectable.

3. **Verify SupplyDeliveryStation IS selectable for GDO**: Click on a SupplyDeliveryStation in the Explored fog while playing as GDO. Verify it IS selected and the InfoPanel appears. Confirm the InfoPanel shows DeliverySize, DeliveryInterval, and NextDeliveryTime. Confirm that CurrentSupplies is NOT shown.

4. **Verify SupplyDeliveryStation in Visible state**: Move a unit back into range of the SupplyDeliveryStation so it becomes Visible. Select it and confirm the InfoPanel now shows all four fields: DeliverySize, DeliveryInterval, NextDeliveryTime, AND CurrentSupplies.

5. **Verify non-GDO cannot select in Explored**: If multiplayer or faction-switching is available, play as Syndicate or another non-GDO faction. Scout a SupplyDeliveryStation, then withdraw. Click the station in Explored fog and verify it is NOT selectable — same as any other object in Explored areas.

6. **Verify Visible state still works for all players**: Regardless of faction, when a SupplyDeliveryStation is in the Visible state, any player should be able to select it and see the full InfoPanel with all fields.
