# fix_qa_test_compilation

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Struct definition (authoritative)
File: `artifacts/developer/src/ui/types.rs` lines 401-417. The `SelectedUnitCapabilities` struct has 7 fields. The field `owned_by_local_player: bool` is the last field (line 416), added after `chopper_has_supplies`. The struct derives `Default`, so the default value is `false`, but all 4 test initializers use explicit struct literal syntax (not `..Default::default()`), so each one needs the field added explicitly.

### Exact change sites (4 total)

1. `artifacts/developer/tests/qa/agent_object_interface_state.rs` line 659: after `chopper_has_supplies: false,` add `owned_by_local_player: true,` (inside `agent_drop_off_disabled_when_not_carrying` test)

2. `artifacts/developer/tests/qa/agent_object_interface_state.rs` line 673: after `chopper_has_supplies: false,` add `owned_by_local_player: true,` (inside `agent_drop_off_enabled_when_carrying` test)

3. `artifacts/developer/tests/qa/basic_combat_unit_interface_state.rs` line 445: after `chopper_has_supplies: false,` add `owned_by_local_player: true,` (inside `step_14_peacekeeper_no_attack_ground_capability` test)

4. `artifacts/developer/tests/qa/basic_combat_unit_interface_state.rs` line 468: after `chopper_has_supplies: false,` add `owned_by_local_player: true,` (inside `step_15_peacekeeper_no_reverse_capability` test)

### Pattern to follow
Each initializer currently ends with:
```rust
        chopper_has_supplies: false,
    };
```
Change to:
```rust
        chopper_has_supplies: false,
        owned_by_local_player: true,
    };
```

All 4 sites use `true` because the tests simulate the local player's own units.

### No other files need changes
The struct definition in `src/ui/types.rs` already has the field. The `auto_capabilities.txt` is already updated. Only these 4 test initializers are missing the field.

## Dependencies

None. This is a standalone compilation fix with no dependencies on other tasks or systems.
