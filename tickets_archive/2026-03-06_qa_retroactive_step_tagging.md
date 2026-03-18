# Ticket: Retroactive QA Step Tagging for Existing Tasks

## Current State
There are ~35 QA tasks in `/qa_tasks` with untagged QA steps. The QA agent cannot process any of them in automated mode because there are no `[auto]`/`[human]`/`[semi]` tags to distinguish automatable steps from human-required steps.

## Desired State
All existing QA tasks in `/qa_tasks` have their QA Steps sections updated with `[auto]`, `[human]`, or `[semi]` tags on every step, following the tagging convention from the QA Step Tagging Convention ticket.

This is a one-time bulk update. Each task file's QA Steps section is reviewed and each step is prefixed with the appropriate tag based on whether the step's precondition/action/verification can be handled by the Command Interface (TestHarness).

After tagging, the QA agent can begin processing tasks in automated mode — running `[auto]` steps programmatically and deferring `[human]`/`[semi]` steps to human review sessions.

## Justification
Required by `features/automated_qa_system.md` Layer 2 (retroactive tagging plan). The ~35 backlogged QA tasks represent the immediate bottleneck. Tagging them enables the automated QA runner to start clearing the backlog. Without tagging, the runner has no way to know which steps are automatable. Based on prior analysis, ~72% of steps (~165/230) are expected to be tagged `[auto]`.

**Note:** This ticket depends on the Command Interface being implemented (so the tagger knows what's automatable) and the tagging convention being established.

## QA Steps
1. [auto] Verify every `.md` file in `/qa_tasks` has QA steps prefixed with `[auto]`, `[human]`, or `[semi]` tags — no untagged steps remain.
2. [human] Spot-check 5 randomly selected QA tasks: verify `[auto]` tagged steps describe operations that map to TestHarness commands/queries, and `[human]` tagged steps genuinely require visual/UX verification.
3. [auto] Count the total number of `[auto]`, `[human]`, and `[semi]` tags across all QA tasks. Verify the `[auto]` percentage is approximately 65-80% (consistent with the ~72% estimate from prior analysis).

## Expected Experience
Every QA step in every `/qa_tasks` file has a tag prefix. The distribution roughly matches the prior estimate: ~72% `[auto]`, ~15% `[human]`, ~13% `[semi]`. Spot-checked tags are accurate — automatable steps are correctly identified as `[auto]` and visual/UX steps are correctly identified as `[human]`.
