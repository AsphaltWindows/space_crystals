# Ticket: GDO Build Area and Deployment Center

## Current State
No build area system or construction placement flow exists. There is no Deployment Center structure.

## Desired State
Implement the GDOBuildArea mechanic and the Deployment Center structure:

**GDOBuildArea**:
- Single shared buildable area per GDO player
- Seeded by each Deployment Center's BuildRadiusExtension (12 grid units)
- Grown by every placed GDO building's BuildRadiusExtension
- Placement rule: at least 1 grid cell of the new building must be within the current build area
- After placement: area expands outward from the placed building by its BuildRadiusExtension in all directions

**Deployment Center** (Structure):
- Faction: GlobalDefenseOrdinance
- Size: 4x4, Symmetry: AAAA, MaxHP: 1000, PointArmor: 1, FullArmor: 16, SightRange: 6
- BuildRadiusExtension: 12, Power: +20, Ungroupable
- DeploymentCenterInstanceState: CurrentConstruction (ObjectEnum|None), ConstructionProgress (frames|None), ReadyToPlace (ObjectEnum|None)
- Constructs (one at a time):
  - Power Plant: 150 SC, 160 frames (10s)
  - Barracks: 200 SC, 160 frames (10s)
  - Supply Tower: 200 SC, 240 frames (15s). Requires: player owns >= 1 Power Plant
- Cancellation: full refund during construction, 75% (rounded down) when ready to place

**Deployment Center Interface**:
- DefaultState: "Build" button enters BuildMenu (StateOnlyTransition)
- BuildMenu (idle): options for Power Plant, Barracks, Supply Tower (conditional). Each deducts cost and starts construction (CommandIssuingTransition -> DefaultState). Escape/right-click -> DefaultState.
- BuildMenu (constructing): Cancel Construction (full refund, CommandIssuingTransition -> DefaultState). Other options unavailable.
- BuildMenu (ready): Select completed building -> AwaitingPlacement (StateOnlyTransition). Cancel Ready Building (75% refund, CommandIssuingTransition -> DefaultState).
- AwaitingPlacement: ghost preview snapped to grid, green/red validity tint, build area overlay. R rotates 90 CW, Shift+R rotates 90 CCW. F flips ghost horizontally, Shift+F flips ghost vertically. Non-square buildings swap footprint on 90/270. Side labels (A/B/C/D per SymmetryType) displayed on ghost, updating with rotation and flipping. Left-click valid location within build radius -> places building (CommandIssuingTransition -> DefaultState). Escape/right-click -> BuildMenu (building remains ready).

## Justification
Defined in `features/gdo_objects.md` (GDOBuildArea and DeploymentCenter sections). The Deployment Center is the foundational GDO structure that enables all other building placement. The build area mechanic is the spatial constraint system for GDO base construction.

## QA Steps
1. Place a Deployment Center. Verify GDOBuildArea is seeded with a 12 grid unit radius around it.
2. Open the BuildMenu while idle. Verify Power Plant and Barracks are available. Verify Supply Tower is unavailable (no Power Plant owned yet).
3. Start constructing a Power Plant (150 SC). Verify SC is deducted and CurrentConstruction is set.
4. While constructing, open BuildMenu. Verify only Cancel Construction is available. Cancel. Verify full 150 SC refunded and construction clears.
5. Construct a Power Plant to completion. Verify ReadyToPlace is set and BuildMenu shows the ready building option.
6. Cancel a ready-to-place building. Verify 75% refund (rounded down): 150 * 0.75 = 112 SC refunded.
7. Construct another Power Plant to ready. Enter AwaitingPlacement. Verify ghost preview follows cursor snapped to grid.
8. Move ghost inside build area. Verify green tint. Move ghost outside build area. Verify red tint.
9. Press R. Verify ghost rotates 90 CW. Press Shift+R. Verify ghost rotates 90 CCW.
10. Press F. Verify ghost flips horizontally. Press Shift+F. Verify ghost flips vertically.
11. Verify side labels (A/B/C/D) on the ghost update correctly after each rotation and flip operation.
12. Left-click a valid location inside build area. Verify building is placed with the current rotation and flip state, ReadyToPlace clears, returns to DefaultState.
13. Verify the build area expanded outward from the placed Power Plant by its BuildRadiusExtension (1 grid unit).
14. Now open BuildMenu again. Verify Supply Tower is available (player owns a Power Plant).
15. Press Escape from AwaitingPlacement. Verify return to BuildMenu with building still ready to place.
16. Construct a Barracks (3x2, ABAC symmetry). Enter AwaitingPlacement. Rotate to 90 degrees. Verify footprint swaps to 2x3 and side labels rotate correctly. Press F to flip horizontally. Verify side labels update to reflect the flipped orientation.
17. Verify at least 1 grid cell of the building ghost must be within build area for valid placement (green tint).

## Expected Experience
The build menu should clearly show which buildings are available, with unavailable options grayed out or hidden. Construction progress should be visible. The AwaitingPlacement mode should feel like a standard RTS building placement: ghost snaps to grid, green/red feedback is immediate, build area overlay is clearly visible. Rotation should feel responsive. Placing a building should instantly expand the visible build area.
