# infopanel_fog_visibility

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-infopanel_fog_of_war_visibility.md

## Task

Add fog-of-war visibility gating to InfoPanel fields for SpaceCrystalsPatch and SupplyDeliveryStation.

Currently, `update_selected_units_grid_system` in `ui/hud.rs` displays all InfoPanel fields unconditionally. Per design (`artifacts/designer/design/entities.md`):
- SpaceCrystalsPatch: RemainingAmount should only display when the entity is **Visible** to the selecting player
- SupplyDeliveryStation: CurrentSupplies (and the next-delivery countdown) should only display when **Visible**; DeliverySize and DeliveryInterval always display

### Implementation

1. Add `Res<FogOfWarMap>` and a `Transform` query for the selected entities to `update_selected_units_grid_system` parameters. Also need the local player ID (check how other systems get it — likely `Res<Player>` or similar).

2. For a selected entity, convert its world position to a grid tile coordinate (use the same conversion other systems use — likely `SPACE_UNITS_PER_GRID_UNIT` division + rounding).

3. Look up `fog_map.get(player_id, tile_x, tile_z)`. If the result is `VisibilityStateEnum::Visible`, display all fields normally. If `Explored`, hide visibility-gated fields (show "???" placeholder or omit them).

4. **SCP section** (~line 728-741 in hud.rs): Wrap the "Remaining" text, has_plate status, and depleted indicator in a visibility check. When not visible, show `"  Remaining: ???"` instead.

5. **SDS section** (~line 803-812 in hud.rs): Wrap the "Current" supplies line and the next-delivery countdown in a visibility check. When not visible, show `"  Current: ???"` instead. Keep "Size" and "Interval" lines unconditional.

6. Also check the ExtractionPlate mining section (~line 488-495) that shows remaining SC from the attached patch — this should also be gated by the patch's tile visibility.

### Key types and locations
- `FogOfWarMap`: `game/world/types.rs` — `.get(player_id, x, z) -> VisibilityStateEnum`
- `VisibilityStateEnum`: `shared/types.rs` — `Unexplored`, `Explored`, `Visible`
- `SpaceCrystalPatch`: `game/world/types.rs` — `remaining_amount`, `initial_amount`, `has_plate`
- `SupplyDeliveryStation`: `game/world/types.rs` — `current_supplies`, `delivery_size`, `delivery_interval`, `time_until_next_delivery`
- Grid conversion: `SPACE_UNITS_PER_GRID_UNIT` constant in `shared/types.rs`
- Local player: Look for `Player` resource or `LocalPlayer` — check how `update_minimap_system` or combat systems determine the selecting player
