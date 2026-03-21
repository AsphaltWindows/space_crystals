# gdo-extraction-facility

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the ExtractionFacility structure stats as defined in `artifacts/designer/design/gdo_objects.md`.

**NOTE: The EF interface/construction flow was already sent as a prior feature request (designer-dc-ef-construction-rework). This request covers the core STRUCTURE STATS.**

GDO resource extraction structure. Constructs Extraction Plates onto Space Crystal Patches within the GDO player's build area.

**Entity Type:** Structure Type
**Faction:** GlobalDefenseOrdinance

**Stats:**
- Size: 3x3 grid units
- SymmetryType: AAAA
- MaxHP: 500
- PointArmor: 1
- FullArmor: 9
- SightRange: 3 grid units
- Destructible: true
- Groupable: false (Ungroupable — each EF gets its own SelectionGroup)
- BuildRadiusExtension: 2 grid units
- Power: -15 (consumes power)

**ExtractionFacilityInstanceState:**
- CurrentConstruction: ExtractionPlate | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ExtractionPlate | None

**Constructs:**
- Extraction Plate: 75 Space Crystals, 96 frames (6 seconds)

**Cancellation:**
- During construction: full refund
- When ready to place: 75% refund (rounded down)

## QA Instructions

1. Verify EF occupies 3x3 grid units with AAAA symmetry.
2. Verify EF has 500 MaxHP, 1/9 armor, 3 SightRange.
3. Verify EF is Ungroupable.
4. Verify EF consumes 15 power (Power: -15).
5. Verify EF extends GDO build area by 2 grid units.
6. Build an Extraction Plate — verify 75 crystals deducted, 6-second build time.
7. Cancel during construction — verify full refund (75 crystals).
8. Let construction complete, cancel ready plate — verify 75% refund (56 crystals).
9. Place plate on valid SpaceCrystalsPatch — verify placement succeeds.
10. Attempt to place on non-SpaceCrystalsPatch tile — verify placement blocked.
