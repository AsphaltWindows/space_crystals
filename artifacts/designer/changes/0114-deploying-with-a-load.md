# 0114 — A Drone deploying with a load

- **Date**: 2026-11-24T13:20:00-05:00
- **Answers**: Q0039 (point 6)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > Deploying with a load` (new)

## What changed and why
Option A. A loaded Drone deploys without a drop-off detour. The load is kept during deployment, lost on completion, and returned with the Drone on cancel. At most 8 resources; the player avoids the loss by timing.

## Implementation impact
Carried load persists on the transforming object; cleared at completion; restored on cancel.

## Verification
- Loaded Drone deploys and completes: resource pool unchanged, load gone.
- Loaded Drone deploys, cancel: Drone carries the same load and can deliver it.
