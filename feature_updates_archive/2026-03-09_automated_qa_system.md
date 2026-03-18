# Feature Update: Automated QA System — UI State Queries

## Modified Feature File
`features/automated_qa_system.md`

## Relevant Design Sources
- Forum topic: `qa_automation_epic_ui_testing_push.md` (operator on behalf of user)
- `features/control_system.md` (defines the UI components being queried)

## Summary of Modifications

### Added: UI State Queries (Layer 1 Query Categories)
New query category for client-side ECS components (ControlState, CommandPanel, InfoPanel, SelectionPanel). Six new queries:
- `get_interface_state()` — current ObjectInterfaceState
- `get_visible_commands()` — command panel button inventory with slot, action, enabled, and common/group classification
- `get_active_group()` — which SelectionGroup is active
- `get_selection_groups()` — all groups with type and member count
- `get_info_panel()` — info panel display data
- `get_selection_panel_portraits()` — portrait grid with ActiveGroup highlighting

### Added: UI State Assertion Helpers
Six new assertion helpers corresponding to the UI queries:
- `assert_interface_state`, `assert_command_visible`, `assert_command_not_visible`, `assert_command_enabled`, `assert_active_group_type`, `assert_info_panel_shows`

### Updated: Automation Coverage Estimate
Revised from ~72% to ~82% automatable. ~8 UI-focused QA tasks (command panel states, button inventories, interface transitions) shift from partially-automatable to fully/mostly automatable with UI State Queries.

### Motivation
8 of 11 current QA tasks are UI-state focused (deterministic state checks like "button X in slot Y"). These are ECS-queryable and don't require human visual verification. Adding UI State Queries to the TestHarness unblocks automated QA for these tasks.
