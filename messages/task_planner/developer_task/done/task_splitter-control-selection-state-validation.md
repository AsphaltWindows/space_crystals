# control-selection-state-validation

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-control-state-selection.md

## Task

Implement ObjectInterfaceState reset and validation tied to selection changes.

### 1. Reset ObjectInterfaceState on Selection/ActiveGroup Change

Per the design doc: "Reset to the default state when the Selection or ActiveGroup changes."

Add a system (e.g., `interface_state_selection_reset_system`) that:
- Tracks the previous Selection state (active_group_index + group types) using a Local or dedicated resource
- Each tick, compares current Selection to previous
- If Selection groups changed OR active_group_index changed: reset `ObjectInterfaceState` to `ObjectInterfaceState::Default`
- Update the tracking state
- Register in the appropriate system set, after `selection_group_sync_system` and `active_group_cycle_system`

### 2. ObjectInterfaceState Tick Validation

Per the design doc: "Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's game state — if the current state is no longer valid, it resets to the default state."

Add a system (e.g., `interface_state_validation_system`) that:
- Reads current `ObjectInterfaceState` and `Selection`
- Checks if the current state is still valid for the active group. Examples:
  - `StructureMenu(DcIdle)` is invalid if no DeploymentCenter is in the active group
  - `StructureMenu(BarracksMenu)` is invalid if no Barracks is in the active group  
  - `StructureMenu(DcConstructing)` is invalid if the DC no longer has an active construction
  - `AgentMenu(*)` is invalid if no SyndicateAgent is in the active group
  - `AwaitingTarget(*)` is invalid if the active group has no entities
- On invalid state: reset to `ObjectInterfaceState::Default`
- Register after the selection reset system

Both systems go in `game/world/resources.rs` or `ui/` depending on existing conventions. The existing `selection_validation_system` in `game/world/resources.rs` is a good reference for the pattern.
