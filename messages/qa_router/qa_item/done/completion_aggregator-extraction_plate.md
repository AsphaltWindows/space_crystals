# extraction_plate

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# extraction-plate

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the ExtractionPlate structure as defined in `artifacts/designer/design/gdo_objects.md`.

GDO resource harvesting structure. Placed onto a SpaceCrystalsPatch by an ExtractionFacility. Mines Space Crystals from the patch at a fixed rate.

**Entity Type:** Structure Type
**Faction:** GlobalDefenseOrdinance

**Stats:**
- Size: 1x1 grid unit
- SymmetryType: AAAA
- MaxHP: 85
- PointArmor: 2
- FullArmor: 2
- SightRange: 0 (provides no vision)
- Destructible: true
- Groupable: true
- BuildRadiusExtension: 0 (does not extend build area)

**Mining Behavior:**
- MiningRate: 10 Space Crystals per 48 frames (3 seconds)
- ResidualMiningRate: 1 Space Crystal per 48 frames (3 seconds) — continues after patch depleted
- Credits are added directly to the owning player's Space Crystals balance

**InfoPanel:** Displays remaining Space Crystals in the underlying patch.

**On Destruction:**
- If underlying SpaceCrystalsPatch still has resources: the patch becomes uncovered and accessible (another plate could be placed)
- If underlying SpaceCrystalsPatch is depleted: the patch is removed from the map

**ObjectInterfaceState:** None (info display only)

**Production:** Constructed by ExtractionFacility for 75 Space Crystals, 96 frames (6 seconds).

## QA Instructions

1. Build an ExtractionPlate from an ExtractionFacility — verify 75 crystals deducted, 6-second build time.
2. Place the plate on a SpaceCrystalsPatch — verify it occupies the 1x1 patch.
3. Verify mining rate: player gains 10 Space Crystals every 3 seconds while patch has resources.
4. Select the ExtractionPlate — verify InfoPanel shows remaining crystals in the patch.
5. Wait for the patch to deplete — verify mining switches to residual rate (1 crystal per 3 seconds).
6. Destroy an ExtractionPlate on a non-depleted patch — verify the patch becomes accessible again.
7. Destroy an ExtractionPlate on a depleted patch — verify the patch disappears from the map.
8. Verify ExtractionPlate provides no vision (SightRange=0).
9. Verify BuildRadiusExtension is 0 (does not expand GDO build area).
10. Attempt to place a second ExtractionPlate on a patch that already has one — verify placement blocked.
