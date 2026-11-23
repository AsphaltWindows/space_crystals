# explored_selectability_rules

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-fog_of_war_selectability.md

## Task

Implement Explored fog-of-war selectability rules: objects in Explored areas should NOT be selectable, with one exception -- SupplyDeliveryStation is selectable by GDO players only.

### Current State

In `game/world/resources.rs`, both `selection_system` (~line 79) and `box_selection_system` (~line 280) have fog filters that allow ALL neutral entities (including SpaceCrystalPatch and SupplyDeliveryStation) to bypass the fog check entirely (`is_neutral` check at ~line 140 and ~line 344). This means neutrals are selectable even in Explored and Unexplored areas.

### Required Changes

1. **Click selection** (`selection_system` in `resources.rs`, ~line 129-145):
   - Remove the blanket `is_neutral` bypass for fog filtering
   - Neutral entities should only be selectable when the tile is `VisibilityStateEnum::Visible`, OR when the entity is a `SupplyDeliveryStation` AND the tile is `VisibilityStateEnum::Explored` AND the local player's faction is `FactionEnum::GlobalDefenseOrdinance`
   - The system already has `Option<&SupplyDeliveryStation>` in its query, so use `sds.is_some()` to identify SDS entities
   - Need to add `SelectedFaction` resource to the system params to check the player's faction

2. **Box selection** (`box_selection_system` in `resources.rs`, ~line 280):
   - Same logic change: neutral entities must pass the fog filter
   - The box selection query currently does NOT include `SupplyDeliveryStation` optional component -- add `Option<&SupplyDeliveryStation>` to the selectables query
   - Apply the same GDO+Explored exception for SDS entities
   - Need to add `SelectedFaction` resource to system params

3. **Fog filter logic** (both systems): The new filter for non-owned entities should be:
   - If tile is `Visible`: selectable (unchanged)
   - If tile is `Explored` AND entity is SDS AND player faction is GDO: selectable
   - Otherwise: not selectable (skip/continue)

4. **Tests**: Update the existing fog_filter tests (~line 1290-1328 in resources.rs) to reflect the new rules. Add tests for:
   - Neutral entity in Explored area: NOT selectable
   - SDS in Explored area + GDO player: selectable
   - SDS in Explored area + non-GDO player: NOT selectable
   - SDS in Unexplored area: NOT selectable (even for GDO)
   - Crystal patch in Explored area: NOT selectable
