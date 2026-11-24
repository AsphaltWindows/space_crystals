# 0051 — Empty-ground clicks decided (Ctrl+Shift still open)

- **Date**: 2026-11-23T19:58:41-05:00
- **Answers**: Q0013 (point 4, mostly)
- **Docs**: `systems/controls/selection_and_hud.md > BoxSelection > Empty clicks and boxes`

## What changed and why
Clarified with the user (supersedes 0050's "not decided" for these two): a plain empty click or empty box **clears** the selection, and Shift/Ctrl-click on empty ground does **nothing**. The user had agreed to these with the earlier list. Still open: Ctrl+Shift click (placeholder: Ctrl wins).

## Implementation impact
Matches the current behaviour.

## Verification
- Units selected, left-click empty ground: the selection clears. Shift-click empty ground: the selection stays.
