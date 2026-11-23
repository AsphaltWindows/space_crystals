# fix_test_harness_selection_groups

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-core_systems_split_r2.md

## Task

Fix TestHarness::set_selection() in src/shared/testing/harness.rs to populate the Selection resource groups, not just the Selected ECS component. This is the root cause of 2 failing QA integration tests.

### Problem

set_selection() (harness.rs:264) only adds/removes the Selected component on entities. It does NOT update the Selection resource (groups field). When update_command_panel_state (ui/command_panel/state.rs:139) runs, it reads selection.active_type() which returns None because groups is empty. The system then falls through to the 'no units selected' branch at line 318 and resets ObjectInterfaceState to Default.

### Fix

1. In set_selection(), after adding Selected components, also update the Selection resource:
   - Query each entity's ObjectInstance to get its ObjectEnum (object_type field)
   - Group entities by ObjectEnum into SelectionGroup entries (each group has object_type + vec of entities)
   - Set selection.groups to the built groups
   - Set selection.active_group_index to Some(0) if groups is non-empty, None otherwise

2. In clear_selection(), ensure Selection resource groups is also cleared (it calls set_selection(&[]) so this should happen automatically).

3. The Selection resource type is defined in src/shared/types.rs — check its fields there. SelectionGroup has object_type: ObjectEnum and entities: Vec<Entity>.

### Verification

These 2 tests must pass after the fix:
- cargo test --test qa automated_qa_ui_state_queries::step_1_get_interface_state_barracks
- cargo test --test qa automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu

Also verify no regressions:
- cargo test --lib
- cargo test --test qa
