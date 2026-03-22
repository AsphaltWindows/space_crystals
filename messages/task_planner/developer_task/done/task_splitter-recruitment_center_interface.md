# recruitment_center_interface

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-cults_objects_formalized.md

## Task

Implement the ObjectInterfaceState for RecruitmentCenter. This covers the command panel and right-click behavior when a Recruitment Center is selected.

### StructureMenuState variant

Add `StructureMenuState::RecruitmentCenterMenu` to the enum in `ui/types.rs`.

### Command panel grid (command_panel.rs)

When `StructureMenuState::RecruitmentCenterMenu` is active, display:
- **X (row=2, col=1): Cancel Production** — cancels the current Recruit in production, resetting `ProductionProgress` to None. Only available (button visible) when `RecruitmentCenterState.production_progress > 0` (i.e., production is active). Action: set `production_progress = 0` on the selected RecruitmentCenter entity.
- **C (row=2, col=2): Set Rally Point** — transitions to `ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint)`. Left-click on ground or object sets the rally point on the RecruitmentCenterState, then returns to `StructureMenuState::RecruitmentCenterMenu`.

### State detection (update_command_panel_state)

When the active selection is a single RecruitmentCenter (ObjectEnum::RecruitmentCenter), set `ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu)`. Follow the same pattern as HeadquartersMenu, BarracksMenu, etc.

### Right-click resolution (core.rs - right_click_move_command)

When the selected unit is a RecruitmentCenter and interface state is `RecruitmentCenterMenu`:
- Right-click Ground: set `RecruitmentCenterState.rally_point = Some(world_position)`
- Right-click Object: set `RecruitmentCenterState.rally_point = Some(object_position)`

Follow the same pattern used by HeadquartersMenu right-click rally (production_rally_point_system).

### AwaitingTarget resolution

When in `AwaitingTarget(SetRallyPoint)` and the selected entity is a RecruitmentCenter:
- Left-click ground: set rally point to clicked location, return to RecruitmentCenterMenu
- Left-click object: set rally point to object location, return to RecruitmentCenterMenu

Follow the existing `set_rally_point_click_system` pattern used by HQ/Barracks/SupplyTower.
