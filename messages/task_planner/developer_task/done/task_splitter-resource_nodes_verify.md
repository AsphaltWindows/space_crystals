# resource-nodes-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-resource-nodes.md

## Task

Verify that the resource node types (SpaceCrystalsPatch and SupplyDeliveryStation) are fully implemented per the design spec. Both resource types already exist in the codebase:

- **SpaceCrystalPatch** component in `game/world/types.rs` (~line 163) with `remaining_amount`, `initial_amount`, `has_plate`
- **SupplyDeliveryStation** component in `game/world/types.rs` (~line 172) with `delivery_size`, `delivery_interval`, `current_supplies`, `time_until_next_delivery`
- **spawn_space_crystal_patches** in `game/world/resources.rs` (~line 11) — spawns as indestructible, neutral, 1x1
- **spawn_supply_delivery_stations** in `game/world/resources.rs` (~line 542) — spawns as indestructible, neutral
- **sds_delivery_timer** in `game/world/resources.rs` (~line 602) — handles countdown and refill
- Info panel display in `ui/hud.rs` (~line 228) for both types
- ObjectEnum variants: `SpaceCrystalsPatch` and `SupplyDeliveryStation` in `shared/types.rs`
- Tests in `game/world/types.rs` for both component types

**Verification checklist:**
1. SpaceCrystalsPatch is indestructible (Destructible=false), unowned (Owner::neutral()), SightRange=0, Size 1x1
2. SupplyDeliveryStation is indestructible, unowned, SightRange=0, Size 2x2
3. SDS delivery countdown only begins when CurrentSupplies reaches 0
4. InfoPanel for SpaceCrystalsPatch shows RemainingAmount only when Visible (not Explored)
5. InfoPanel for SDS always shows DeliverySize and DeliveryInterval; shows CurrentSupplies only when Visible
6. Depleted patch disappears from the map (despawn when remaining_amount == 0)

If any of these are not correctly implemented, fix them. If all are correct, confirm with a task_completion.
