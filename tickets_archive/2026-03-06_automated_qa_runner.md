# Ticket: Automated QA Runner and Human Review Pipeline

## Current State
The QA agent operates in a single mode: interactive sessions where a human walks through every QA step manually. There is no automated execution path. Tasks that could be verified programmatically still require human time. There is no `/qa_human_review` directory for tasks needing partial human verification.

## Desired State
The QA agent operates in two modes:

**1. Automated mode** (non-interactive, scheduled by supervisor):
- Picks up a QA task from `/qa_tasks`
- Parses QA steps for `[auto]`, `[human]`, and `[semi]` tags
- For each `[auto]` step: generates a Rust test function using the TestHarness API, compiles and runs it via `cargo test --features testing`
- Each test function maps to exactly one QA step
- Test files are written to `tests/qa/[task_name].rs`
- Results are collected per-step: PASS or FAIL (with error details)

**Evaluation logic:**
- If any `[auto]` step FAILs: task FAILs. Annotate with failure details and return to `/developer_tasks` with `## QA Failure` annotation. Preserve the generated test file for developer reproduction. Do NOT proceed to human steps.
- If all `[auto]` steps PASS and there are NO `[human]`/`[semi]` steps: task PASSES. Move to `/completed_tasks`.
- If all `[auto]` steps PASS and there ARE `[human]`/`[semi]` steps: move task to `/qa_human_review` with automated results annotated.

**2. Human review mode** (interactive, user-initiated):
- Walks the user through accumulated `[human]` and `[semi]` steps from `/qa_human_review`
- Same interaction flow as current QA but only for steps requiring human eyes
- Pass moves to `/completed_tasks`; fail returns to `/developer_tasks`

**Result reporting format:**
```
## Automated QA Results
- Step 1 [auto]: PASS
- Step 2 [auto]: FAIL — expected X, got Y
- Steps 3-5 [auto]: SKIPPED (prior failure)
- Steps 6-7 [human]: DEFERRED to human review
```

**New directory:** `/qa_human_review` — accumulates tasks that passed automated steps but need human verification.

## Justification
Required by `features/automated_qa_system.md` Layer 3. This is the payoff layer — it enables the QA agent to clear the majority of the backlog without human involvement. The ~72% automation rate means most tasks can be fully or partially resolved in automated mode. Mixed tasks (majority of the backlog) get their automated steps verified first, reducing human QA sessions from 30+ minutes to ~5 minutes of visual spot-checks.

**Dependencies:** Requires the Command Interface (TestHarness) and QA Step Tagging to be complete.

## QA Steps
1. [auto] Place a fully `[auto]`-tagged QA task in `/qa_tasks`. Run the QA agent in automated mode. Verify it generates a test file in `tests/qa/`, runs `cargo test`, and moves the task to `/completed_tasks` on all-pass.
2. [auto] Place a fully `[auto]`-tagged QA task with a deliberately failing step in `/qa_tasks`. Run automated mode. Verify the task is returned to `/developer_tasks` with a `## QA Failure` annotation containing the step number, expected vs actual values, and that the test file is preserved.
3. [auto] Place a mixed-tag QA task (some `[auto]`, some `[human]`) in `/qa_tasks`. Run automated mode. Verify `[auto]` steps are executed, and on all-pass the task moves to `/qa_human_review` (not `/completed_tasks`) with automated results annotated.
4. [human] In human review mode, verify the QA agent presents only the `[human]` and `[semi]` steps for a task in `/qa_human_review`, with automated results visible for context.
5. [auto] Verify that when an `[auto]` step fails, subsequent `[auto]` steps are SKIPPED and `[human]` steps are NOT presented.
6. [auto] Verify the result reporting format matches the spec: each step listed with tag, PASS/FAIL/SKIPPED/DEFERRED status, and error details for failures.
7. [human] Verify the `/qa_human_review` directory exists and tasks accumulate there correctly between human review sessions.

## Expected Experience
In automated mode, the QA agent processes tasks without user interaction. Fully automated tasks flow through to `/completed_tasks` or back to `/developer_tasks` on failure. Mixed tasks land in `/qa_human_review`. In human review mode, the user only sees the small subset of steps requiring visual verification. The backlog of 35 tasks begins decreasing as the automated runner processes them.
