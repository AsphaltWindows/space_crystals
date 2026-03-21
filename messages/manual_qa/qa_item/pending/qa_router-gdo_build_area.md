# gdo_build_area

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# gdo-build-area

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the GDO Build Area system as defined in `artifacts/designer/design/gdo_objects.md` under GDOBuildArea.

**GDOBuildArea:**
A single shared buildable area per GDO player. Rules:
- Seeded by each DeploymentCenter's BuildRadiusExtension (12 grid units)
- Grown by every placed GDO building's BuildRadiusExtension
- When placing a building, at least 1 grid cell of the new building must be within the current build area
- After placement, the build area expands outward from the placed building by that building's BuildRadiusExtension in all directions

**BuildRadiusExtension values from existing GDO buildings:**
- DeploymentCenter: 12 grid units
- PowerPlant: 1 grid unit
- Barracks: 2 grid units
- ExtractionFacility: 2 grid units
- SupplyTower: 1 grid unit
- ExtractionPlate: 0 (does not extend build area)

**Visual feedback during placement:**
- The player's current GDO build area is highlighted as a semi-transparent overlay on the ground during AwaitingPlacement
- Building ghost is green when valid (within build area, all tiles Buildable, no overlap), red when invalid

**Placement validation:**
- All tiles under footprint must be in Visible state for the placing player
- All tiles must be Buildable
- No existing structure overlap
- At least 1 cell within current build area

## QA Instructions

1. Start as GDO — verify build area extends 12 grid units from the DeploymentCenter.
2. Enter AwaitingPlacement — verify the build area is shown as a semi-transparent ground overlay.
3. Place a PowerPlant at the edge of the build area — verify the area expands by 1 grid unit around the PowerPlant.
4. Place a Barracks — verify area expands by 2 grid units around it.
5. Attempt to place a building entirely outside the build area — verify red ghost and placement blocked.
6. Attempt to place where at least 1 cell is inside the build area — verify green ghost and placement allowed.
7. Attempt to place on non-Buildable tiles (e.g., Rugged Terrain, Water) — verify placement blocked.
8. Attempt to place overlapping an existing structure — verify placement blocked.
9. Attempt to place on unexplored or explored (but not Visible) tiles — verify placement blocked.
10. Place an ExtractionPlate (BuildRadiusExtension=0) — verify build area does NOT expand.
