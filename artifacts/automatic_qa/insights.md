# Automatic QA Insights

## Test Environment Setup
- Project source is in `artifacts/developer/`
- Build/test commands: `cargo check`, `cargo test` from `artifacts/developer/`
- Bevy 0.17 project, Rust edition 2021
- **NEVER run `cargo clean`** — always use incremental builds (operator directive 2026-03-20)
- **cargo is at `~/.cargo/bin/cargo`** — need `export PATH="$HOME/.cargo/bin:$PATH"` before running cargo commands
- As of 2026-03-21: `cargo check` passes, `cargo test --no-run` passes, `cargo test` runs (293 passed, 21 failed, 8 ignored)

## Automation Capabilities
- Can run `cargo check`, `cargo test`, `cargo clippy` for compilation and test verification
- Can verify ECS component presence/absence via test harnesses
- Can verify command dispatch logic via unit tests
- Cannot verify visual rendering (requires human eyes)
- Cannot verify UX feel or interactive behavior requiring real-time input

## Pending Infrastructure
- `automated_qa_ui_state_queries` feature (assert_command_visible, assert_interface_state, etc.) — these functions now exist in `src/shared/testing/assertions.rs` and compile
- Once auto_capabilities.txt is updated by architect, qa_items can be routed here

## Common Failure Patterns
- **Test compilation fixed (2026-03-21)**: The 37 compilation errors are resolved. Tests now compile and run.
- **21 runtime test failures (2026-03-21)**: Mostly movement-related — move commands resolving to `Idle` immediately. Affected test modules: air_unit_soft_separation, autonomous_targeting, basic_combat_unit_interface_state, combat_behaviors, fix_memory_leak_oom_freeze, fix_units_moving_while_attacking, gdo_supply_tower_and_chopper, ground_unit_collision, movement_behaviors, pathfinding_diagonal_and_oscillation_fix, tunnel_expansions_and_starting_condition, automated_qa_ui_state_queries.
- **auto_capabilities.txt is empty**: All patterns commented out, so nothing routes to automatic_qa. Needs architect to update now that tests compile.

## Flaky Tests
(none confirmed yet — need multiple runs to distinguish flaky from consistently failing)
