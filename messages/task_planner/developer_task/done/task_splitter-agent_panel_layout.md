# agent_panel_layout

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent-interface.md

## Task

Update the Agent DefaultState command panel layout to match the design spec: show only 2 buttons instead of the current 7.

### Current State
In `command_panel.rs`, the `AgentMenuState::AgentDefault` grid (around line 153) maps 7 buttons:
- (0,0) Move, (0,1) Attack, (0,2) Enter
- (1,0) Gather, (1,1) DropOff, (1,2) BuildTunnel
- (2,0) Stop

### Required Change
The design spec defines only 2 panel commands for the Agent DefaultState:
- Slot (0,0) = `CommandButtonAction::AgentBuildTunnel` (hotkey Q: Build Tunnel)
- Slot (0,1) = `CommandButtonAction::AgentDropOff` (hotkey W: Drop Off Resources)

All other commands (Move, Stop, Attack, Enter, Gather) are right-click-only and should NOT appear in the panel.

### Additional Fix: AwaitingTarget Escape
In the Escape handler (around line 902), when cancelling `ObjectInterfaceState::AwaitingTarget(_)`, the state returns to `ObjectInterfaceState::Default`. When selected units are Agents, it should return to `ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)` instead. Check how agent detection is done in the `update_command_panel_state` system (around line 408) and replicate the logic.

### Files to Modify
- `src/ui/command_panel.rs`: Update `AgentDefault` match arm in `get_grid_slot_action`, fix AwaitingTarget escape handler, update related tests.

### Tests to Update
The existing agent grid tests (starting around line 3041) test the 7-button layout. Update them to verify only (0,0)=AgentBuildTunnel and (0,1)=AgentDropOff are present, with all other slots returning None.
