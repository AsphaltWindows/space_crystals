# agent_dropoff_target_click

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent-interface.md

## Task

Implement left-click entity handling in AwaitingTarget(DropOff) mode so that clicking an own Tunnel issues a DropOffResources command to selected Agents.

### Current State
In `core.rs`, the `right_click_move_command` system handles the Agent's DropOff flow partially:
- The AgentDropOff button correctly enters `AwaitingTarget(CommandType::DropOff)` (in command_panel.rs ~line 1419)
- Right-click on own Tunnel in Default mode correctly issues DropOffResources/Enter (lines 364-386)
- But left-click on a Tunnel entity while in `AwaitingTarget(DropOff)` mode falls through to the ground handler, which simply resets the interface state without issuing any command (lines 568-571)

### Required Change
In the entity-click section of `right_click_move_command` (around line 256), add handling for `CommandType::DropOff`:
- When `command_type == CommandType::DropOff` and the user left-clicks on a target entity
- Check if the target is an own Tunnel (same pattern as lines 365: `tunnel_opt.is_some() && target_owner.player_number() == Some(local_player.0)`)
- If yes: iterate selected agents, clear movement state, insert `UnitCommand::DropOffResources(target_entity)` and `AttackState::default()`
- Return interface state to `ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault)` (not Default, since agents are selected)
- If the target is not an own Tunnel, reset back to AgentDefault without issuing a command

### Files to Modify
- `src/game/units/systems/core.rs`: Add DropOff target-click handling in entity-click section of `right_click_move_command`

### Tests
Add tests verifying:
- Left-click on own Tunnel in DropOff mode issues DropOffResources command
- Left-click on non-Tunnel entity in DropOff mode resets to AgentDefault without issuing command
- Left-click on enemy Tunnel in DropOff mode resets without issuing command (must be own Tunnel)
