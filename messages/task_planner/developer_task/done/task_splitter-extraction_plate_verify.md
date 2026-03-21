# extraction-plate-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-extraction-plate.md

## Task

Verify the existing ExtractionPlate implementation against the design spec. All code appears to be in place — this is a verification-only task.

**What exists (verify correctness of each):**

1. **ObjectEnum::ExtractionPlate** in game/types/objects.rs — ObjectType (1x1, destructible, sight_range=0, groupable=true), StructureType (AAAA, point_armor=2, full_armor=2)
2. **ExtractionPlateState** component in game/types/structures.rs — attached_patch entity ref, mining_timer counter
3. **Mining constants** in game/types/structures.rs — EXTRACTION_PLATE_MINING_RATE=10, EXTRACTION_PLATE_RESIDUAL_RATE=1, EXTRACTION_PLATE_MINING_INTERVAL=48
4. **extraction_plate_mining_system** in game/world/faction.rs — ticks mining_timer, mines from patch (capped at remaining), switches to residual when depleted, credits GDO player resources
5. **spawn_extraction_plate** in game/utils.rs — spawns with correct components (ObjectInstance HP=85, BuildRadiusExtension(0), ExtractionPlateState, GridPosition, etc.)
6. **On destruction** in game/combat/systems/core.rs remove_dead_entities_system — if patch has resources: sets has_plate=false (uncovers patch); if depleted: despawns patch entity
7. **depleted_patch_despawn_system** in game/world/faction.rs — despawns patches when remaining_amount=0
8. **EF construction cost** in game/types/structures.rs ExtractionFacilityState::construction_cost() — 75 SC, 96 frames
9. **Placement validation** in game/world/utils.rs can_place_building — special case for ExtractionPlate (must be on SpaceCrystalsPatch without existing plate)
10. **InfoPanel** in ui/hud.rs — shows remaining crystals in patch
11. **No ObjectInterfaceState** — command_panel.rs treats ExtractionPlate as info-only (no active commands)

**Verification steps:**
- Confirm all stats match the spec values listed above
- Run `cargo test` — all existing ExtractionPlate tests should pass
- Confirm mining_timer increments each frame and resets at interval=48
- Confirm residual mining produces 1 SC when patch.remaining_amount == 0
- If any value is wrong, fix it to match the spec
