# command_panel_action_decomposition

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-command_panel_split.md

## Task

Decompose the ~560-line `execute_command_action()` function in `src/ui/command_panel/actions.rs` into 5 category-specific sub-dispatchers. The main function should become a thin match dispatcher (~30 lines) that delegates to the sub-functions.

### Sub-dispatchers to Create

All in `actions.rs` (or as `pub(crate)` helpers callable from actions.rs):

1. **`execute_dc_action()`** -- Deployment Center build/cancel/placement actions (CommandButtonAction variants related to DC construction menus)
2. **`execute_production_action()`** -- Barracks/HQ/SupplyTower/RC/Armory train/cancel actions (all production queue CommandButtonAction variants)
3. **`execute_unit_command_action()`** -- Move/Stop/Patrol/Attack/Hold/Reverse/Enter/Gather command actions (all unit command CommandButtonAction variants)
4. **`execute_tunnel_action()`** -- All tunnel-specific actions (delegates to functions already in tunnel_actions.rs)
5. **`execute_placement_action()`** -- Enter placement mode actions

### Implementation Steps

1. Identify the match arms in `execute_command_action()` and group them by category
2. Extract each group into its own function with the same parameters needed from the parent
3. Replace the extracted match arms in `execute_command_action()` with calls to the sub-dispatchers
4. The resulting `execute_command_action()` should be ~30 lines: a match on the action category delegating to the 5 sub-functions
5. All sub-dispatchers should be in actions.rs (tunnel actions can delegate to tunnel_actions.rs functions that already exist)

### Constraints

- No behavioral changes -- the exact same actions must execute for the exact same inputs
- This task depends on the directory split being completed first (actions.rs must exist)
- All existing tests for execute_command_action must continue to pass
- The sub-dispatchers do not need their own separate tests since existing tests cover the behavior through execute_command_action

### Verification

- `cargo check` must pass
- `cargo test --lib` must pass
- `cargo test --test qa` must pass
- `execute_command_action()` should be ~30 lines (thin dispatcher)
