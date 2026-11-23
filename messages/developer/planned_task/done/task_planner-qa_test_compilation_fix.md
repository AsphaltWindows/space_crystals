# qa_test_compilation_fix

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Current State (IMPORTANT)

Investigation reveals the **compilation errors are already fixed** -- all 4 `SelectedUnitCapabilities` initializers in the test files already include `owned_by_local_player: true`. Running `cargo test --test qa --no-run` compiles successfully with 0 errors (only 13 warnings, all about deprecated Bevy methods and unused variables -- not in scope for this task).

The remaining work is:

### File 1: `artifacts/qa_router/auto_capabilities.txt`
- **Line 49**: Remove the `observe` pattern. This is the only extra pattern not in the required set.
- The rest of the file (lines 18-46) already matches the required patterns exactly: verify, confirm, check, select, order, place, build, start, send, set, wait, press, click, use, test, repeat, right-click, now add, for comparison, cargo test, cargo check, compile.
- The file uses `# ---` comment blocks as section separators with descriptive headers. Preserve this style.

### File 2: `artifacts/developer/tests/qa/agent_object_interface_state.rs` (lines 653, 668)
- Already has `owned_by_local_player: true` on both initializers. **No change needed.**

### File 3: `artifacts/developer/tests/qa/basic_combat_unit_interface_state.rs` (lines 439, 463)
- Already has `owned_by_local_player: true` on both initializers. **No change needed.**

### File 4: `artifacts/developer/src/ui/types.rs` (line 416)
- This is the source of truth for the `SelectedUnitCapabilities` struct. `owned_by_local_player: bool` is already defined. **No change needed.**

### Runtime Test Failures (NOT in scope)

`cargo test --test qa` currently has 9 runtime failures (309 pass, 9 fail). These are pre-existing failures unrelated to compilation -- they are assertion failures in test logic (e.g., autonomous_targeting::step_1 expects auto-engage behavior that is not triggering). The task's verification criterion says "existing tests should not fail" but these failures pre-date this task. The developer should confirm the 9 failures are unchanged after the auto_capabilities.txt edit (i.e., no regressions introduced).

Failing tests (for reference):
- automated_qa_ui_state_queries::step_1_get_interface_state_barracks
- automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu
- autonomous_targeting::step_6_infantry_auto_targeting
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- basic_combat_unit_interface_state::step_3_stop_immediate_command
- combat_behaviors::step_10_hold_position_engages_nearby
- gdo_supply_tower_and_chopper::dc_construction_cost_supply_tower
- movement_behaviors::step_8_stop_command
- tunnel_expansions_and_starting_condition::hq_production_ejects_agent_without_rally_point

### QA Router Script (`scripts/qa_router.sh`)
- Parses auto_capabilities.txt by skipping comment lines (`#`) and empty lines.
- Each remaining line becomes a grep pattern matched case-insensitively against QA instruction lines.
- Removing `observe` simply tightens the routing -- items with only "observe" verbs will go to manual_qa instead of automatic_qa.

## Dependencies

None. This is a standalone fix. The auto_capabilities.txt file is read by the qa_router script node, not by any Rust code.
