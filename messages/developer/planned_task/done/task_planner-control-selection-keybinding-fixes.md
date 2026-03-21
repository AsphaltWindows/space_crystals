# control-selection-keybinding-fixes

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-control-state-selection.md

## Task

Fix two keybinding issues in the control/selection system:

### 1. Control Group Add: Change from Shift+Num to Ctrl+Shift+Num

In `game/world/resources.rs`, `control_group_system` (~line 645):
- Currently Ctrl+Num = Assign and Shift+Num = Add
- The design spec requires Ctrl+Num = Assign and Ctrl+Shift+Num = Add
- Fix: Check `ctrl_held && shift_held` BEFORE checking `ctrl_held` alone (order matters since Ctrl+Shift has both flags set)
- The branch order should be: (1) ctrl+shift = Add, (2) ctrl-only = Assign, (3) plain number = Recall

### 2. Shift-Tab Backward Group Cycling

Add backward cycling (Shift-Tab) to complement the existing forward Tab cycling:

**In `shared/types.rs`**: Add `cycle_active_group_backward()` method to `Selection`:
- If active_group_index is Some(idx) and groups is non-empty: set to `(idx + groups.len() - 1) % groups.len()`
- Add unit tests for backward cycling (wrapping, single group, empty)

**In `game/world/resources.rs`**, `active_group_cycle_system` (~line 811):
- Add check for Shift+Tab (keyboard shift pressed AND Tab just pressed) -> call `cycle_active_group_backward()`
- Existing Tab-only check should verify shift is NOT pressed to avoid double-cycling

**In `ui/command_panel.rs`** (if Tab cycling is also handled there for commandable groups):
- Add matching Shift-Tab backward cycling logic alongside existing Tab forward cycling

## Technical Context

### Files to modify:

**1. `artifacts/developer/src/game/world/resources.rs`**

- **`control_group_system`** (line 645): The branch structure at lines 688-710 needs reordering. Currently:
  - Line 688: `if ctrl_held` -> Assign
  - Line 700: `else if shift_held` -> Add
  - Line 710: `else` -> Recall
- Must change to:
  - `if ctrl_held && shift_held` -> Add (lines 700-709 logic)
  - `else if ctrl_held` -> Assign (lines 688-699 logic)
  - `else` -> Recall (unchanged)
- Both `ctrl_held` and `shift_held` are already computed at lines 663-664.

- **`active_group_cycle_system`** (line 811): Currently checks `keyboard.just_pressed(KeyCode::Tab)` at line 827 with no shift guard. Add `keyboard` already available as `Res<ButtonInput<KeyCode>>` (line 812). Changes:
  - Before existing Tab check (line 827): add `if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)` check combined with `keyboard.just_pressed(KeyCode::Tab)` -> call `selection.cycle_active_group_backward()` with same logging pattern.
  - Guard existing Tab check: add `&& !shift_held` so plain Tab only fires without Shift.
  - The `has_commandable_groups` filter (line 824-825) applies to both forward and backward cycling here (this system only handles resource-only groups; commandable groups are handled in command_panel.rs).

**2. `artifacts/developer/src/shared/types.rs`**

- Add `cycle_active_group_backward()` method to `Selection` impl, right after `cycle_active_group()` (line 226). Pattern to follow exactly from `cycle_active_group()` (line 220-226):
```rust
pub fn cycle_active_group_backward(&mut self) {
    if let Some(idx) = self.active_group_index {
        if !self.groups.is_empty() {
            self.active_group_index = Some((idx + self.groups.len() - 1) % self.groups.len());
        }
    }
}
```
- Add 3 unit tests after existing cycle tests (lines 695-725), following the same pattern:
  - `selection_cycle_active_group_backward`: 2 groups, cycle backward from 0 wraps to 1, then back to 0
  - `selection_cycle_active_group_backward_single_group`: stays at 0
  - `selection_cycle_active_group_backward_empty_selection`: stays None

**3. `artifacts/developer/src/ui/command_panel.rs`**

- **`command_panel_hotkeys`** (line 828): Tab cycling at lines 851-863 handles commandable groups. Add Shift-Tab backward cycling block BEFORE the existing Tab block:
```rust
// Shift-Tab: Cycle active group backward
if (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight))
    && keyboard.just_pressed(KeyCode::Tab)
{
    if selection.groups.len() > 1 {
        selection.cycle_active_group_backward();
        if !matches!(*interface_state, ObjectInterfaceState::Default) {
            *interface_state = ObjectInterfaceState::Default;
        } else {
            interface_state.set_changed();
        }
        info!("Group cycling backward: active group index {:?}", selection.active_group_index);
    }
    return;
}
```
- Guard existing Tab check (line 852): change `if keyboard.just_pressed(KeyCode::Tab)` to also check shift is NOT pressed: `if keyboard.just_pressed(KeyCode::Tab) && !(keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight))`

### Key types and resources:
- `Selection` (shared/types.rs line ~170): `groups: Vec<SelectionGroup>`, `active_group_index: Option<usize>`
- `ControlGroups` (shared/types.rs): `groups: [Vec<Entity>; 10]`
- `ButtonInput<KeyCode>`: Bevy keyboard input resource, `.pressed()` for held keys, `.just_pressed()` for edge detection
- `ObjectInterfaceState` (ui/types.rs): reset to Default on group change

### System ordering:
- `control_group_system` runs in `Update` in `DiagCategory::Faction` set (mod.rs line 76)
- `active_group_cycle_system` runs in same set (mod.rs line 84)
- `selection_group_sync_system` runs after both (mod.rs line 81-83)
- No ordering changes needed for this task

## Dependencies

None. This task is standalone — it modifies existing keybinding logic and adds a new Selection method. No dependency on other planned tasks.
