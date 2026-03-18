# Ticket: QA Step Tagging Convention

## Current State
All QA steps in tickets and QA tasks are written as untagged natural language instructions. The QA agent must execute every step interactively with a human, regardless of whether the step could be verified programmatically.

## Desired State
All QA steps in new tickets are tagged with one of three prefixes:

- `[auto]` — Fully automatable. Precondition, action, and verification can all be done via the Command Interface (TestHarness). The QA agent executes these programmatically without human involvement.
- `[human]` — Requires human verification. Visual checks, UX feel, animations, audio, or anything not verifiable through ECS state queries.
- `[semi]` — Automated setup, human judgment. The automated runner constructs the scenario; a human evaluates the result.

**Tagging rules for the Project Manager:**
- A step is `[auto]` when: (1) precondition can be set up via TestHarness commands, (2) action can be triggered via TestHarness commands, (3) result can be verified via TestHarness queries/assertions.
- A step is `[human]` when: verification requires visual inspection, subjective quality judgment, or real-time interaction feel.
- A step is `[semi]` when: the scenario is constructable programmatically but pass/fail requires human eyes.

This convention applies to all new tickets created by the Project Manager going forward. The ticket format's QA Steps section prefixes each step with the appropriate tag.

## Justification
Required by `features/automated_qa_system.md` Layer 2. Step tagging is the bridge between the Command Interface (Layer 1) and the Automated QA Runner (Layer 3). Without tags, the QA agent cannot distinguish which steps to run automatically vs present to a human. This is a process change — no code required, only a convention adopted in ticket authoring.

## QA Steps
1. [human] Review 3 newly created tickets (post-convention adoption) and verify each QA step has an `[auto]`, `[human]`, or `[semi]` tag prefix.
2. [human] Verify `[auto]` tagged steps describe preconditions, actions, and verifications that map to TestHarness commands/queries.
3. [human] Verify `[human]` tagged steps involve visual, UX, or subjective checks that genuinely cannot be automated.
4. [human] Verify no steps are left untagged.

## Expected Experience
Every QA step in newly created tickets has a clear `[auto]`, `[human]`, or `[semi]` prefix. The tagging is consistent with the guidelines — automatable steps are tagged `[auto]`, visual/UX steps are tagged `[human]`, and mixed steps are tagged `[semi]`. No steps are ambiguously tagged.
