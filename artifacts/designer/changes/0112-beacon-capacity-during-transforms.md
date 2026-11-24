# 0112 — Beacon Capacity during Deploy and Warp In; over capacity

- **Date**: 2026-11-24T13:00:00-05:00
- **Answers**: Q0039 (point 4)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > Beacon Capacity during deployment` (new); `factions/colonists/economy.md > Warp In > Beacon Capacity Retention`; `factions/colonists/overview.md > Beacon Capacity > Over capacity` (new); `factions/colonists/units.md > Drone` (intro wording)

## What changed and why
Option B (differs from the Director's proposal). The deploying Drone's 1 Beacon Capacity is freed **when deployment starts** (it is already a Beacon, 0109); the Beacon's 5 arrives on completion. A Beacon being warped in keeps providing 5 throughout.

Cancel takes the Drone's 1 back even if that puts the player over capacity. Over capacity (also reachable by losing buildings) is allowed: the counter shows red and no new units can be built until there's room again under the normal rule. Nothing is killed or removed, and the cancel is never refused.

## Implementation impact
- Used capacity −1 at deploy start; available +5 at deploy completion; used +1 on cancel.
- Over-capacity state must be representable (used > available) and displayed red in the HUD counter.

## Verification
- At 10/10, deploy a Drone: counter shows 9/10 immediately; at completion 9/15.
- At 10/10: deploy (9/10), queue a Drone (10/10), cancel the deploy: 11/10 in red; production blocked until a Beacon completes.
- Warp In a building: available capacity doesn't dip during the Warp In.
