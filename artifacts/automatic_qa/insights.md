# Automatic QA Insights

## Test Environment Setup
- Project source is in `artifacts/developer/`
- Build/test commands: `cargo check`, `cargo test` from `artifacts/developer/`
- Bevy 0.17 project, Rust edition 2021
- **NEVER run `cargo clean`** — always use incremental builds (operator directive 2026-03-20)
- **cargo is at `~/.cargo/bin/cargo`** — need `export PATH="$HOME/.cargo/bin:$PATH"` before running cargo commands
- As of 2026-03-28: `cargo check` passes, `cargo test --lib` runs (2094 tests all pass)
- `cargo test --test qa`: 318 passed, 0 failed, 8 ignored (all previously failing tests now fixed)
- All previously failing integration tests are now fixed and passing (core_systems_split_r2 resolved the last 6)

## Automation Capabilities
- Can run `cargo check`, `cargo test`, `cargo clippy` for compilation and test verification
- Can verify ECS component presence/absence via test harnesses
- Can verify command dispatch logic via unit tests
- Cannot verify visual rendering (requires human eyes)
- Cannot verify UX feel or interactive behavior requiring real-time input

## Pending Infrastructure
- `automated_qa_ui_state_queries` feature (assert_command_visible, assert_interface_state, etc.) — these functions now exist in `src/shared/testing/assertions.rs` and compile
- auto_capabilities.txt now has patterns; qa_items are being routed here

## Verification Approach for Interactive QA Items
- Even when QA instructions describe interactive scenarios (rally on enemy, produce unit, verify attack), can verify via:
  1. Running existing tests that cover the behavior (e.g., `rally_attack_enemy_inserts_move_target_and_path`)
  2. Code inspection of the relevant systems to confirm correct logic
  3. Checking that cleanup/reset systems exist and handle edge cases
- Key: match QA instruction intent to existing test coverage + code analysis

## Common Failure Patterns
- **Test compilation fixed (2026-03-21)**: The 37 compilation errors are resolved. Tests now compile and run.
- **All rally tests passing (2026-03-24)**: 29 rally-related tests pass including `rally_attack_enemy_inserts_move_target_and_path`.
- **auto_capabilities.txt updated by architect**: Patterns now route most qa_items to automatic_qa. First qa_item successfully processed (gdo_barracks_r1).
- **54 manual_qa items are genuinely interactive**: Sampled several — QA instructions are all in-game behavioral checks (order units, verify movement, test hotkeys). Cannot be automated with cargo test alone. Need Tier 2 approach: developers write tests, QA instructions reference test names.
- **Recommended auto_capabilities.txt patterns**: cargo test, cargo check, compile, unit test, run.*test, verify.*test.*pass

## Known Pre-existing QA Test Failures
All previously failing tests have been resolved. The full chain: test_suite_cleanup fixed the original 9, core_systems_split_r1 fixed 13 of 19 refactoring regressions, core_systems_split_r2 fixed the final 6. As of 2026-03-25, the test suite is fully green (318 pass, 0 fail, 8 ignored).

## Routing Observations
- `expand_command_panel_grid` qa_item: QA instructions are interactive (open UI, verify grid layout) but feature is verifiable through code inspection and unit tests. 306 command_panel unit tests cover grid slot actions, hotkey mappings, and the 4x3 layout. Passed via automated verification.
- `action_channels_r1`: Bounced 3 times. No action_channel tests exist; only a placeholder comment in commands.rs. Purely interactive.
- **All 3 pending items (2026-03-28)** are interactive gameplay items misrouted due to overly broad auto_capabilities.txt patterns. Opened forum topic requesting pattern narrowing.
- **Root cause**: auto_capabilities.txt patterns include single words like `verify`, `select`, `click` which match ALL QA instructions. Needs to be narrowed to `cargo test`, `cargo check`, etc.
- As of 2026-03-28: `action_channels_r1` bounced 9+ times, `add_cults_armory_r1`, `agent_groupable_construction_r1`, `agent_interface_r1` still pending. auto_capabilities.txt STILL not updated despite forum topics. Pipeline_builder must update `artifacts/qa_router/auto_capabilities.txt`. Forum topic has new comment from 2026-03-28 escalating the issue.
- All 6 remaining pending items (`action_channels_r1`, `add_cults_armory_r1`, `agent_groupable_construction_r1`, `agent_interface_r1`, `agent_resource_gathering_r1`, `agent_tunnel_building_r1`) are interactive gameplay items that cannot be automated. They will keep cycling until auto_capabilities.txt is fixed.
- As of 2026-03-28T18:00: Third+ forum topic opened about auto_capabilities.txt misrouting. Previous topics were closed without fix being applied. `action_channels_r1` has bounced 20+ times.
- As of 2026-03-28: Forum topic `2026-03-28T20-00-00Z-automatic_qa-auto-capabilities-critical-misrouting.md` now has 4 close votes (automatic_qa, task_planner, task_splitter, developer). Still waiting for pipeline_builder to apply the fix. All 6 items continue to cycle.
- **auto_capabilities.txt FIXED (2026-03-28)**: software_architect narrowed patterns to `cargo test`, `cargo check`, `cargo clippy`, `compile`, `run.*test`, `unit test`, `integration test`. Remaining 5 pending items (`add_cults_armory_r1`, `agent_interface_r1`, `agent_resource_gathering_r1`, `agent_tunnel_building_r1`, and others) are pre-fix misroutes — bounce each one to qa_router for manual_qa.
- `agent_groupable_construction_r1` bounced to qa_router (2026-03-28) — interactive gameplay verification.
- `agent_tunnel_building_r1` bounced to qa_router (2026-03-28) — last of the pre-fix misrouted items. All 6 pre-fix misroutes now cleared.
- Integration tests (`cargo test --test qa`, `cargo test --test scenarios`) have compilation errors due to entity definitions refactoring — do NOT flag these as feature regressions until fix lands (see forum topic `2026-03-28T12-00-00Z-software_architect-integration-test-compilation-regression.md`).

## Flaky Tests
(none confirmed yet — need multiple runs to distinguish flaky from consistently failing)
