# resource-nodes-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Verification Point 1: Indestructible, Neutral, SightRange=0, Correct Sizes — PASS
- `ObjectEnum::SpaceCrystalsPatch` in `artifacts/developer/src/game/types/objects.rs` line 302: `destructible: false, sight_range: 0, size: (1,1)` ✓
- `ObjectEnum::SupplyDeliveryStation` in `artifacts/developer/src/game/types/objects.rs` line 309: `destructible: false, sight_range: 0, size: (2,2)` ✓
- SCP spawn (`artifacts/developer/src/game/world/resources.rs` line 53-67): uses `ObjectInstance::indestructible()`, `Owner::neutral()` ✓
- SDS spawn (`artifacts/developer/src/game/world/resources.rs` line 575-590): uses `ObjectInstance::indestructible()`, `Owner::neutral()` ✓
- Tests at `artifacts/developer/src/game/world/types.rs` lines 1091-1096 verify sight_range is 0 for both.

### Verification Point 2: SDS Spawn Footprint — POTENTIAL GAP
- SDS is defined as Size 2x2 but the spawn code (resources.rs line 567-593) only marks **one tile** as non-traversible (the anchor tile). Compare with structures like Tunnel (4x4) which mark multiple tiles.
- The mesh is a Cylinder with radius 0.8 (diameter 1.6), and SelectionBounds is 1.6x0.2x1.6 — roughly 2x2 in world units.
- **Action needed**: Check if other 2x2 objects mark all 4 tiles. The SDS should mark the 2x2 footprint tiles as non-traversible/non-buildable. Look at how `spawn_grid` in `artifacts/developer/src/game/world/map.rs` handles multi-tile entities for reference.

### Verification Point 3: SDS Delivery Timer — PASS
- `sds_delivery_timer` at `artifacts/developer/src/game/world/resources.rs` lines 602-617: countdown only decrements when `sds.current_supplies == 0`. Refills to `delivery_size` when timer expires. ✓

### Verification Point 4 & 5: InfoPanel Visibility Checks — POTENTIAL GAP
- **SCP InfoPanel** (`artifacts/developer/src/ui/hud.rs` lines 669-742): displays RemainingAmount unconditionally. The design spec says 'when visible to the selecting player'. The info panel does NOT check FogOfWarMap visibility state.
- **SDS InfoPanel** (`artifacts/developer/src/ui/hud.rs` lines 744-815): displays DeliverySize, DeliveryInterval, AND CurrentSupplies unconditionally. The design spec says CurrentSupplies should only show 'when visible'.
- **However**: resource entities can only be selected when visible (selection_system in resources.rs line 79-109 uses FogOfWarMap). If you can only select a resource when it's visible, and the info panel only shows for selected entities, then visibility gating at the info panel level may be redundant. Check whether entities remain selected when they transition from Visible→Explored. If they do, the info panel would show stale data.
- **Key files to check/modify**: `artifacts/developer/src/ui/hud.rs` (the resource info panel blocks at lines 669-815), `artifacts/developer/src/game/world/types.rs` (FogOfWarMap, VisibilityStateEnum).

### Verification Point 6: Depleted Patch Despawn — POTENTIAL GAP
- The gathering system (`artifacts/developer/src/game/units/systems/behaviors.rs` line 688-689) does NOT decrement `remaining_amount` — it only adds to `carry_state.crystals`. Crystal depletion is handled by the Extraction Plate mining system in `artifacts/developer/src/game/world/faction.rs` line 504-507.
- Despawn on depletion only happens when an Extraction Plate is destroyed over a depleted patch (`artifacts/developer/src/game/combat/systems/core.rs` lines 573-576).
- There is NO auto-despawn when `remaining_amount` reaches 0 during normal extraction plate mining. The design says 'When a patch is fully depleted, it disappears from the map.'
- **Action needed**: Add a check in the extraction plate mining system (`artifacts/developer/src/game/world/faction.rs` ~line 507) to despawn the patch entity (and the plate) when `remaining_amount` reaches 0. Or add a separate system that queries `SpaceCrystalPatch` entities with `remaining_amount == 0` and despawns them.
- **Also**: The Agent gathering behavior doesn't deplete patches at all — agents pick up resources magically without reducing the patch. If agents are supposed to deplete patches too, that's another gap. But this may be by design (agents gather from a conceptual resource pool, plates actually mine the physical deposit).

### Key Files Summary
- `artifacts/developer/src/game/types/objects.rs` — ObjectType definitions (lines 302-315)
- `artifacts/developer/src/game/world/resources.rs` — spawn functions (lines 11-76 SCP, 542-599 SDS, 602-617 timer)
- `artifacts/developer/src/game/world/types.rs` — component definitions (lines 162-177), tests (lines 549-658, 1091-1096)
- `artifacts/developer/src/game/world/faction.rs` — extraction plate mining (line ~504)
- `artifacts/developer/src/game/combat/systems/core.rs` — plate destruction/patch despawn (lines 570-576)
- `artifacts/developer/src/ui/hud.rs` — info panel display (lines 669-815)
- `artifacts/developer/src/game/units/systems/behaviors.rs` — gathering behavior (lines 644-715)
- Design spec: `artifacts/designer/design/entities.md` lines 186-207

### Existing Test Coverage
- `artifacts/developer/src/game/world/types.rs`: 8 unit tests for SCP/SDS components (creation, depletion, mining, plate, countdown, refill, collect)
- `artifacts/developer/src/game/combat/systems/core.rs`: `depleted_patch_marked_for_removal_on_plate_destroy` (line 801), `extraction_plate_hud_display_depleted` (line 847)
- `artifacts/developer/src/ui/hud.rs`: `scp_selection_shows_in_groups` (line 1860), `sds_selection_shows_in_groups` (line 1872), `mixed_resource_and_unit_selection` (line 1884)

## Dependencies

None — this is a standalone verification task. All referenced systems are already implemented.
