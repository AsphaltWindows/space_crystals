# explored_selectability_rules

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-fog_of_war_selectability.md

## Task

Implement Explored fog-of-war selectability rules: objects in Explored areas should NOT be selectable, with one exception -- SupplyDeliveryStation is selectable by GDO players only.

### Current State

In `game/world/resources.rs`, both `selection_system` (~line 79) and `drag_box_system` (~line 273) have fog filters that allow ALL neutral entities (including SpaceCrystalPatch and SupplyDeliveryStation) to bypass the fog check entirely (`is_neutral` check at ~line 140 and ~line 344). This means neutrals are selectable even in Explored and Unexplored areas.

### Required Changes

1. **Click selection** (`selection_system` in `resources.rs`, ~line 129-145):
   - Remove the blanket `is_neutral` bypass for fog filtering
   - Neutral entities should only be selectable when the tile is `VisibilityStateEnum::Visible`, OR when the entity is a `SupplyDeliveryStation` AND the tile is `VisibilityStateEnum::Explored` AND the local player's faction is `FactionEnum::GlobalDefenseOrdinance`
   - The system already has `Option<&SupplyDeliveryStation>` in its query, so use `sds.is_some()` to identify SDS entities
   - Need to add `SelectedFaction` resource to the system params to check the player's faction

2. **Box selection** (`drag_box_system` in `resources.rs`, ~line 273):
   - Same logic change: neutral entities must pass the fog filter
   - The box selection query currently does NOT include `SupplyDeliveryStation` optional component -- add `Option<&SupplyDeliveryStation>` to the selectables query
   - Apply the same GDO+Explored exception for SDS entities
   - Need to add `SelectedFaction` resource to system params

3. **Fog filter logic** (both systems): The new filter for non-owned entities should be:
   - If tile is `Visible`: selectable (unchanged)
   - If tile is `Explored` AND entity is SDS AND player faction is GDO: selectable
   - Otherwise: not selectable (skip/continue)

4. **Tests**: Update the existing fog_filter tests (~line 1287-1375 in resources.rs) to reflect the new rules. Add tests for:
   - Neutral entity in Explored area: NOT selectable
   - SDS in Explored area + GDO player: selectable
   - SDS in Explored area + non-GDO player: NOT selectable
   - SDS in Unexplored area: NOT selectable (even for GDO)
   - Crystal patch in Explored area: NOT selectable

## Technical Context

### Files to Change

- **`artifacts/developer/src/game/world/resources.rs`** -- Primary file. Contains both selection systems and all fog filter tests.

### Key Types and Imports

All needed types are already available via the existing `use crate::types::*;` on line 1:
- `SelectedFaction(pub FactionEnum)` -- Resource, defined in `shared/types.rs:24`. Holds the local player's faction choice.
- `FactionEnum` -- Enum with variants `GlobalDefenseOrdinance`, `TheSyndicate`, `TheCults`, `Colonists`. Defined in `shared/types.rs:275`, re-exported via `game/types/factions.rs:6`.
- `Owner::is_neutral(&self) -> bool` -- Returns true when `self.0.is_none()`. Defined in `shared/types.rs:46`.
- `SupplyDeliveryStation` -- Component, already in the click-selection query (line 85) as `Option<&SupplyDeliveryStation>`.
- `VisibilityStateEnum` -- Enum with `Unexplored`, `Explored`, `Visible`. Already used in both systems.

No new imports needed. Just add `selected_faction: Res<SelectedFaction>` to both system parameter lists.

### Click Selection System (`selection_system`, line 79)

Current fog filter logic (lines 138-145):
```rust
let is_owned = owner.0 == Some(local_player.0);
let is_neutral = owner.is_neutral();
if !is_owned && !is_neutral {
    let grid = world_to_grid(entity_pos);
    if fog_map.get(local_player.0, grid.x, grid.z) != VisibilityStateEnum::Visible {
        continue;
    }
}
```

Replace with: remove the `!is_neutral` bypass, apply fog check to ALL non-owned entities, then add the SDS+GDO+Explored exception inside the fog check block. The `sds` variable is already destructured from the query on line 129 (`for (entity, transform, _bounds, scp, sds, owner) in selectables.iter()`).

New logic:
```rust
let is_owned = owner.0 == Some(local_player.0);
if !is_owned {
    let grid = world_to_grid(entity_pos);
    let visibility = fog_map.get(local_player.0, grid.x, grid.z);
    let selectable_through_fog = visibility == VisibilityStateEnum::Visible
        || (visibility == VisibilityStateEnum::Explored
            && sds.is_some()
            && selected_faction.0 == FactionEnum::GlobalDefenseOrdinance);
    if !selectable_through_fog {
        continue;
    }
}
```

### Drag Box System (`drag_box_system`, line 273)

The selectables query on line 280 does NOT include `SupplyDeliveryStation`:
```rust
selectables: Query<(Entity, &Transform, &Owner, Option<&Unit>, Option<&StructureInstance>), With<Selectable>>,
```

Add `Option<&SupplyDeliveryStation>` to the query tuple and destructure it in the loop on line 335. Also add `selected_faction: Res<SelectedFaction>` to system params.

Current fog filter logic (lines 343-349) has the same `!is_neutral` bypass. Apply the identical replacement pattern as the click selection system.

### Test Changes (lines 1287-1375)

The existing `fog_filter_allows` helper (line 1291) takes `(is_owned, is_neutral, visibility)` and returns `bool`. It needs a new signature to support the SDS+GDO exception:

```rust
fn fog_filter_allows(is_owned: bool, is_sds: bool, is_gdo_player: bool, visibility: VisibilityStateEnum) -> bool {
    if is_owned {
        return true;
    }
    if visibility == VisibilityStateEnum::Visible {
        return true;
    }
    if visibility == VisibilityStateEnum::Explored && is_sds && is_gdo_player {
        return true;
    }
    false
}
```

Update all existing tests that call `fog_filter_allows` to use the new signature. The `is_neutral` parameter is removed -- neutral entities now go through the same filter as enemies (no special bypass). Add the five new test cases specified in the task.

### Pattern to Follow

The existing fog filter tests (lines 1302-1375) are pure logic tests using the helper function. Follow the same pattern: test the helper with various combinations. No need for full ECS integration tests -- the helper mirrors the logic in both selection systems.

## Dependencies

- **sds_infopanel_visibility_rework** (sibling task in same feature): That task depends on this one completing first, since it assumes SDS is selectable in Explored state for GDO players. However, this task has no dependency on it -- this task is standalone and can be implemented independently.
- No other external dependencies. The `SelectedFaction` resource is already inserted during `setup_player_resources` (faction/setup.rs:15) so it will always be available at runtime.
