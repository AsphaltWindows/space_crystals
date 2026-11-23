# hotkey_command_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit_command_system_r1.md

## Task

Fix command hotkeys (Q=Move, W=Reverse, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround, X=Stop) so they respond when units are selected.

**Bug**: None of the command hotkeys respond when a unit is selected. The command panel grid buttons may work via mouse click, but keyboard hotkeys produce no effect. This blocks testing of HoldPosition, Stop, Patrol, Reverse, AttackGround, and AttackMove commands.

**Investigation areas**:
- The `command_panel_hotkeys` system (command_panel.rs line 946) has a guard at line 965: `if !is_panel_visible(...) || !unit_caps.owned_by_local_player { return; }`. Check that `owned_by_local_player` is being set correctly by `update_command_panel_state` (line 324+). If this field defaults to false and the update system does not run before hotkeys, all hotkeys would be silently blocked.
- Check system ordering: `update_command_panel_state` must run before `command_panel_hotkeys` so that `SelectedUnitCapabilities.owned_by_local_player` is accurate.
- Check that `is_panel_visible` returns true when units are selected (line 310-322): it checks `selection.groups.is_empty()` and filters out resource-only selections.
- Check `SelectedUnitCapabilities` resource initialization — does it default to owned_by_local_player=true or false?
- Check that the legacy `command_input_system` (commands.rs line 14) does not interfere — it returns early when panel_visible is true (line 25-33), which is correct, but verify the two systems do not conflict.
- Test: select a Peacekeeper, press E (HoldPosition), verify the unit stops. Press Q (Move), verify AwaitingTarget mode. Press A (Attack), verify AwaitingTarget mode. Press X (Stop), verify unit ceases activity.
