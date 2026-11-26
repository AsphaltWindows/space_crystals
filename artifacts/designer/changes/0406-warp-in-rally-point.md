# 0406 — A portal under Warp In can have its rally point set (doc fix)

- **Date**: 2026-11-26T15:45:00-05:00
- **Answers**: Q0150
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`

## What changed and why
0396 (user, 2026-11-27) is the later decision and names "Colonists portals warping in" as an example, so it wins (option a). The Colonists doc's "no rally point" line was stale; it now carries an exception: a building under Warp In that will produce or release units (Robo or Bio Portal) also shows **C: Set Rally Point**, and right-click sets its rally point. A cancelled Warp In drops the point. A Beacon under deployment trains nothing and keeps only Cancel.

## Implementation impact
None: matches the M5-S6 build as described in Q0150.

## Verification
- Warp in a Bio Portal; while it transforms its panel shows X Cancel and C Set Rally Point; right-click ground sets the rally; after completion new units go there.
- Cancel a Warp In: the rally point is gone.
- A deploying Beacon shows only Cancel.
