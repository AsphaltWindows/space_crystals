# 0111 — Deploy / Warp In cancel: HP and Shields keep their fraction

- **Date**: 2026-11-24T12:50:00-05:00
- **Answers**: Q0039 (point 3)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > Cancel Deployment`; `factions/colonists/economy.md > Warp In > Cancel Warp In`

## What changed and why
Option A. On cancel, the original form (Drone, or Beacon for Warp In) returns with its own maxima and the same fraction of HP and Shields the transforming object had at cancel time, at least 1 HP. Rejected: restoring start values (free damage sponge, since cancel is free) and subtracting the interpolation gain (could kill on cancel).

## Implementation impact
current_after = max(1, original_max × current_before ÷ max_before) for HP; Shields by the same fraction (no minimum needed). Rounding otherwise per the sim's normal HP arithmetic.

## Verification
- Cancel a deploy at 90/120 → Drone 15/20.
- Cancel a Warp In at 50% HP → Beacon 60/120.
- Cancel a nearly dead transforming object: it survives at ≥ 1 HP.
