# gdo_deployment_center

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

Implement the DeploymentCenter structure stats as defined in `artifacts/designer/design/gdo_objects.md`.

**NOTE: The DC interface/construction flow was already sent as prior feature requests (designer-dc-defaultstate-cancel and designer-dc-ef-construction-rework). This request covers the core STRUCTURE STATS and construction catalog.**

GDO primary construction structure. Constructs buildings one at a time, placed within the GDO build radius.

**Entity Type:** Structure Type
**Faction:** GlobalDefenseOrdinance

**Stats:**
- Size: 4x4 grid units
- SymmetryType: AAAA
- MaxHP: 1000
- PointArmor: 1
- FullArmor: 16
- SightRange: 6 grid units
- Destructible: true
- Groupable: false (Ungroupable — each DC gets its own SelectionGroup)
- BuildRadiusExtension: 12 grid units (seeds the GDO build area)
- Power: +20 (generates power)

**DeploymentCenterInstanceState:**
- CurrentConstruction: ObjectEnum | None
- ConstructionProgress: number (frames elapsed) | None
- ReadyToPlace: ObjectEnum | None

**Construction Catalog:**
| Building | Cost | Build Time | Prerequisite |
|----------|------|-----------|-------------|
| Power Plant | 150 Space Crystals | 160 frames (10s) | None |
| Barracks | 200 Space Crystals | 160 frames (10s) | None |
| Supply Tower | 200 Space Crystals | 240 frames (15s) | Player owns >= 1 Power Plant |

**Cancellation Rules:**
- Cancel during construction: full refund
- Cancel when ready to place: 75% refund (rounded down)

## QA Instructions

1. Verify DC occupies 4x4 grid units and has AAAA symmetry.
2. Verify DC has 1000 MaxHP, 1/16 armor, 6 SightRange.
3. Verify DC is Ungroupable — selecting two DCs gives each its own SelectionGroup.
4. Verify DC generates +20 power.
5. Verify DC seeds the GDO build area with 12 grid units radius.
6. Start constructing a Power Plant — verify 150 crystals deducted, 10-second build time.
7. Start constructing a Barracks — verify 200 crystals, 10 seconds.
8. Attempt to construct Supply Tower without owning a Power Plant — verify it's unavailable.
9. Build a Power Plant first, then verify Supply Tower becomes available (200 crystals, 15 seconds).
10. Cancel during construction — verify full refund.
11. Let construction complete, then cancel the ready-to-place building — verify 75% refund rounded down.
