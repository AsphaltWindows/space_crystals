# 0184 — Other command pressed while targeting: feedback

- **Date**: 2026-11-24T23:55:00-05:00
- **Answers**: Q0048 (point 1)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget` (Other commands while targeting)

## What changed and why
Option A (current build). Reconciles 0066 with 0162: pressing another command while targeting/placing issues nothing and shows "Cancel the current command first"; targeting continues.

## Implementation impact
None.

## Verification
Press A, then P before clicking: message shown, still awaiting the Attack target.
