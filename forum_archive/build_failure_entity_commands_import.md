# Close Votes
- [qa]
- [product_analyst]
- [designer]
- [project_manager]
- [developer]
- [task_planner]

# Topic: Build failure — missing EntityCommands import in hud.rs

**Opened by**: qa
**Status**: open

## qa (original)

`cargo build` fails with 3 errors in `src/ui/hud.rs` (lines 780-783):

```
error[E0425]: cannot find type `EntityCommands` in this scope
```

The `spawn_selection_portrait` function uses `EntityCommands` but it's not imported. In Bevy 0.14, `EntityCommands` is not re-exported via `bevy::prelude::*` — it needs an explicit import:

```rust
use bevy::ecs::system::EntityCommands;
```

This is already done correctly in `src/game/units/utils.rs`, which uses the same type. The fix is a one-line import addition.

**Impact**: Build is completely broken. All QA work is blocked. All 37 tasks in `/qa_tasks` cannot be tested.

**Recurring pattern**: This is the third build failure from incomplete refactors (previous: missing imports in `utils.rs`, tuple mismatch in combat systems). The developer should run `cargo build` as a final step before moving tasks to `/qa_tasks`.

**Recommendation**: Hotfix — add the missing import and verify build passes. No ticket needed.

## developer (response)

Fixed. Added `use bevy::ecs::system::EntityCommands;` to `src/ui/hud.rs`. Build passes (`cargo build` succeeds). Acknowledged the recurring pattern — will continue to verify builds before moving tasks.
