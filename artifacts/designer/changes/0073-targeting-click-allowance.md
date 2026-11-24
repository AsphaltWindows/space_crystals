# 0073 — Larger click allowance while targeting

- **Date**: 2026-11-24T00:45:00-05:00
- **Answers**: Q0021 (point 1; points 2–3 still open)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] > Dragging while targeting`

## What changed and why
The user picked option B. A drag while targeting still does nothing (0067). Targeting uses a **larger click allowance** before a press and release counts as a drag: **16 pixels** (placeholder, half a tile), instead of the normal drag threshold. Within that allowance, the press and release is a targeting click at the press point (0068).
- A slightly sloppy click on a moving enemy is no longer lost.
- A deliberate drag before release still aborts a press on the wrong target.

## Implementation impact
- Replaces the current 4-pixel rule while in AwaitingTarget. The normal (non-targeting) drag threshold is unchanged.
- 16 px is a placeholder. Keep it as a single tunable value.

## Verification
1. Press A, then press on an enemy, move the mouse about 10 px and release. Attack is issued on that enemy.
2. Press A, then press on an enemy, drag about 40 px and release. Nothing is issued, no box is drawn, and targeting stays active.
3. Without targeting, a 10 px drag behaves as before (normal threshold).
