# gdo-extraction-facility-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

This is a **verification task** — the implementation already exists. The developer should confirm each item against the design spec and fix any discrepancies.

### Files to check (all paths relative to `artifacts/developer/`):

1. **`src/game/types/objects.rs`** (line ~260): `ObjectEnum::ExtractionFacility` ObjectType definition
   - VERIFIED: size (3,3), destructible=true, sight_range=3, groupable=false ✓
   - StructureType symmetry AAAA (line ~334) ✓

2. **`src/game/types/structures.rs`** (lines 408-413): Constants block
   - VERIFIED: EF_MAX_HP=500.0, EF_POINT_ARMOR=1, EF_FULL_ARMOR=9, EF_BUILD_RADIUS=2, EF_POWER=-15 ✓

3. **`src/game/types/structures.rs`** (lines 186-195): `ExtractionFacilityState` struct
   - VERIFIED: fields `current_construction: bool`, `construction_progress: Option<f32>`, `ready_to_place: bool` ✓
   - `construction_cost()` (line ~199): returns StructureCost { space_crystals: 75, build_frames: 96 } ✓
   - `cancellation_refund()` (line ~208): full refund during construction, 75% during ready_to_place ✓

4. **`src/game/utils.rs`** (lines 300-352): `spawn_extraction_facility()` function
   - VERIFIED spawns with: ObjectInstance::destructible(ExtractionFacility, EF_MAX_HP), PowerValue(EF_POWER), BuildRadiusExtension(EF_BUILD_RADIUS), ExtractionFacilityState::default(), SightRange(sight_range from ObjectType = 3) ✓

5. **`src/game/world/faction.rs`** (line ~789): `ef_construction_tick_system` registered
   - VERIFIED: system exists and is registered in `src/game/world/mod.rs` (line 104) ✓

6. **`src/game/world/faction.rs`** (lines 1403-1408): EF in GDO game start building placement ✓

### Verification approach:
- Read each file location listed above and confirm values match the design spec at `artifacts/designer/design/gdo_objects.md` (lines 166-209)
- All values have been pre-verified by the planner — if everything matches, run `cargo test` to confirm tests pass
- If any discrepancy is found, fix it in the source file and re-run tests

### Existing test coverage:
- `src/game/types/structures.rs` has tests for EF_POWER values (line ~789, ~811)
- `src/game/types/objects.rs` has tests confirming EF is a structure, is in validation lists (lines ~693, ~748, ~769)
- No dedicated EF construction tick system tests found in faction.rs test module — note this but don't block on it

## Dependencies

None — this is a standalone verification task with no code dependencies on other planned_tasks.
