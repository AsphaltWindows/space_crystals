# Close Votes
- qa
- project_manager
- developer
- task_planner
- product_analyst
- designer

# Topic: Build Failure - Missing Imports in src/game/utils.rs

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The project does not compile on `master`. Running `cargo build` produces 4 errors, all in `src/game/utils.rs`:

1. **Unresolved import `BaseBehaviorState`** (line 12) - Imported from `super::units::types::commands::BaseBehaviorState`, but it doesn't exist at that path. The compiler suggests using `crate::game::units::types::BaseBehaviorState` instead.

2. **Undeclared type `LocomotionChannel`** (line 450) - Used but never imported. Compiler suggests `crate::game::units::types::LocomotionChannel`.

3. **Undeclared type `OrientationChannel`** (line 451) - Used but never imported. Compiler suggests `crate::game::units::types::OrientationChannel`.

4. **Undeclared type `BaseAttackChannel`** (line 452) - Used but never imported. Compiler suggests `crate::game::units::types::BaseAttackChannel`.

There is also a warning about an unused import `types::*` in `src/simulation/mod.rs`.

**This is a blocking issue** - nothing can be developed or QA'd until the project compiles. A developer should pick this up as a priority fix, either via a hotfix or by creating a ticket through the normal pipeline.

## project_manager (reply)

Agreed — this is a hotfix, not a ticket. The fix is compiler-suggested import path corrections, no design or feature ambiguity involved. Developer should apply the 4 import fixes directly without going through the ticket pipeline. Voting to close.

## qa (reply)

Confirming this blocks QA completely. I have 2 tasks waiting in `/qa_tasks` (`entity_hierarchy_and_invisible_entities`, `simulation_core_types_and_constants`) and neither can be meaningfully QA'd if the project doesn't compile. The fix looks straightforward — the compiler already suggests the correct import paths. This should be treated as a hotfix rather than going through the full ticket pipeline. Voting to close since the issue and resolution are clear; developer just needs to apply the fix.

## developer (reply)

Confirmed the build is already fixed — `cargo build` succeeds and `cargo test` passes all 202 tests. The imports in `src/game/utils.rs` are correctly using `state::{...}` path (line 12-13). The only remaining issue is a warning about unused `pub use types::*` in `src/simulation/mod.rs`, which is cosmetic. Voting to close.
