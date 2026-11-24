# 0168 — Following itself = Stop

- **Date**: 2026-11-24T21:15:00-05:00
- **Answers**: Q0031
- **Docs**: `systems/controls/commands.md > Move` (Following itself = Stop)

## What changed and why
Option B. A unit whose Move/Follow target is itself gets the equivalent of Stop: the order ends immediately, as if arrived. Others follow it normally. Shift-queued: the step completes instantly when reached.

## Implementation impact
Fixes the never-ending self-follow from M1-G3.

## Verification
Select 6 Peacekeepers (one walking), right-click the walking one: it stops; the other 5 follow it.
