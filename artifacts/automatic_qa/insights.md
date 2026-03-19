# Automatic QA Insights

## Test Environment Setup
- Project source is in `artifacts/developer/`
- Build/test commands: `cargo check`, `cargo test` from `artifacts/developer/`
- Bevy 0.17 project, Rust edition 2021

## Automation Capabilities
- Can run `cargo check`, `cargo test`, `cargo clippy` for compilation and test verification
- Can verify ECS component presence/absence via test harnesses
- Can verify command dispatch logic via unit tests
- Cannot verify visual rendering (requires human eyes)
- Cannot verify UX feel or interactive behavior requiring real-time input

## Pending Infrastructure
- `automated_qa_ui_state_queries` feature (assert_command_visible, assert_interface_state, etc.) is not yet implemented
- Once available, many UI state verification steps currently tagged [human]/[semi] can be automated
- Re-tagging of QA steps to [auto] should be gated on this infrastructure existing

## Common Failure Patterns
(none yet)

## Flaky Tests
(none yet)
