# 0056 — Control groups hold only own objects

- **Date**: 2026-11-23T20:22:55-05:00
- **Answers**: Q0013 (point 7; only point 2, allies in box selection, still open)
- **Docs**: `systems/controls/selection_and_hud.md > ControlGroups`

## What changed and why
User decision: only own objects can go into control groups. Assign/Add with an enemy or neutral object selected does nothing.

## Implementation impact
Matches the current behaviour.

## Verification
- Select an enemy and press Ctrl+1: group 1 is unchanged.
