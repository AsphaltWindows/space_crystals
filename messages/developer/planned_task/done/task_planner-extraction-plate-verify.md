# extraction-plate-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

This is a **verification-only task** — all code should already be in place. The developer needs to confirm values match the design spec and that tests pass.

### Design Spec Reference (from `artifacts/designer/design/gdo_objects.md` lines 211-234)
- Size: 1x1, SymmetryType: AAAA, MaxHP: 85, PointArmor: 2, FullArmor: 2
- SightRange: 0, Destructible: true, Groupable: true, BuildRadiusExtension: 0
- MiningRate: 10 SC per 48 frames, ResidualMiningRate: 1 SC per 48 frames
- On destruction: if patch has resources -> uncover patch; if depleted -> despawn patch
- ObjectInterfaceState: None (info display only)

### Files to Verify (with exact locations)

1. **`artifacts/developer/src/game/types/objects.rs`** (lines 267-273, 338-341):
   - `ObjectEnum::ExtractionPlate` ObjectType: size (1,1), destructible true, sight_range 0, groupable true — **matches spec**
   - StructureType: symmetry AAAA — **matches spec**

2. **`artifacts/developer/src/game/types/structures.rs`** (lines 228-240, 419-423):
   - `ExtractionPlateState`: `attached_patch: Entity`, `mining_timer: u32` — correct fields
   - Constants: `EXTRACTION_PLATE_MINING_RATE=10`, `EXTRACTION_PLATE_RESIDUAL_RATE=1`, `EXTRACTION_PLATE_MINING_INTERVAL=48` — **matches spec**
   - `EP_MAX_HP=85.0`, `EP_POINT_ARMOR=2`, `EP_FULL_ARMOR=2`, `EP_BUILD_RADIUS=0` — **matches spec**
   - `ExtractionFacilityState::construction_cost()`: 75 SC, 96 frames — **matches spec**

3. **`artifacts/developer/src/game/world/faction.rs`** (lines 492-523):
   - `extraction_plate_mining_system`: increments `mining_timer` each frame, resets at >= 48, mines `min(MINING_RATE, remaining)` from patch, switches to `RESIDUAL_RATE` when depleted, credits GDO player — **logic matches spec**
   - System is registered in `artifacts/developer/src/game/world/mod.rs` line 102

4. **`artifacts/developer/src/game/world/faction.rs`** (lines 533-549):
   - `depleted_patch_despawn_system`: despawns patches where `remaining_amount == 0`, also despawns any attached plate
   - Registered in `artifacts/developer/src/game/world/mod.rs` line 103

5. **`artifacts/developer/src/game/utils.rs`** (lines 375-393):
   - `spawn_extraction_plate`: spawns with `ObjectInstance::destructible(ExtractionPlate, 85.0)`, `BuildRadiusExtension(0)`, `ExtractionPlateState`, `GridPosition`, `StructureInstance` — correct

6. **`artifacts/developer/src/game/combat/systems/core.rs`** (lines 556-581):
   - `remove_dead_entities_system`: queries `Option<&ExtractionPlateState>`, on death: if `patch.remaining_amount > 0` sets `has_plate=false`; else despawns patch entity — **matches spec**

7. **`artifacts/developer/src/game/world/utils.rs`** (lines 257-284):
   - `can_place_building`: special EP case requires placement on SpaceCrystalPatch without existing plate — correct

8. **`artifacts/developer/src/ui/hud.rs`** (line 227, 331):
   - Info panel shows EP abbreviation, queries `ExtractionPlateState` — correct
   
9. **`artifacts/developer/src/ui/command_panel.rs`** (line 384-389):
   - EP falls into the `_ =>` match arm setting `StructureMenuState::Inert` — info-only, no commands — **matches spec**

### Existing Tests (all in `artifacts/developer/src/game/world/faction.rs` lines 2148-2232)
- `depleted_patch_is_despawned` — depleted patch gets despawned
- `non_depleted_patch_is_not_despawned` — non-depleted patch survives
- `depleted_patch_also_despawns_attached_plate` — plate dies with depleted patch
- `plate_on_non_depleted_patch_is_not_despawned` — plate survives with live patch
- Additional test in `core.rs` line 776: `extraction_plate_state_has_attached_patch`

### Verification Procedure
1. Run `cargo test` from `artifacts/developer/` — confirm all tests pass
2. Spot-check each value listed above against the spec reference
3. If anything is wrong, fix it. All values I checked during investigation match.

## Dependencies

None — this is a standalone verification task with no dependencies on other planned tasks.
