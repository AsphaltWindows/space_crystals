# gdo-extraction-facility-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-gdo-extraction-facility.md

## Task

Verify that the ExtractionFacility structure implementation matches the design spec. All stats and systems appear to already be implemented. Confirm the following exist and are correct:

1. **ObjectEnum::ExtractionFacility** in objects.rs — size (3,3), destructible=true, sight_range=3, groupable=false
2. **StructureType** — symmetry AAAA
3. **Constants** in structures.rs — EF_MAX_HP=500, EF_POINT_ARMOR=1, EF_FULL_ARMOR=9, EF_BUILD_RADIUS=2, EF_POWER=-15
4. **spawn_extraction_facility()** in utils.rs — spawns with ObjectInstance(EF_MAX_HP), PowerValue(EF_POWER), BuildRadiusExtension(EF_BUILD_RADIUS), ExtractionFacilityState, SightRange(3)
5. **ExtractionFacilityState** — has current_construction, construction_progress, ready_to_place fields
6. **ef_construction_tick_system** — ticks construction progress
7. **EP construction**: 75 SC cost, 96 frames build time
8. **Cancellation**: full refund during construction, 75% during ready-to-place

If any stat is wrong or any component is missing, fix it. Run `cargo test` to confirm all tests pass.
