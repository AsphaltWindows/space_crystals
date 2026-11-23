# test_harness_selection_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Fix the TestHarness::set_selection() infrastructure bug and the 2 Category D interface state test failures it causes.

### Problem

TestHarness::set_selection() (src/shared/testing/harness.rs:264) only adds the Selected ECS component to entities. It does NOT update the Selection resource's groups field. When update_command_panel_state runs during test_app.step(), it reads selection.groups to determine active_type(). Since groups is empty, active_type() returns None and the system resets to Default.

### Fix Required

1. In set_selection(), after adding/removing Selected components, also update the Selection resource (src/shared/types.rs:138):
   - Build SelectionGroup entries from the selected entities by reading their ObjectEnum component
   - Set selection.groups to the built groups
   - Set selection.active_group_index to Some(0) if groups is non-empty, None otherwise

2. In clear_selection(), ensure Selection resource groups is also cleared.

3. Verify the 2 Category D tests now pass:
   - automated_qa_ui_state_queries::step_1_get_interface_state_barracks (should show BarracksMenu)
   - automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu (should show DcBuildMenu)

### Files
- src/shared/testing/harness.rs -- fix set_selection() and clear_selection()
- tests/qa/automated_qa_ui_state_queries.rs -- verify tests pass (no changes expected)

### Verification
- cargo test --test qa automated_qa_ui_state_queries -- both tests must pass
- cargo test --lib -- no regressions
