# 0110 — Deploy / Warp In: damage stays taken during the interpolation

- **Date**: 2026-11-24T12:40:00-05:00
- **Answers**: Q0039 (point 2)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > HP/Shields Interpolation`; `factions/colonists/economy.md > Warp In > HP/Shields Interpolation`

## What changed and why
Option A. The interpolation formulas set the **maximum** HP/Shields. Each frame the current value rises by the same amount as the maximum, so damage is carried through unchanged (not scaled). Shields regenerate normally up to the current maximum.

## Implementation impact
current += (newMax − oldMax) each frame; never a fraction-of-max rescale. Applies to damage taken before the transformation started too.

## Verification
- Drone at 10/20 HP deploys undisturbed: completes at 110/120.
- Hit a deploying Beacon for 30 at 50%: shows 40/70, completes at 90/120.
- Same arithmetic for a Warp In (Beacon 120 → target MaxHP).
