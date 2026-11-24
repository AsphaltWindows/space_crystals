# 0116 — While transforming: only Cancel; functions start at completion

- **Date**: 2026-11-24T13:40:00-05:00
- **Answers**: Q0039 (point 8; with 0109–0115, Q0039 is fully answered)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel` (new)

## What changed and why
Option A. A Beacon under deployment and a building under Warp In show only Cancel; no Set Rally Point either. Production, drop-off, research and abilities start at completion. The only function active during a Warp In is the retained 5 Beacon Capacity (0112).

## Implementation impact
One CommandPanel for every transforming object: Cancel only. Cancel hotkey TBD (Colonists panels, Q0040).

## Verification
- Select a warping-in Portal: only Cancel shown; no rally can be set.
- A Drone can't deliver to an Egress Node under Warp In.
