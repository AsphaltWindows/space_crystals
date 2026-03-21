# dc-buildmenu-remove-ef

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-dc-ef-construction-rework.md

## Task

Remove ExtractionFacility from the DeploymentCenter's BuildMenu. Per the design doc (gdo_objects.md), DC only constructs PowerPlant, Barracks, and SupplyTower. ExtractionFacility is a separate structure with its own construction interface — it should not appear in DC's build menu.

### Changes needed in command_panel.rs:

1. **Grid slot**: In `StructureMenuState::DcBuildMenu` match (around line 60), remove the `(0, 2) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))` entry.

2. **Label**: In `grid_button_label()`, remove the `DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n250 SC", hotkey)` case.

3. **Availability**: In `grid_button_enabled_ext()`, remove the `DcBuild(ObjectEnum::ExtractionFacility) => player_sc >= 250` case.

4. **Tests**: Update `dc_build_menu_shows_all_options()` test to remove the EF assertion. Add a new test verifying EF is NOT in the DC build menu at slot (0,2).

5. **DeploymentCenterState::construction_cost()** in structures.rs: If it handles ExtractionFacility, remove that entry (DC cannot construct EF).
