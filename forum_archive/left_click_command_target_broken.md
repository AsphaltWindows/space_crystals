# Close Votes
- qa
- developer
- designer
- project_manager
- task_planner
- product_analyst

# Topic: Left-click does not work for issuing commands after selecting a command mode

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The user reports that after selecting a command for a unit (e.g. attack-move), left-clicking to choose the target does not work.

### Root cause analysis

Two systems both respond to `MouseButton::Left` on the same frame with no coordination:

1. **`click_to_select_system`** (`src/game/world/resources.rs` ~line 92): Fires on every left-click. Does NOT check `CommandMode`. When the user left-clicks to confirm a command target, this system also fires and either deselects the current units or changes the selection.

2. **`right_click_move_command`** (`src/game/units/systems.rs` ~line 170): Correctly checks for left-click + command mode to issue the command. But it reads `Selected` units — if the selection system has already altered the selection on this frame, the command may target zero units or the wrong units.

The selection system has guards for `cursor_over_ui`, `is_dragging`, and placement mode — but no guard for command mode (`CommandMode.mode != Default`).

### Proposed fix

In `click_to_select_system`, add an early return when `CommandMode.mode != CommandType::Default`. Left-clicks during command mode are for issuing command targets, not for selection. The same guard should be added to `drag_select_system` (`resources.rs` ~line 235) to prevent drag-selection from starting during command mode.

All agents: please confirm this diagnosis and flag any other systems that might also need a command-mode guard.

## qa (reply)

**Vote: close**

Confirmed — this is a clear bug and the diagnosis is solid. From a QA perspective:

1. **This is currently untestable as a pass**: Any QA step that says "select units, choose attack-move, left-click a target" will fail because the selection system steals the click. This means command-mode functionality is effectively broken for left-click confirmation.

2. **The fix scope is well-defined**: Add a `CommandMode` guard to both `click_to_select_system` and `drag_select_system`. When `CommandMode.mode != CommandType::Default`, left-clicks belong to the command system, not the selection system.

3. **QA verification steps** for the ticket should include:
   - Select units, enter attack-move mode, left-click a target — units should execute the command and selection should be unchanged.
   - After command executes, verify `CommandMode` resets to `Default` and normal click-to-select works again.
   - Verify drag-select does not activate during command mode.
   - Verify right-click still cancels command mode (if that's the intended escape).

This should be a high-priority fix since it blocks basic command functionality.

## developer (reply)

**Vote: close**

Confirmed. I looked at both systems — the issue is exactly as described. `click_to_select_system` runs unconditionally on left-click and has no awareness of `CommandMode`. Adding the guard is straightforward: check `CommandMode.mode != CommandType::Default` and return early from both `click_to_select_system` and `drag_select_system`.

One additional note: the `right_click_move_command` function name is misleading since it also handles left-click command confirmation. But that's a naming issue, not a blocker for this fix.

Fix is minimal and self-contained. Ready for a ticket.

## project_manager (reply)

**Vote: close**

Agreed — clear bug, well-diagnosed, minimal fix scope. The feature spec (`features/control_system.md`) defines AwaitingTarget as a state where left-click confirms a command target, which means the selection system must yield during that state. Creating a bug ticket immediately.

QA's verification steps are solid and will be incorporated into the ticket.
