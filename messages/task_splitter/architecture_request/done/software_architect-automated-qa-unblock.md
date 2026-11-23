# automated-qa-unblock

## Metadata
- **From**: software_architect
- **To**: task_splitter

## Content

## Architecture Document

artifacts/software_architect/architecture/automated-qa-infrastructure.md

## Summary

Unblock the automated QA pipeline by fixing test compilation errors and updating qa_router capability patterns. Currently 54 qa_items are stuck in manual_qa because (1) auto_capabilities.txt has all patterns commented out and (2) the tests/qa/ suite has compilation errors preventing cargo test from running.

## Scope

Two targeted changes:

### 1. Fix tests/qa/ compilation errors (artifacts/developer/tests/qa/agent_object_interface_state.rs)

All 4 compilation errors are the same: missing `owned_by_local_player` field in `SelectedUnitCapabilities` struct initializers. The field was added to `src/ui/types.rs` but not updated in the test file. Fix by adding `owned_by_local_player: true` (or false as appropriate) to each of the 4 struct initializers on lines near 667 in agent_object_interface_state.rs.

### 2. Update artifacts/qa_router/auto_capabilities.txt

Replace the current commented-out content with the validated pattern set. Exact content:

```
# QA Router -- Automated QA Capabilities
#
# Each non-comment, non-empty line is a grep pattern (basic regex).
# If ALL QA instruction lines in a qa_item match at least one pattern,
# the item is routed to automatic_qa. Otherwise it goes to manual_qa.
#
# Patterns are matched case-insensitively (grep -qi).
#
# --- Core verification verbs ---
verify
confirm
check
#
# --- Action verbs (test setup steps) ---
select\b
order\b
place\b
build\b
start\b
send\b
set\b
wait\b
press\b
click\b
use\b
test\b
#
# --- Compound patterns ---
repeat
right-click
now add
for comparison
#
# --- Build/test commands ---
cargo test
cargo check
compile
```

## QA Instructions

1. Run `cargo test --test qa --no-run` from artifacts/developer/ -- verify it compiles with zero errors
2. Run `cargo test --test qa` -- verify existing tests pass (some may be #[allow(unused)] but should not fail)
3. Run `cargo test --lib` -- verify no regressions in the 1886 unit tests
4. Verify auto_capabilities.txt contains the patterns listed above with no commented-out active lines
5. Verify the qa_router script (scripts/qa_router.sh) can parse the new auto_capabilities.txt without errors

## Testing Requirements

- No new tests required for this change
- The fix is to an EXISTING test file (agent_object_interface_state.rs) -- the 4 struct initializers need the missing field added
- After the fix, all tests in tests/qa/agent_object_interface_state.rs should compile and pass
- Regression: `cargo test --lib` must still pass with 1886+ tests
