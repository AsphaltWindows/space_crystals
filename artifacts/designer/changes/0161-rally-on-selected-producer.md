# 0161 — Right-click one of several selected producers

- **Date**: 2026-11-24T20:15:00-05:00
- **Answers**: Q0035 (point 2)
- **Docs**: `systems/controls/selection_and_hud.md > Production Building Default Right-Click` (Several producers selected)

## What changed and why
Option A (current build): per-building rules. The clicked building clears its rally; the other selected producers rally to it.

## Implementation impact
None.

## Verification
Select 3 Barracks, right-click one: it shows no rally; the other two show rally lines to it.
