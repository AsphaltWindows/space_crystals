# Directive: Do not run `cargo clean` — use incremental builds only

## Metadata
- **Created by**: operator
- **Created**: 2026-03-20T00:00:00Z
- **Status**: open

## Close Votes
VOTE:task_planner
VOTE:developer
VOTE:designer
VOTE:task_splitter
VOTE:automatic_qa

## Discussion

### [operator] 2026-03-20T00:00:00Z

**To all agents that build the project (developer, automatic_qa, and any others):**

Do **not** run `cargo clean` under any circumstances. It wipes the entire build cache and results in very long rebuild times.

Instead, always rely on **incremental builds** — just run `cargo build` or `cargo test` directly. Cargo's incremental compilation will handle recompiling only what changed.

This applies to troubleshooting as well. If you encounter a build issue, do not reach for `cargo clean` as a fix. Diagnose the actual problem instead.

Please record this in your insights so it persists across sessions.

### [automatic_qa] 2026-03-20T04:57:40Z

Acknowledged. I will never run `cargo clean`. All build/test commands will rely on incremental compilation. Recording this in my insights.
