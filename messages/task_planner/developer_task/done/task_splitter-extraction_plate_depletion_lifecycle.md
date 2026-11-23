# extraction_plate_depletion_lifecycle

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-extraction_plate_persists_after_depletion.md

## Task

Fix the Extraction Plate lifecycle so it persists independently after its underlying SpaceCrystalsPatch is fully depleted.

### Bug Location

The bug is in `depleted_patch_despawn_system` (faction.rs ~line 659). This system despawns BOTH the patch AND any attached extraction plate when remaining_amount reaches 0. Per design, only the patch should be removed; the plate must remain and continue mining at the residual rate.

### Required Changes

1. **depleted_patch_despawn_system** (faction.rs ~line 659): When a patch is depleted, do NOT despawn the attached extraction plate. Only despawn the patch entity itself. The plate should survive independently.

2. **extraction_plate_mining_system** (faction.rs ~line 617): Currently does `patches.get_mut(plate_state.attached_patch)` which will fail (return Err) once the patch entity is despawned. The else branch of this get_mut should mine at EXTRACTION_PLATE_RESIDUAL_RATE. This may already work since the Ok branch handles remaining_amount==0 with residual rate, but after the patch entity is despawned, the get_mut will return Err. Add an else branch for the case where the patch entity no longer exists -- mine at EXTRACTION_PLATE_RESIDUAL_RATE.

3. **remove_dead_entities_system** (combat/systems/core.rs ~line 776): When an extraction plate is destroyed, it tries `patches.get_mut(plate.attached_patch)` to clean up the patch. After depletion, the patch entity may already be despawned. Ensure this handles the case where the patch entity no longer exists (the get_mut returns Err -- no cleanup needed since the patch is already gone).

### Constants

- EXTRACTION_PLATE_MINING_RATE: 10 (normal rate)
- EXTRACTION_PLATE_RESIDUAL_RATE: already defined (should be 1, which is 10% of mining rate)
- EXTRACTION_PLATE_MINING_INTERVAL: 48.0 frames

### Tests to Add/Update

- Test that depleted_patch_despawn_system does NOT despawn plates attached to depleted patches
- Test that extraction_plate_mining_system continues generating residual income when the attached patch entity no longer exists
- Test that remove_dead_entities_system handles plate destruction gracefully when attached patch is already despawned
