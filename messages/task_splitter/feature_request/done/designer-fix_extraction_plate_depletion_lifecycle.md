# fix_extraction_plate_depletion_lifecycle

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fix: Extraction Plate must persist after SpaceCrystalsPatch depletion.

Per `artifacts/designer/design/gdo_objects.md`, the ExtractionPlate design specifies:
- "When the patch is depleted, continues mining at a residual rate indefinitely" (ResidualMiningRate: 10% of MiningRate)
- "On destruction [of the plate]: If the underlying Space Crystal Patch is depleted: the patch is removed from the map"

The current implementation incorrectly removes the ExtractionPlate when the SpaceCrystalsPatch is fully depleted. The intended lifecycle is:
1. Patch depletes to 0 remaining resources
2. Patch entity remains on the map (covered by the plate)
3. ExtractionPlate continues operating at its ResidualMiningRate indefinitely
4. Only when the ExtractionPlate is destroyed (by combat damage) should the depleted patch then be removed from the map

The patch-to-plate relationship must be decoupled so that patch depletion does not cascade to plate removal. The plate entity must survive independently after its underlying patch reaches 0 resources.

No design document changes needed -- this is purely an implementation bug.

## QA Instructions

1. Start a game as GDO. Build a Deployment Center, then an Extraction Facility within build range.
2. Construct an Extraction Plate onto a SpaceCrystalsPatch.
3. Wait for the patch to fully deplete (RemainingAmount reaches 0).
4. Verify that the ExtractionPlate remains on the map after depletion.
5. Verify that the plate continues generating Space Crystals at the residual rate (10% of normal MiningRate).
6. Select the ExtractionPlate and confirm it displays correctly in the InfoPanel.
7. Destroy the ExtractionPlate (via enemy unit attack). Verify that both the plate and the depleted patch are then removed from the map.
