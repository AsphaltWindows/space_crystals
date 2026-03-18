# Feature Update: Automated QA System

## Feature File
`features/automated_qa_system.md`

## Relevant Design Sources
- Forum discussions: `automated_game_testing_facility`, `agent_testing_next_phase`, `fully_automated_qa_pipeline`
- Phase 1 headless TestApp (already implemented)

## Summary of Modifications
Created new feature specification for the Automated QA System — a three-layer infrastructure to unblock the QA pipeline bottleneck (35 tasks currently backlogged).

**Layer 1 — Command Interface**: ECS-direct `TestHarness` API with spawn commands (units, structures, resources), unit commands (issue_command, set_selection), game state commands (advance_frames, set_resources, reveal_map), entity queries (position, health, attack phase, behavior), world queries (visibility, resources, entity counts), and assertion helpers. Same-process, no IPC. Built on existing Phase 1 TestApp.

**Layer 2 — QA Step Tagging**: Convention for Project Manager to tag QA steps as `[auto]` (fully automatable), `[human]` (requires visual/UX verification), or `[semi]` (automated setup, human judgment). Includes tagging guidelines and retroactive tagging plan for 35 existing tasks.

**Layer 3 — Automated QA Runner**: QA agent operates in automated mode (scheduled, non-interactive) and human review mode (interactive, user-initiated). Automated mode runs `[auto]` steps as Rust test functions, passes/fails tasks, defers human steps to `/qa_human_review`. Test generation pattern, execution via `cargo test`, and result reporting format specified.

**Pipeline changes**: New `/qa_human_review` directory for tasks needing human verification. PM tags steps during ticket creation. Supervisor schedules QA in automated mode.

**Coverage estimate**: ~72% of existing QA steps are automatable (~165/230).

**Priority**: Command Interface first (2-3 tickets, no dependencies), then tagging (1-2 tickets), then runner (2-3 tickets).
