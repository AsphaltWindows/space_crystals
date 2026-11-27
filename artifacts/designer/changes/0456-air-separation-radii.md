# 0456 — SeparationRadius for the Hornet, Supply Chopper, drone, Phantom and Prism

- **Date**: 2026-11-27T13:50:00-05:00
- **Answers**: Q0181
- **Docs**: `factions/gdo/units/hornet.md`, `factions/gdo/units/supply_chopper.md`, `factions/gdo/units/drone_operator.md` (the drone), `factions/syndicate/units/phantom.md`, `factions/syndicate/units/prism.md` (Placeholder stats)

## What changed and why
Option (a), as the Director assumed. These are placeholder numbers, set under the user's standing permission for placeholder stats. Each radius is the Silhouette's side + 8 su, the same rule as the three radii already given:

| Unit | SeparationRadius |
|---|---|
| Hornet | 56 |
| Supply Chopper | 68 |
| Drone | 32 |
| Phantom | 52 |
| Prism | 48 |

## Implementation impact
None beyond removing the "Placeholder (Q0181)" marks. The values are still placeholders for the numbers pass.

## Verification
Fly two of each unit onto the same spot. They push apart until their centres are about the radius apart.
