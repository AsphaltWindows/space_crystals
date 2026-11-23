# automated_qa_unblock

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

## Content

Unblock the automated QA pipeline by fixing test compilation errors and updating qa_router capability patterns. Currently 54 qa_items are stuck in manual_qa because (1) auto_capabilities.txt has all patterns commented out and (2) the tests/qa/ suite has compilation errors preventing cargo test from running.

## Scope

Two targeted changes:

### 1. Fix tests/qa/ compilation errors (artifacts/developer/tests/qa/agent_object_interface_state.rs)

All 4 compilation errors are the same: missing owned_by_local_player field in SelectedUnitCapabilities struct initializers. The field was added to src/ui/types.rs but not updated in the test file. Fix by adding owned_by_local_player: true (or false as appropriate) to each of the 4 struct initializers.

### 2. Update artifacts/qa_router/auto_capabilities.txt

Replace the current commented-out content with the validated pattern set (core verification verbs, action verbs, compound patterns, build/test commands).

## QA Instructions

1. Run cargo test --test qa --no-run from artifacts/developer/ -- verify it compiles with zero errors
2. Run cargo test --test qa -- verify existing tests pass (some may be #[allow(unused)] but should not fail)
3. Run cargo test --lib -- verify no regressions in the 1886 unit tests
4. Verify auto_capabilities.txt contains the patterns listed above with no commented-out active lines
5. Verify the qa_router script (scripts/qa_router.sh) can parse the new auto_capabilities.txt without errors
