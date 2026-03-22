# dc_buildmenu_add_ef

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-dc_builds_extraction_facility.md

## Task

Add Extraction Facility to the Deployment Center's BuildMenu grid and construction cost table.

### Changes needed:

1. **command_panel.rs** - `StructureMenuState::DcBuildMenu` grid (around line 59): Add slot `(1, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))` — this places EF next to Supply Tower in the second row.

2. **structures.rs** - `DeploymentCenterState::construction_cost()` (around line 86): Add match arm `ObjectEnum::ExtractionFacility => Some(StructureCost { space_crystals: 200, build_frames: 320 })`.

3. **Update/add tests**: The existing test at structures.rs line 1181 asserts `construction_cost(&ObjectEnum::ExtractionFacility).is_none()` — this must be updated to assert `Some` with the correct values (200 SC, 320 frames). Add a test for the DcBuildMenu grid slot returning the correct action.

### Context:
- The DC build menu already has PowerPlant at (0,0), Barracks at (0,1), SupplyTower at (1,0). EF has no prerequisite (unlike SupplyTower which requires a PowerPlant).
- The construction/placement flow for DC-built structures is fully generic — DcBuild action triggers crystal deduction and sets current_construction, then the ef_construction_tick_system handles progress, and the placement flow is shared. No new systems needed.
- ExtractionFacility is already a valid ObjectEnum variant with spawn function, EF state, and construction tick system.
