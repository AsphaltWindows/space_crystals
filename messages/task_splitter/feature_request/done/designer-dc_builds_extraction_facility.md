# dc_builds_extraction_facility

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

The Deployment Center's Constructs list in `artifacts/designer/design/gdo_objects.md` was missing the Extraction Facility. Two changes were made:

1. **Extraction Facility added to DC Constructs** (line ~121): Added `Extraction Facility: 200 Space Crystals, 320 frames (20 seconds)` with no prerequisite. Also added the corresponding BuildMenu entry in the DC's ObjectInterfaceState idle state.

2. **Extraction Plate power cost** (ExtractionPlate section): Added `Power - -3` to the Extraction Plate. Each plate now draws 3 power from the GDO global power grid.

## QA Instructions

1. **Building an Extraction Facility from the Deployment Center**:
   - Select the Deployment Center and open the Build Menu (Q).
   - Verify the Extraction Facility appears as a build option alongside Power Plant, Barracks, and Supply Tower.
   - Verify it is available immediately with no prerequisite (unlike Supply Tower which requires a Power Plant).
   - Build an Extraction Facility. Verify 200 Space Crystals are deducted.
   - Wait 20 seconds (320 frames at 16 FPS) for construction to complete.
   - Place the Extraction Facility within the build radius. Verify it appears and functions (can build Extraction Plates).

2. **Extraction Plate power drain**:
   - From a placed Extraction Facility, build and place an Extraction Plate onto a Space Crystal Patch.
   - Verify the player's power total decreases by 3 for each placed Extraction Plate.
   - Place multiple plates and confirm the power drain stacks (e.g., 3 plates = -9 power).
   - Destroy an Extraction Plate and verify the power is restored.
