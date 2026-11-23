# extraction_plate_depletion_lifecycle

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-extraction_plate_persists_after_depletion.md

## Task

Fix the Extraction Plate lifecycle so it persists independently after its underlying SpaceCrystalsPatch is fully depleted.

### Bug Location

The bug is in depleted_patch_despawn_system (faction.rs ~line 659). This system despawns BOTH the patch AND any attached extraction plate when remaining_amount reaches 0. Per design, only the patch should be removed; the plate must remain and continue mining at the residual rate.

### Required Changes

1. **depleted_patch_despawn_system** (faction.rs ~line 659): When a patch is depleted, do NOT despawn the attached extraction plate. Only despawn the patch entity itself. The plate should survive independently.

2. **extraction_plate_mining_system** (faction.rs ~line 617): Currently does patches.get_mut(plate_state.attached_patch) which will fail (return Err) once the patch entity is despawned. The else branch of this get_mut should mine at EXTRACTION_PLATE_RESIDUAL_RATE. This may already work since the Ok branch handles remaining_amount==0 with residual rate, but after the patch entity is despawned, the get_mut will return Err. Add an else branch for the case where the patch entity no longer exists -- mine at EXTRACTION_PLATE_RESIDUAL_RATE.

3. **remove_dead_entities_system** (combat/systems/core.rs ~line 776): When an extraction plate is destroyed, it tries patches.get_mut(plate.attached_patch) to clean up the patch. After depletion, the patch entity may already be despawned. Ensure this handles the case where the patch entity no longer exists (the get_mut returns Err -- no cleanup needed since the patch is already gone).

### Constants

- EXTRACTION_PLATE_MINING_RATE: 10 (normal rate)
- EXTRACTION_PLATE_RESIDUAL_RATE: already defined (should be 1, which is 10% of mining rate)
- EXTRACTION_PLATE_MINING_INTERVAL: 48.0 frames

### Tests to Add/Update

- Test that depleted_patch_despawn_system does NOT despawn plates attached to depleted patches
- Test that extraction_plate_mining_system continues generating residual income when the attached patch entity no longer exists
- Test that remove_dead_entities_system handles plate destruction gracefully when attached patch is already despawned

## Technical Context

### File 1: artifacts/developer/src/game/world/faction.rs

**depleted_patch_despawn_system** (line 659-678):
- Iterates all SpaceCrystalPatch entities; when remaining_amount == 0, it finds any ExtractionPlateState entity whose attached_patch matches, despawns the plate (line 670), then despawns the patch (line 675).
- **Fix**: Remove lines 667-672 entirely (the for loop that finds and despawns attached plates). Only despawn the patch entity. The plate should remain.
- System is registered in FixedUpdate/DiagCategory::Construction (world/mod.rs:114).
- No explicit ordering with extraction_plate_mining_system within that set -- both run in the same FixedUpdate set. After the fix, on the tick the patch is despawned, the mining system may still read the patch entity (Bevy Commands are deferred). On subsequent ticks, patches.get_mut() will return Err since the patch is gone.

**extraction_plate_mining_system** (line 617-650):
- Line 630: `if let Ok(mut patch) = patches.get_mut(plate_state.attached_patch)` -- this is the main branch. When the patch exists AND has remaining_amount > 0, it mines at EXTRACTION_PLATE_MINING_RATE (line 632). When remaining_amount == 0, it mines at EXTRACTION_PLATE_RESIDUAL_RATE (line 637).
- **Current gap**: When the patch entity no longer exists (despawned by depleted_patch_despawn_system), the get_mut returns Err and the entire mining tick is SKIPPED (no else branch). The plate produces nothing.
- **Fix**: Add an else branch after line 647 that mines at EXTRACTION_PLATE_RESIDUAL_RATE and adds it to the player's resources. The pattern is identical to the depleted-patch branch (lines 636-637 + 641-645) but outside the Ok arm.
- Query signature: `plates: Query<(&Owner, &mut ExtractionPlateState)>`, `patches: Query<&mut SpaceCrystalPatch>`, `players: Query<(&Player, &mut GdoPlayerResources)>`.
- Uses `get_power_ratio_for_owner()` helper (faction.rs:1153) to apply power deficit slowdown to mining_timer accumulation.

### File 2: artifacts/developer/src/game/combat/systems/core.rs

**remove_dead_entities_system** (line 776-823):
- Line 787-801: When a dead entity has ExtractionPlateState, it tries `patches.get_mut(plate.attached_patch)`.
- If patch exists and has resources (remaining_amount > 0): sets patch.has_plate = false (uncovering it).
- If patch exists and is depleted: despawns the patch entity (line 798).
- **If patch does NOT exist (get_mut returns Err)**: currently falls through silently (the if-let-Ok just doesn't match, no else needed). This is ALREADY correct -- no change required here. The patch was already despawned by depleted_patch_despawn_system, so there's nothing to clean up.

### File 3: artifacts/developer/src/game/types/structures.rs

- ExtractionPlateState struct (line 230-235): has attached_patch: Entity and mining_timer: f32. No changes needed to the struct.
- Constants (lines 238-240): EXTRACTION_PLATE_MINING_RATE = 10, EXTRACTION_PLATE_RESIDUAL_RATE = 1, EXTRACTION_PLATE_MINING_INTERVAL = 48.0. All already defined.

### Existing Tests to Update (faction.rs)

- **depleted_patch_also_despawns_attached_plate** (line 2774): This test ASSERTS that the plate IS despawned (line 2794-2795). This assertion is WRONG per the new behavior. Change the assertion to verify the plate entity is still alive and the patch entity is despawned.
- **depleted_patch_is_despawned** (line 2740): Keep unchanged -- patch should still be despawned.
- **non_depleted_patch_is_not_despawned** (line 2756): Keep unchanged.
- **plate_on_non_depleted_patch_is_not_despawned** (line 2798): Keep unchanged.

### New Tests to Add

1. **extraction_plate_survives_patch_depletion**: Spawn patch with remaining_amount=0 and an attached plate. Run depleted_patch_despawn_system. Assert patch is despawned, plate is alive.

2. **extraction_plate_mines_residual_after_patch_despawned**: Spawn player + plate (with attached_patch pointing to a valid entity), then despawn the patch entity manually. Run extraction_plate_mining_system for 48 ticks. Assert player resources increased by EXTRACTION_PLATE_RESIDUAL_RATE.

3. **plate_destruction_after_patch_despawned**: Spawn plate with ExtractionPlateState pointing to a non-existent entity + ObjectInstance with hp=0. Run remove_dead_entities_system. Assert no panic and plate is despawned cleanly.

### Test Pattern Reference

Existing tests use `World::new()` + `world.run_system_once(system_fn).unwrap()` + `world.flush()` for deferred commands. See lines 2740-2821 for depleted_patch tests and lines 2823-2929 for mining tests. The mining tests spawn Player + GdoPlayerResources + Owner + ExtractionPlateState + SpaceCrystalPatch entities.

### System Ordering Consideration

Both depleted_patch_despawn_system and extraction_plate_mining_system are registered in FixedUpdate/DiagCategory::Construction (world/mod.rs:113-114) with no relative ordering. After the fix, if depleted_patch_despawn_system runs first in a tick, it despawns the patch entity (deferred), but the mining system still sees the patch that tick because despawn is deferred. On the NEXT tick, the mining system's get_mut will return Err. This is safe -- the only impact is that on the depletion tick, the mining system mines at EXTRACTION_PLATE_RESIDUAL_RATE (from the remaining_amount==0 branch), and on subsequent ticks it mines from the new else branch. Both produce the same amount (EXTRACTION_PLATE_RESIDUAL_RATE), so there's no gameplay discrepancy. No ordering constraint is needed.

## Dependencies

None -- this is a standalone bug fix. The three systems involved (depleted_patch_despawn_system, extraction_plate_mining_system, remove_dead_entities_system) all already exist and are registered.
