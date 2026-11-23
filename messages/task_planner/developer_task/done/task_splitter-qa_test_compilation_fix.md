# qa_test_compilation_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-automated_qa_unblock.md

## Task

Fix test compilation errors in tests/qa/ and verify auto_capabilities.txt content to unblock the automated QA pipeline.

### 1. Fix SelectedUnitCapabilities struct initializers (4 errors in 2 files)

The `owned_by_local_player: bool` field was added to `SelectedUnitCapabilities` in `src/ui/types.rs` (line 416) but not updated in the test files. Add the missing field to all 4 struct initializers:

- `tests/qa/agent_object_interface_state.rs` line 653: add `owned_by_local_player: true,`
- `tests/qa/agent_object_interface_state.rs` line 667: add `owned_by_local_player: true,`
- `tests/qa/basic_combat_unit_interface_state.rs` line 439: add `owned_by_local_player: true,`
- `tests/qa/basic_combat_unit_interface_state.rs` line 462: add `owned_by_local_player: true,`

Use `true` for all since these tests simulate local player scenarios.

### 2. Verify auto_capabilities.txt matches required content

`artifacts/qa_router/auto_capabilities.txt` should contain the following active (non-commented) patterns. The software_architect has already updated this file, but verify it matches exactly and fix if needed:

Active patterns required:
- Core verification verbs: verify, confirm, check
- Action verbs (with \b word boundary): select, order, place, build, start, send, set, wait, press, click, use, test
- Compound patterns: repeat, right-click, now add, for comparison
- Build/test commands: cargo test, cargo check, compile

Remove any extra patterns not in this list (e.g., `observe` if present). The file currently has `observe` which is not in the specified set.

### Verification

- `cargo test --test qa --no-run` must compile with zero errors
- `cargo test --test qa` must pass (existing tests should not fail)
- `cargo test --lib` must still pass with 1886+ tests (no regressions)
- auto_capabilities.txt must contain exactly the patterns listed above
