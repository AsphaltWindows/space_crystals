# Ticket: Fix left-click not working for command target confirmation

## Current State
When a player selects units and enters a command mode (e.g., attack-move via the command panel), left-clicking to confirm the target does not work correctly. The `click_to_select_system` in `src/game/world/resources.rs` fires on every left-click regardless of `CommandMode` state, stealing the click from the command system and either deselecting units or changing the selection. Similarly, `drag_select_system` can initiate a drag-select during command mode.

## Desired State
When `CommandMode.mode != CommandType::Default`, left-clicks should be consumed by the command system, not the selection system. Specifically:
1. `click_to_select_system` (`src/game/world/resources.rs` ~line 92) must early-return when `CommandMode.mode != CommandType::Default`.
2. `drag_select_system` (`src/game/world/resources.rs` ~line 235) must early-return when `CommandMode.mode != CommandType::Default`.
3. The command system (`right_click_move_command` in `src/game/units/systems.rs` ~line 170) should process the left-click for command target confirmation without the selection being altered on the same frame.

## Justification
This is a bug that blocks all left-click command confirmation. The control system feature (`features/control_system.md`) defines AwaitingTarget as a state where left-click confirms a command target. Forum topic `forum/left_click_command_target_broken.md` has consensus from qa, developer, designer, and project_manager.

## QA Steps
1. Spawn units and select them.
2. Open the command panel and select attack-move (or press the attack-move hotkey).
3. Verify the interface enters AwaitingTarget/command mode (cursor should change).
4. Left-click on a ground location.
5. Verify the selected units receive the attack-move command (they begin moving toward the target).
6. Verify the selection is unchanged after the command is issued (same units still selected).
7. Verify `CommandMode` resets to `Default` after the command is confirmed.
8. Left-click on empty ground to verify normal click-to-deselect works again.
9. Select units, enter a command mode, then start a drag — verify drag-select does NOT activate.
10. Select units, enter a command mode, then right-click — verify command mode is cancelled (if right-click cancel is implemented).

## Expected Experience
After selecting units and choosing a command, left-clicking should smoothly confirm the command target. The units should immediately begin executing the command (e.g., moving toward the attack-move location). The selection highlight should remain on the same units throughout. After the command is issued, the interface returns to normal and click-to-select/drag-select work as before. No flickering of selection, no deselected units, no "nothing happened" on left-click.
