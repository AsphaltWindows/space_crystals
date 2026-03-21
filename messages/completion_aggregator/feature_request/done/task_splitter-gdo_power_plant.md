# gdo-power-plant

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

Implement the PowerPlant structure as defined in `artifacts/designer/design/gdo_objects.md`.

GDO power generation structure. Provides power to the GDO global power grid.

**Entity Type:** Structure Type
**Faction:** GlobalDefenseOrdinance

**Stats:**
- Size: 2x2 grid units
- SymmetryType: AAAA (fully symmetrical, orientation irrelevant)
- MaxHP: 350
- PointArmor: 1
- FullArmor: 4
- SightRange: 3 grid units
- Destructible: true
- Groupable: true
- BuildRadiusExtension: 1 grid unit (extends GDO build area)
- Power: +20 (generates power)

**ObjectInterfaceState:** None (info display only — no commands, just InfoPanel showing structure stats).

**Production:** Constructed by DeploymentCenter for 150 Space Crystals, 160 frames (10 seconds). Follows ConstructionHP Rule: starts at 10% HP, gains HP linearly as construction progresses.

## QA Instructions

1. Build a PowerPlant from the DeploymentCenter — verify 150 crystals deducted and 10-second construction time.
2. Place the PowerPlant — verify it occupies 2x2 grid units.
3. Verify the GDO power display increases by +20 when PowerPlant is placed.
4. Select the PowerPlant — verify InfoPanel shows stats but no commands (info display only).
5. Verify the GDO build area extends by 1 grid unit around the PowerPlant after placement.
6. Verify the PowerPlant starts at 10% HP during construction and reaches full 350 HP when complete.
7. Attack the PowerPlant during construction — verify it can be destroyed before completion.
8. Destroy a PowerPlant — verify power decreases by 20, potentially causing slowdowns for power-consuming buildings.
9. Select multiple PowerPlants — verify they group together (Groupable=true).
10. Verify AAAA symmetry — rotation during placement should not matter (all sides identical).
