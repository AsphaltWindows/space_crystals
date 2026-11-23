# extraction_plate_persists_after_depletion

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

The Extraction Plate must persist independently after the underlying SpaceCrystalsPatch is fully depleted, switching to its ResidualMiningRate. Currently, when a patch is depleted and removed, the Extraction Plate is also removed — this contradicts the design.

Per artifacts/designer/design/gdo_objects.md ExtractionPlate:
- MiningRate: 10 Space Crystals per 48 frames (while patch has resources)
- ResidualMiningRate: 1 Space Crystal per 48 frames (after depletion, indefinitely)
- On destruction of the PLATE (not the patch): if the patch is depleted, remove the patch from the map

Per artifacts/designer/design/entities.md SpaceCrystalsPatch:
- 'When a patch is fully depleted, it disappears from the map'

The correct lifecycle is:
1. Patch has resources -> plate mines at MiningRate, patch amount decreases
2. Patch fully depleted -> patch entity may be removed from the map, BUT the plate REMAINS
3. Plate continues mining at ResidualMiningRate indefinitely (generating crystals from nothing, as a design feature)
4. If the plate is destroyed (by combat) AFTER the patch was already depleted -> the patch entity is removed if it still exists (this is the 'On destruction' rule)

The bug is that the plate entity's lifecycle is coupled to the patch entity — likely via a parent-child relationship or a despawn cascade. The plate must be decoupled so it survives patch removal.

## QA Instructions

1. Start a game as GDO. Build a Deployment Center, then an Extraction Facility.
2. Build an Extraction Plate on a SpaceCrystalsPatch.
3. Wait for the patch to be fully depleted (RemainingAmount reaches 0). Use a cheat/debug command to speed this up if available.
4. Verify: the SpaceCrystalsPatch disappears from the map (per design).
5. Verify: the Extraction Plate REMAINS on the map and is still selectable.
6. Verify: the player's Space Crystals income continues at the residual rate (1 SC per 3 seconds from this plate).
7. Verify: the plate can still be attacked and destroyed by enemy units.
8. Destroy the plate after depletion and verify no errors or unexpected behavior.
9. Test the reverse scenario: destroy the plate BEFORE the patch is depleted. The patch should become uncovered and accessible (per the 'On destruction' rules in gdo_objects.md).
