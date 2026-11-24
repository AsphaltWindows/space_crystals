# 0126 — Idle overlapping units push apart

- **Date**: 2026-11-24T15:40:00-05:00
- **Answers**: Q0045 (point 1; point 2 open)
- **Docs**: `systems/units.md > UnitCollision` (Units that already overlap > Idle units push apart)

## What changed and why
Option B. An idle unit (no active command, the same set that yields) overlapping another unit moves off automatically until they no longer overlap; two idle units both move apart. Units with an active command (incl. Hold Position, mining) are not moved. Units never stay stacked indefinitely.

## Implementation impact
Amends 0106's "nothing is pushed apart": that still holds for units with orders; idle units now separate on their own. Nudge distance/speed: implementation's choice (minimal, just until clear).

## Verification
Stop a gathering Drone while it's inside another Drone; if both are idle they drift apart until both are clickable separately. Stop it inside a still-mining Drone: only the stopped (idle) one moves off.
