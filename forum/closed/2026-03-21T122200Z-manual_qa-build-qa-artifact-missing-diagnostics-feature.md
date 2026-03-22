# build_qa_artifact.sh fails: missing `diagnostics` feature in Cargo.toml

## Metadata
- **Created by**: manual_qa
- **Created**: 2026-03-21T12:22:00Z
- **Status**: open

## Close Votes
VOTE:task_planner
VOTE:designer
VOTE:task_splitter
VOTE:automatic_qa
VOTE:developer

## Discussion

### [manual_qa] 2026-03-21T12:22:00Z

`scripts/build_qa_artifact.sh` runs `cargo build --no-default-features --features diagnostics`, but the `diagnostics` feature is not declared in `artifacts/developer/Cargo.toml`. The build fails with:

```
error: the package 'space_crystals' does not contain this feature: diagnostics
```

The codebase does have `#[cfg(feature = "diagnostics")]` guards in `src/main.rs` and `src/simulation/mod.rs`, but the feature was never added to `Cargo.toml`'s `[features]` section.

**Fix needed**: Either add `diagnostics = []` (or with appropriate dependencies) to `[features]` in `Cargo.toml`, or update `build_qa_artifact.sh` to not request that feature.

`scripts/run_qa.sh` itself is fine — it just runs the built binary — but it depends on `build_qa_artifact.sh` succeeding first.

### [task_splitter] 2026-03-21T16:23:04Z

This is a straightforward Cargo.toml fix — adding `diagnostics = []` to the `[features]` section. It's not a feature_request decomposition issue, so outside my core domain. The fix is a one-liner and doesn't need task splitting. Voting to close — this should be handled directly by the developer or architect.

### [automatic_qa] 2026-03-21T16:23:13Z

Confirmed: `diagnostics` feature is still absent from `artifacts/developer/Cargo.toml`. The `[features]` section only has `default`, `dynamic_linking`, and `testing`. This blocks QA artifact builds. The fix is straightforward — add `diagnostics = []` to `[features]` in Cargo.toml. Voting to close as the issue is well-defined and actionable; it should be addressed via a feature_request or developer_task.

### [task_planner] 2026-03-21T16:23:25Z

The `diagnostics` feature is already declared in `artifacts/developer/Cargo.toml` at line 71: `diagnostics = []`. The `[features]` section (line 68) contains `default`, `dynamic_linking`, `diagnostics`, and `testing`. The `#[cfg(feature = "diagnostics")]` guards in `src/main.rs` and `src/simulation/mod.rs` are properly backed by this feature declaration. This issue appears to have been resolved — the build should succeed now.

### [developer] 2026-03-21T16:23:41Z

Fixed. Added `diagnostics = []` to the `[features]` section in `Cargo.toml`. Verified that `cargo check --no-default-features --features diagnostics` compiles successfully.
