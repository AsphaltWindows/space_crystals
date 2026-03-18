# Ticket: GDO Extraction Facility and Extraction Plate

## Current State
No GDO resource extraction structures exist. There is no mechanism to harvest Space Crystals from patches.

## Desired State
Implement the Extraction Facility and Extraction Plate structures:

**Extraction Facility** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 3x3, Symmetry: AAAA, MaxHP: 500, PointArmor: 1, FullArmor: 9, SightRange: 3
- BuildRadiusExtension: 2, Power: -15, Ungroupable
- ExtractionFacilityInstanceState: CurrentConstruction (ExtractionPlate|None), ConstructionProgress (frames|None), ReadyToPlace (ExtractionPlate|None)
- Constructs: Extraction Plate (75 SC, 96 frames / 6 seconds)
- Same construct-then-place flow as Deployment Center (no sub-menu, only one product)
- Cancellation: full refund during construction, 75% (rounded down) when ready to place
- Plate placement rule: must be on a Space Crystal Patch within GDO build area with no existing plate

**Extraction Facility Interface**:
- DefaultState (idle): Build Extraction Plate (CommandIssuingTransition, deducts 75 SC). Available only if player has sufficient SC.
- DefaultState (constructing): Cancel Construction (full refund, CommandIssuingTransition)
- DefaultState (ready): Place Plate -> AwaitingPlacement (StateOnlyTransition). Cancel Ready Plate (75% refund, CommandIssuingTransition).
- AwaitingPlacement: ghost preview snapped to grid. Green tint over valid Space Crystal Patch (within build area, no existing plate), red otherwise. Build area overlay visible. R/Shift+R rotation, F/Shift+F flipping (horizontal/vertical). Side labels (A/B/C/D per SymmetryType) displayed on ghost, updating with rotation and flipping. Left-click valid patch -> places plate (CommandIssuingTransition -> DefaultState). Escape/right-click -> DefaultState (plate remains ready).
  - Note: Extraction Plates are 1x1 AAAA, so rotation and flipping are no-ops. Controls should still be present for consistency with the shared construct-then-place flow.

**Extraction Plate** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 1x1, Symmetry: AAAA, MaxHP: 85, PointArmor: 2, FullArmor: 2, SightRange: 0
- BuildRadiusExtension: 0 (does NOT extend build area)
- MiningRate: 10 SC per 48 frames (3 seconds)
- ResidualMiningRate: 1 SC per 48 frames (when patch depleted)
- ObjectInterfaceState: None (info display only, shows remaining SC in underlying patch)
- On destruction:
  - If underlying patch still has resources: patch becomes uncovered and accessible
  - If underlying patch is depleted: patch is removed from the map

## Justification
Defined in `features/gdo_objects.md` (ExtractionFacility, ExtractionPlate sections). This is the primary GDO economy mechanic — Space Crystal harvesting is the main resource income for the faction. The construct-then-place flow mirrors the Deployment Center pattern.

## QA Steps
1. Select an Extraction Facility (idle). Verify "Build Extraction Plate" is available.
2. Click Build Extraction Plate with sufficient SC (75). Verify SC is deducted and construction begins.
3. While constructing, verify Cancel Construction is available. Cancel and verify full 75 SC refund.
4. Construct to completion. Verify "Place Plate" and "Cancel Ready Plate" are available.
5. Cancel a ready plate. Verify 75% refund: floor(75 * 0.75) = 56 SC refunded.
6. Construct another plate to ready. Enter AwaitingPlacement.
7. Hover ghost over a Space Crystal Patch within the build area with no existing plate. Verify green tint.
8. Hover ghost over a Space Crystal Patch outside the build area. Verify red tint.
9. Hover ghost over a patch that already has a plate. Verify red tint.
10. Hover ghost over non-patch terrain. Verify red tint.
11. Left-click a valid patch. Verify Extraction Plate is placed and returns to DefaultState.
12. Verify the Extraction Plate does NOT extend the build area (BuildRadiusExtension: 0).
13. Wait 48 frames (3 seconds). Verify 10 SC mined from the patch and added to the player's SC.
14. Select the Extraction Plate. Verify info display shows remaining SC in the underlying patch.
15. Deplete the patch completely. Verify mining continues at ResidualMiningRate (1 SC per 48 frames).
16. Destroy an Extraction Plate over a non-depleted patch. Verify the patch becomes uncovered and accessible.
17. Destroy an Extraction Plate over a depleted patch. Verify the patch is removed from the map.
18. Press Escape from AwaitingPlacement. Verify return to DefaultState with plate still ready.
19. Verify Extraction Facility is Ungroupable (always gets its own SelectionGroup).

## Expected Experience
The Extraction Facility should feel like a dedicated resource structure. The construct-then-place flow should mirror the Deployment Center's AwaitingPlacement mode, but the ghost should only light green over valid Space Crystal Patches. Mining should be a steady, visible income stream. The info panel on an Extraction Plate should clearly show the remaining resources in the patch. Destroying plates over depleted patches should visually remove the patch from the map.
