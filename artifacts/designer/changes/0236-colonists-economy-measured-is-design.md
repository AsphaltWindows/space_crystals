# 0236 — The measured Colonists economy is the design (Drone ×1.5 kept)

- **Date**: 2026-11-25T09:20:00-05:00
- **Answers**: Q0055, Q0072
- **Docs**: `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` (target 1, reference check, table); `factions/colonists/units.md > Drone > Patch Occupancy` (saturation estimate)

## What changed and why
User: keep the ×1.5 Drone acceleration (0205), not ×2. Leave the economy where it is (option A of Q0072).
- **The Colonists column of the table is now the M3-K3 measurement**, rounded: 6.9 / 11.4 / 13.3 / 16.0 / 18.7 / 22.9 / 25.1 / 26.3 / 27.8. The GDO column uses the M2 oracle figures.
- **Target 1 is restated**: "the Colonists lead from the start to about 1:40". The old "grows faster until 2 Drones per patch" fails only because the Colonists start slightly higher.
- **Saturation**: about 24 working Drones (3 per patch) for 32 SC/s. 2 per patch (16) is the efficient point. The 17–20 plateau (third Drones waiting) is accepted for now.
- Targets 2–4 are unchanged and hold.

## Implementation impact
- No game number changes beyond 0205.
- The oracle should pin the measured figures as the table, with the ±15% check against them, and target 1 as restated.

## Verification
Run the Colonists oracle: all figures within ±15% of the new table; targets 1 (as restated) to 4 hold.
