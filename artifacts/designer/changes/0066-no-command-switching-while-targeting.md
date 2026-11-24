# 0066 — While targeting, other commands are unavailable; cancel first

- **Date**: 2026-11-23T20:44:54-05:00
- **Answers**: Q0016 (point 3, first part; box-drag and selection-change while targeting, and point 4, still open)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (Other commands while targeting)

## What changed and why
The user kept the current behaviour. While a target is awaited, the panel shows the pending command plus Back/Cancel (Z), and other commands are unavailable. The player cancels with Escape, right-click or Z, then picks another command. There's no direct switching.

## Implementation impact
Matches the current behaviour.

## Verification
- Press Attack, then Patrol's hotkey: nothing happens. Press Esc, then Patrol: Patrol targeting starts.
