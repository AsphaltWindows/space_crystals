# fix_qa_test_compilation

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-automated_qa_unblock.md

## Task

Fix compilation errors in tests/qa/ by adding the missing `owned_by_local_player` field to all `SelectedUnitCapabilities` struct initializers.

### Background

The `owned_by_local_player: bool` field was added to the `SelectedUnitCapabilities` struct in `src/ui/types.rs` but 4 struct initializers in the test files were not updated, causing compilation failures that block `cargo test --test qa`.

### Files to modify

1. **tests/qa/agent_object_interface_state.rs** -- 2 initializers near lines 653 and 667. Add `owned_by_local_player: true,` to both (these tests check agent drop-off capability, and agents are owned by the local player in these test scenarios).

2. **tests/qa/basic_combat_unit_interface_state.rs** -- 2 initializers near lines 439 and 462. Add `owned_by_local_player: true,` to both (these tests check Peacekeeper attack/reverse capabilities for the local player's units).

### Verification

- `cargo test --test qa --no-run` must compile with zero errors
- `cargo test --test qa` must pass (existing tests should not fail)
- `cargo test --lib` must still pass with 1886+ tests (no regressions)

### Notes

- The auto_capabilities.txt file has already been updated separately -- do NOT modify it
- This is a trivial fix: add one field to 4 struct initializers, all with value `true`
