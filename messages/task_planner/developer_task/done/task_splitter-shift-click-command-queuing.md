# shift-click-command-queuing

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit-command-system.md

## Task

Add shift-click command queuing to all command-issuing code paths so that holding Shift appends commands to the `CommandQueue` instead of replacing the current command.

### Context

The design spec says 'Commands can be queued by the player (e.g., shift-click)'. Currently, all command issuing paths (right-click in `right_click_move_command`, hold_position_system, stop_command_system, and `execute_command_action` in command_panel.rs) directly replace the unit's `UnitCommand` component. The `CommandQueue` component exists but nothing pushes to it.

### Requirements

1. **Detect Shift key** in all command-issuing code paths:
   - `right_click_move_command` in `src/game/units/systems/core.rs` — all branches that insert `UnitCommand` (Move, Attack, Patrol, AttackMove, AttackGround, Reverse, Enter, plus Agent/Chopper specific commands)
   - `hold_position_system` in `src/game/units/systems/commands.rs`
   - `stop_command_system` in `src/game/units/systems/commands.rs`
   - `execute_command_action` in `src/ui/command_panel.rs` — the HoldPosition and Stop immediate commands

2. **Queuing behavior**:
   - When Shift is NOT held: clear `CommandQueue`, replace `UnitCommand` (current behavior — no change needed)
   - When Shift IS held: push the new command onto `CommandQueue` instead of replacing `UnitCommand`. Do NOT clear movement state or current command.
   - The `ButtonInput<KeyCode>` resource already exists in all these systems — check `keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)`

3. Add tests verifying:
   - Non-shift command replaces UnitCommand and clears queue
   - Shift command appends to CommandQueue without replacing current UnitCommand
   - Multiple shift-clicks accumulate commands in queue order

### Design reference
See `artifacts/designer/design/control_system.md` under 'Unit Command': 'Commands can be queued by the player (e.g., shift-click).'
