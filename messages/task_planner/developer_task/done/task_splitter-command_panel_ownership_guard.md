# command_panel_ownership_guard

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-command_panel_ownership_guard.md

## Task

Add an ownership guard so that the CommandPanel, hotkeys, and right-click command resolution only function when the Selection contains objects owned by the local player. Currently these systems operate on any selected entity regardless of ownership.

### Changes needed:

1. **`is_panel_visible()` in `command_panel.rs`**: Add a check that all entities in the Selection are owned by the local player (`Owner::player_number() == Some(local_player.0)`). If any selected entity is enemy-owned or neutral, the panel should be hidden. This function needs access to `LocalPlayer` and an `Owner` query — either pass them as parameters or refactor `is_panel_visible` into a system helper that can query ownership.

2. **`update_command_panel_state()` in `command_panel.rs`**: Early-return (reset to default state, clear panel target) if the selection contains non-owned entities. Add `LocalPlayer` resource and `Owner` query to the system parameters.

3. **`command_panel_hotkeys()` in `command_panel.rs`**: Already calls `is_panel_visible()` — if that function properly gates on ownership, hotkeys are covered. Verify the early return at line 841 is sufficient.

4. **`right_click_move_command()` in `core.rs`**: Add an early-return guard that checks all entities in `selected_units` are owned by `local_player`. The query already fetches `&Owner` — add a check like: `if selected_units.iter().any(|(.., owner, ..)| owner.player_number() != Some(local_player.0)) { return; }`. This prevents issuing move/attack/enter/gather/etc. commands to enemy units.

5. **`execute_command_action()` in `command_panel.rs`**: Also add an ownership guard — this function handles button clicks and hotkey actions. It already has access to `selected_owners` for structures; add equivalent check for units.

### Existing patterns to follow:
- `LocalPlayer` resource is already used in `right_click_move_command` and in the click-target resolution system at line 180 of command_panel.rs
- `Owner` component is on all `ObjectInstance` entities
- `Owner::player_number()` returns `Option<u8>`, neutral entities return `None`
- The selection system already prevents mixed player/enemy selections, so the guard just needs to check if the first entity is owned (but a full check is safer)
