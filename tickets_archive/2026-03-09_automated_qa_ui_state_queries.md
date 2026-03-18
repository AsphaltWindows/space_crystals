# Ticket: UI State Query Extensions to TestHarness

## Current State
The TestHarness (`src/testing/harness.rs`) provides command and query methods for game simulation state (entity position, health, attack phase, behavior, resources, visibility, etc.) and corresponding assertion helpers. It does not support querying client-side UI ECS components — command panel state, button inventory, info panel content, or selection panel portraits.

## Desired State
The TestHarness exposes six new UI State Query methods and six new assertion helpers for client-side ECS components:

**Queries:**
- `get_interface_state() -> ObjectInterfaceState` — current command panel mode (DefaultState, ConstructionSubmenu, AwaitingTarget, etc.)
- `get_visible_commands() -> Vec<(SlotPosition, CommandButtonAction, IsEnabled, IsCommon)>` — all commands currently displayed in the command panel with grid slot, enabled state, and common/group classification
- `get_active_group() -> Option<SelectionGroup>` — which SelectionGroup's commands are currently displayed
- `get_selection_groups() -> Vec<SelectionGroup>` — all SelectionGroups in current selection with type and member count
- `get_info_panel() -> InfoPanelContent` — current info panel display data
- `get_selection_panel_portraits() -> Vec<(Entity, IsHighlighted)>` — portrait grid contents with ActiveGroup highlighting

**Assertion helpers:**
- `assert_interface_state(expected_state)`
- `assert_command_visible(slot, command_action)`
- `assert_command_not_visible(command_action)`
- `assert_command_enabled(slot)`
- `assert_active_group_type(expected_object_type)`
- `assert_info_panel_shows(entity)`

UI state queries must operate after UI sync systems have run. Since `advance_frames()` runs all systems, callers should call `advance_frames(1)` before querying to ensure fresh UI state.

## Justification
Per `features/automated_qa_system.md` Layer 1 UI State Queries section. 8 of 11 current QA tasks are UI-state focused (command panel states, button inventories, interface transitions). These are deterministic ECS state checks that don't require human visual verification. Adding UI State Queries to the TestHarness shifts ~8 tasks from `[human]` to `[auto]`, raising overall QA automation from ~72% to ~82%. Originated from forum topic `qa_automation_epic_ui_testing_push.md`.

## QA Steps
1. [auto] Spawn a Barracks, select it, advance 1 frame, call `get_interface_state()` — verify it returns `ObjectInterfaceState::DefaultState`.
2. [auto] With a Barracks selected in DefaultState, call `get_visible_commands()` — verify the result contains the expected build commands in their correct slot positions with correct enabled states.
3. [auto] Select a unit with a construction submenu (e.g., Drone Controller), trigger the submenu, advance 1 frame, call `get_interface_state()` — verify it returns `ObjectInterfaceState::ConstructionSubmenu`.
4. [auto] Select multiple units of different types (creating multiple SelectionGroups), advance 1 frame, call `get_selection_groups()` — verify it returns the correct groups with correct types and member counts.
5. [auto] With a multi-group selection, call `get_active_group()` — verify it returns the expected active group. Switch active group and re-query — verify the return value changes.
6. [auto] Select a single unit, advance 1 frame, call `get_info_panel()` — verify the returned `InfoPanelContent` matches the entity's name, health, and portrait data.
7. [auto] Select multiple units, advance 1 frame, call `get_selection_panel_portraits()` — verify it returns the correct entities with ActiveGroup highlighting on the expected subset.
8. [auto] Use `assert_command_visible(slot, action)` for a known visible command — verify it passes. Use `assert_command_not_visible(action)` for a command not in the panel — verify it passes. Use `assert_command_visible` for a command not present — verify it panics/fails.
9. [auto] Use `assert_interface_state` with the correct state — verify pass. Use it with the wrong state — verify failure.
10. [auto] Use `assert_active_group_type` and `assert_info_panel_shows` with correct and incorrect values — verify pass and failure respectively.

## Expected Experience
All 10 QA steps run as automated Rust tests via `cargo test --features testing`. Each test spawns a headless TestApp, sets up a scenario using existing TestHarness commands (spawn units/structures, set selection, advance frames), then calls the new UI State Query methods and assertion helpers. Tests pass when queries return correct data matching the ECS component state, and assertion helpers correctly pass on match / fail on mismatch. No visual output or human interaction needed.
